use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Atomic fixed-point budget: store cents as u64 to avoid floating-point drift.
/// $1.00 = 100 cents. $0.0001 = 0.01 cents → stored as 1 (1/100th of a cent).
/// We use 4 decimal places of precision: value * 10_000 = raw.
const PRECISION: u64 = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Total budget in USD
    pub total_usd: f64,
    /// Per-request cap in USD (prevents a single request from draining everything)
    pub per_request_cap_usd: f64,
    /// Safety margin: refuse dispatch if remaining < margin * estimated_cost
    pub safety_margin: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            total_usd: 10.0,
            per_request_cap_usd: 1.0,
            safety_margin: 1.2,
        }
    }
}

/// Thread-safe budget state. Uses atomic operations for lock-free budget checks.
#[derive(Debug)]
pub struct BudgetState {
    spent_centi_milli: AtomicU64,
    total_centi_milli: AtomicU64,
    config: BudgetConfig,
}

impl BudgetState {
    pub fn new(config: BudgetConfig) -> Self {
        let total_raw = usd_to_raw(config.total_usd);
        Self {
            spent_centi_milli: AtomicU64::new(0),
            total_centi_milli: AtomicU64::new(total_raw),
            config,
        }
    }

    pub fn remaining_usd(&self) -> f64 {
        let spent = self.spent_centi_milli.load(Ordering::SeqCst);
        let total = self.total_centi_milli.load(Ordering::SeqCst);
        raw_to_usd(total.saturating_sub(spent))
    }

    pub fn spent_usd(&self) -> f64 {
        raw_to_usd(self.spent_centi_milli.load(Ordering::SeqCst))
    }

    /// Predictive check: can we afford this estimated cost?
    pub fn can_afford(&self, estimated_usd: f64) -> bool {
        if estimated_usd > self.config.per_request_cap_usd {
            return false;
        }
        let remaining = self.remaining_usd();
        let threshold = self.config.safety_margin * estimated_usd;
        remaining >= threshold
    }

    /// Atomically reserve budget for a request via CAS loop.
    /// This WRITES to spent_centi_milli, preventing concurrent requests from
    /// both passing the check. Call BEFORE dispatching to a provider.
    /// On failure, call reconcile(est, 0.0) to release the hold.
    /// On success, call reconcile(est, actual).
    pub fn reserve(&self, estimated_usd: f64) -> Result<(), BudgetError> {
        if estimated_usd > self.config.per_request_cap_usd {
            return Err(BudgetError::PerRequestCapExceeded {
                cap: self.config.per_request_cap_usd,
                estimated: estimated_usd,
            });
        }
        let estimated_raw = usd_to_raw(estimated_usd);
        let threshold_raw = usd_to_raw(self.config.safety_margin * estimated_usd);

        loop {
            let current_spent = self.spent_centi_milli.load(Ordering::SeqCst);
            let total = self.total_centi_milli.load(Ordering::SeqCst);
            let remaining_raw = total.saturating_sub(current_spent);

            if remaining_raw < threshold_raw {
                return Err(BudgetError::Insufficient {
                    remaining: raw_to_usd(remaining_raw),
                    estimated: estimated_usd,
                    threshold: self.config.safety_margin * estimated_usd,
                });
            }

            let new_spent = current_spent.saturating_add(estimated_raw);
            match self.spent_centi_milli.compare_exchange(
                current_spent,
                new_spent,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
    }

    /// Reconcile actual cost after a request completes. Subtracts the estimated
    /// reservation and adds the actual cost. Call with actual=0.0 to release a hold.
    pub fn reconcile(&self, estimated_usd: f64, actual_usd: f64) -> Result<(), BudgetError> {
        let estimated_raw = usd_to_raw(estimated_usd);
        let actual_raw = usd_to_raw(actual_usd);

        loop {
            let current = self.spent_centi_milli.load(Ordering::SeqCst);
            let adjusted = current.saturating_sub(estimated_raw).saturating_add(actual_raw);
            if self.spent_centi_milli
                .compare_exchange(current, adjusted, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    pub fn snapshot(&self) -> BudgetSnapshot {
        BudgetSnapshot {
            remaining_usd: self.remaining_usd(),
            spent_usd: self.spent_usd(),
            total_usd: raw_to_usd(self.total_centi_milli.load(Ordering::SeqCst)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    pub remaining_usd: f64,
    pub spent_usd: f64,
    pub total_usd: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("insufficient budget: remaining ${remaining:.4}, estimated ${estimated:.4}, threshold ${threshold:.4}")]
    Insufficient {
        remaining: f64,
        estimated: f64,
        threshold: f64,
    },

    #[error("per-request cap exceeded: cap ${cap:.4}, estimated ${estimated:.4}")]
    PerRequestCapExceeded { cap: f64, estimated: f64 },
}

fn usd_to_raw(usd: f64) -> u64 {
    (usd * PRECISION as f64).round() as u64
}

fn raw_to_usd(raw: u64) -> f64 {
    raw as f64 / PRECISION as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_reserve_and_reconcile() {
        let config = BudgetConfig {
            total_usd: 1.0,
            per_request_cap_usd: 0.50,
            safety_margin: 1.2,
        };
        let budget = BudgetState::new(config);

        assert!(budget.can_afford(0.01));
        assert!(!budget.can_afford(0.60));
        assert!(!budget.can_afford(0.90));

        budget.reserve(0.01).unwrap();
        budget.reconcile(0.01, 0.008).unwrap();

        assert!((budget.spent_usd() - 0.008).abs() < 0.0001);
        assert!((budget.remaining_usd() - 0.992).abs() < 0.0001);
    }

    #[test]
    fn test_budget_exhaustion() {
        let config = BudgetConfig {
            total_usd: 0.10,
            per_request_cap_usd: 0.05,
            safety_margin: 1.0,
        };
        let budget = BudgetState::new(config);

        assert!(budget.can_afford(0.05));
        budget.reserve(0.05).unwrap();
        budget.reconcile(0.05, 0.05).unwrap();

        assert!(!budget.can_afford(0.05));
        assert!(budget.can_afford(0.04));
    }

    #[test]
    fn test_concurrent_reserve() {
        let config = BudgetConfig {
            total_usd: 0.10,
            per_request_cap_usd: 0.10,
            safety_margin: 1.0,
        };
        let budget = Arc::new(BudgetState::new(config));

        let budget_clone = budget.clone();
        let handle = std::thread::spawn(move || {
            budget_clone.reserve(0.06).unwrap();
            budget_clone.reconcile(0.06, 0.06).unwrap();
        });

        handle.join().unwrap();
        assert!((budget.remaining_usd() - 0.04).abs() < 0.0001);
    }
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::sync::Barrier;

    /// THE KILLER TEST: Two concurrent reserves against a budget
    /// that can only satisfy one. With the OLD (buggy) code, both
    /// pass. With the FIXED code, exactly one succeeds.
    #[test]
    fn concurrent_reserve_prevents_double_spend() {
        let config = BudgetConfig {
            total_usd: 0.10,
            per_request_cap_usd: 0.10,
            safety_margin: 1.0,
        };
        let budget = Arc::new(BudgetState::new(config));
        let budget_clone = Arc::clone(&budget);

        let barrier = Arc::new(Barrier::new(2));

        let barrier_clone = Arc::clone(&barrier);
        let handle1 = thread::spawn(move || {
            barrier_clone.wait();
            budget.reserve(0.06)
        });

        let barrier_clone2 = Arc::clone(&barrier);
        let handle2 = thread::spawn(move || {
            barrier_clone2.wait();
            budget_clone.reserve(0.06)
        });

        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        let successes = [&result1, &result2]
            .iter()
            .filter(|r| r.is_ok())
            .count();

        assert_eq!(
            successes, 1,
            "Exactly one concurrent reserve must succeed — got {}.",
            successes
        );
    }

    /// Three threads, budget for exactly two.
    #[test]
    fn concurrent_reserve_three_way_contention() {
        let config = BudgetConfig {
            total_usd: 0.15,
            per_request_cap_usd: 0.10,
            safety_margin: 1.0,
        };
        let budget = Arc::new(BudgetState::new(config));
        let barrier = Arc::new(Barrier::new(3));

        let mut handles = vec![];
        for _ in 0..3 {
            let b = Arc::clone(&budget);
            let bar = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                bar.wait();
                b.reserve(0.06)
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let successes = results.iter().filter(|r| r.is_ok()).count();

        assert_eq!(
            successes, 2,
            "Exactly two of three concurrent reserves must succeed — got {}",
            successes
        );
    }

    /// Reserve then reconcile with refund frees budget for next reserve.
    #[test]
    fn reserve_reconcile_refund_then_reserve() {
        let config = BudgetConfig {
            total_usd: 1.0,
            per_request_cap_usd: 0.50,
            safety_margin: 1.0,
        };
        let budget = BudgetState::new(config);

        budget.reserve(0.80).expect("reserve 0.80 should succeed");
        budget.reconcile(0.80, 0.50).expect("reconcile should succeed");

        assert!((budget.spent_usd() - 0.50).abs() < 0.0001,
            "spent should be 0.50 after refund, got {}", budget.spent_usd());

        budget.reserve(0.50).expect("should have 0.50 available after refund");
    }

    /// Reserve then reconcile with overrun charges extra.
    #[test]
    fn reserve_reconcile_overrun() {
        let config = BudgetConfig {
            total_usd: 1.0,
            per_request_cap_usd: 0.50,
            safety_margin: 1.0,
        };
        let budget = BudgetState::new(config);

        budget.reserve(0.30).expect("reserve should succeed");
        budget.reconcile(0.30, 0.45).expect("reconcile should succeed");

        assert!((budget.spent_usd() - 0.45).abs() < 0.0001,
            "spent should be 0.45 after overrun, got {}", budget.spent_usd());

        budget.reserve(0.50).expect("should have 0.55 available");
        budget.reserve(0.60).expect_err("should fail — only 0.05 remaining");
    }

    /// Sequential reserves respect the budget limit exactly.
    #[test]
    fn sequential_reserve_exact_limit() {
        let config = BudgetConfig {
            total_usd: 0.10,
            per_request_cap_usd: 0.10,
            safety_margin: 1.0,
        };
        let budget = BudgetState::new(config);

        assert!(budget.reserve(0.06).is_ok(), "first reserve of 0.06 should succeed");
        assert!(budget.reserve(0.06).is_err(),
            "second reserve of 0.06 should fail — only 0.04 left");
        assert!(budget.reserve(0.04).is_ok(),
            "reserve of 0.04 should succeed — exactly fits");
        assert!(budget.reserve(0.01).is_err(),
            "reserve of 0.01 should fail — budget exhausted");
    }

    /// Reconcile without prior reserve is safe (error path recovery).
    #[test]
    fn reconcile_without_reserve_is_safe() {
        let config = BudgetConfig {
            total_usd: 1.0,
            per_request_cap_usd: 0.50,
            safety_margin: 1.0,
        };
        let budget = BudgetState::new(config);

        let result = budget.reconcile(0.10, 0.05);
        assert!(result.is_ok(), "reconcile without reserve should not panic");

        assert!((budget.spent_usd() - 0.05).abs() < 0.0001,
            "spent should be 0.05 after reconcile-without-reserve, got {}",
            budget.spent_usd());
    }
}
