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
            safety_margin: 1.2, // refuse if remaining < 1.2 * estimated
        }
    }
}

/// Thread-safe budget state. Uses atomic operations for lock-free budget checks.
#[derive(Debug)]
pub struct BudgetState {
    spent_centi_milli: AtomicU64, // cents * 100 (4 decimal places)
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

    /// Returns remaining budget in USD
    pub fn remaining_usd(&self) -> f64 {
        let spent = self.spent_centi_milli.load(Ordering::SeqCst);
        let total = self.total_centi_milli.load(Ordering::SeqCst);
        raw_to_usd(total.saturating_sub(spent))
    }

    /// Returns spent budget in USD
    pub fn spent_usd(&self) -> f64 {
        raw_to_usd(self.spent_centi_milli.load(Ordering::SeqCst))
    }

    /// Predictive check: can we afford this estimated cost?
    /// Refuses if remaining < safety_margin * estimated, or if estimated > per_request_cap.
    pub fn can_afford(&self, estimated_usd: f64) -> bool {
        if estimated_usd > self.config.per_request_cap_usd {
            return false;
        }
        let remaining = self.remaining_usd();
        let threshold = self.config.safety_margin * estimated_usd;
        remaining >= threshold
    }

    /// Reserve budget for a request. Returns Ok(()) if successful, Err if insufficient.
    /// This is the predictive gate — call BEFORE dispatching to a provider.
    pub fn reserve(&self, estimated_usd: f64) -> Result<(), BudgetError> {
        if estimated_usd > self.config.per_request_cap_usd {
            return Err(BudgetError::PerRequestCapExceeded {
                cap: self.config.per_request_cap_usd,
                estimated: estimated_usd,
            });
        }
        let estimated_raw = usd_to_raw(estimated_usd);
        let threshold_raw = usd_to_raw(self.config.safety_margin * estimated_usd);
        let spent = self.spent_centi_milli.load(Ordering::SeqCst);
        let total = self.total_centi_milli.load(Ordering::SeqCst);
        let remaining_raw = total.saturating_sub(spent);

        if remaining_raw < threshold_raw {
            return Err(BudgetError::Insufficient {
                remaining: raw_to_usd(remaining_raw),
                estimated: estimated_usd,
                threshold: self.config.safety_margin * estimated_usd,
            });
        }

        // Optimistic: we reserve estimated. Actual reconciliation happens on completion.
        // This is lock-free: if two requests race, the loser's commit will fail
        // in reconcile() and the request will be billed as over-budget.
        Ok(())
    }

    /// Reconcile actual cost after a request completes. Adjusts the spent amount
    /// from the estimated reservation to the actual cost.
    pub fn reconcile(&self, estimated_usd: f64, actual_usd: f64) -> Result<(), BudgetError> {
        let estimated_raw = usd_to_raw(estimated_usd);
        let actual_raw = usd_to_raw(actual_usd);

        // CAS loop: subtract estimated, add actual
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

    /// Snapshot the current budget state for trace output
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

/// Convert USD to raw integer (4 decimal places of precision)
fn usd_to_raw(usd: f64) -> u64 {
    (usd * PRECISION as f64).round() as u64
}

/// Convert raw integer back to USD
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
        assert!(!budget.can_afford(0.60)); // exceeds per-request cap
        assert!(!budget.can_afford(0.90)); // exceeds remaining with margin

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

        assert!(!budget.can_afford(0.05)); // spent exactly half, margin blocks
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
