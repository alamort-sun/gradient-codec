use crate::adapter::ProviderId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Classification of a failure — determines how the retry is routed.
/// This is Susano's domain: breaking what was built, then deciding what to try next.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureClass {
    /// Request took too long. Retry with a faster provider or shorter prompt.
    Timeout,
    /// Provider rate-limited. Retry with a different provider, or back off.
    RateLimit,
    /// Content policy violation. Retry with a provider that has different policy.
    ContentPolicy,
    /// Provider returned malformed/unparseable output. Retry with a more reliable provider.
    MalformedOutput,
    /// Output was syntactically valid but quality was below threshold.
    /// Retry with a higher-quality model.
    QualityDegradation,
    /// Generic provider error (5xx, network, etc).
    ProviderError { status_code: u16 },
}

impl FailureClass {
    /// Returns the recommended retry strategy for this failure class.
    pub fn retry_strategy(&self) -> RetryStrategy {
        match self {
            FailureClass::Timeout => RetryStrategy::PreferFaster,
            FailureClass::RateLimit => RetryStrategy::PreferAlternative,
            FailureClass::ContentPolicy => RetryStrategy::PreferAlternative,
            FailureClass::MalformedOutput => RetryStrategy::PreferMoreReliable,
            FailureClass::QualityDegradation => RetryStrategy::PreferHigherQuality,
            FailureClass::ProviderError { .. } => RetryStrategy::PreferAlternative,
        }
    }

    /// Should this failure class blocklist the provider temporarily?
    pub fn blocklist_duration_s(&self) -> u64 {
        match self {
            FailureClass::Timeout => 30,
            FailureClass::RateLimit => 60,
            FailureClass::ContentPolicy => 0, // don't block — just reroute
            FailureClass::MalformedOutput => 0,
            FailureClass::QualityDegradation => 0,
            FailureClass::ProviderError { status_code } => {
                if *status_code >= 500 { 120 } else { 0 }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetryStrategy {
    /// Prefer a provider with lower avg latency
    PreferFaster,
    /// Prefer any provider that isn't the one that just failed
    PreferAlternative,
    /// Prefer a provider with stricter output formatting / structured output support
    PreferMoreReliable,
    /// Prefer a provider with higher quality score
    PreferHigherQuality,
}

/// A recorded failure event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    pub provider: ProviderId,
    pub class: FailureClass,
    pub timestamp: DateTime<Utc>,
    pub request_prompt_hash: String,
}

/// Thread-safe failure history — shared across the orchestration engine.
/// Tracks recent failures per provider and provides blocklist + penalty data.
#[derive(Debug)]
pub struct FailureHistory {
    events: Arc<Mutex<Vec<FailureEvent>>>,
    /// Blocklist: provider → until when
    blocklist: Arc<Mutex<HashMap<ProviderId, DateTime<Utc>>>>,
    /// Max events to retain (ring buffer)
    max_events: usize,
}

impl FailureHistory {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            blocklist: Arc::new(Mutex::new(HashMap::new())),
            max_events: 1000,
        }
    }

    /// Record a failure and update the blocklist if applicable
    pub fn record(&mut self, provider: ProviderId, class: FailureClass) {
        let event = FailureEvent {
            provider: provider.clone(),
            class: class.clone(),
            timestamp: Utc::now(),
            request_prompt_hash: String::new(), // filled by caller in production
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        // Trim to max_events (ring buffer behavior)
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        // Update blocklist
        let block_duration = class.blocklist_duration_s();
        if block_duration > 0 {
            let mut bl = self.blocklist.lock().unwrap();
            bl.insert(provider, Utc::now() + chrono::Duration::seconds(block_duration as i64));
        }
    }

    /// Is this provider currently blocklisted?
    pub fn is_blocklisted(&self, provider: &ProviderId) -> bool {
        let bl = self.blocklist.lock().unwrap();
        if let Some(until) = bl.get(provider) {
            return Utc::now() < *until;
        }
        false
    }

    /// Count recent failures for a provider (last N events)
    pub fn recent_failure_count(&self, provider: &ProviderId) -> usize {
        let events = self.events.lock().unwrap();
        events.iter()
            .filter(|e| &e.provider == provider)
            .count()
    }

    /// Get the most recent failure class for a provider
    pub fn last_failure(&self, provider: &ProviderId) -> Option<FailureClass> {
        let events = self.events.lock().unwrap();
        events.iter()
            .rev()
            .find(|e| &e.provider == provider)
            .map(|e| e.class.clone())
    }

    /// Get all failures for a specific provider
    pub fn failures_for(&self, provider: &ProviderId) -> Vec<FailureEvent> {
        let events = self.events.lock().unwrap();
        events.iter()
            .filter(|e| &e.provider == provider)
            .cloned()
            .collect()
    }

    /// Clear expired blocklist entries
    pub fn prune_blocklist(&self) {
        let mut bl = self.blocklist.lock().unwrap();
        let now = Utc::now();
        bl.retain(|_, until| *until > now);
    }

    /// Snapshot for trace output
    pub fn snapshot(&self) -> FailureHistorySnapshot {
        let events = self.events.lock().unwrap();
        FailureHistorySnapshot {
            total_events: events.len(),
            providers_with_failures: events.iter()
                .map(|e| e.provider.clone())
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

impl Default for FailureHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FailureHistory {
    fn clone(&self) -> Self {
        let events = self.events.lock().unwrap().clone();
        let blocklist = self.blocklist.lock().unwrap().clone();
        Self {
            events: Arc::new(Mutex::new(events)),
            blocklist: Arc::new(Mutex::new(blocklist)),
            max_events: self.max_events,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureHistorySnapshot {
    pub total_events: usize,
    pub providers_with_failures: usize,
}

/// The failure critic trait — classifies failures and recommends retry targets.
/// This is the Susano component: it breaks what was built, understands why,
/// and decides what to try next.
pub trait FailureCritic: Send + Sync {
    /// Classify a failure into a FailureClass
    fn classify(&self, error: &crate::errors::GcError) -> FailureClass;

    /// Given a failure class and available providers, recommend a retry target.
    /// Returns None if retry is not recommended.
    fn recommend_retry(
        &self,
        class: &FailureClass,
        failed_provider: &ProviderId,
        available: &[crate::adapter::ProviderInfo],
    ) -> Option<ProviderId>;
}

/// Default failure critic with heuristic classification
pub struct DefaultCritic;

impl FailureCritic for DefaultCritic {
    fn classify(&self, error: &crate::errors::GcError) -> FailureClass {
        match error {
            crate::errors::GcError::ProviderTimeout { .. } => FailureClass::Timeout,
            crate::errors::GcError::RateLimited { .. } => FailureClass::RateLimit,
            crate::errors::GcError::ContentPolicy { .. } => FailureClass::ContentPolicy,
            crate::errors::GcError::MalformedOutput { .. } => FailureClass::MalformedOutput,
            crate::errors::GcError::ProviderError { status_code, .. } => {
                FailureClass::ProviderError { status_code: *status_code }
            }
            _ => FailureClass::ProviderError { status_code: 0 },
        }
    }

    fn recommend_retry(
        &self,
        class: &FailureClass,
        failed_provider: &ProviderId,
        available: &[crate::adapter::ProviderInfo],
    ) -> Option<ProviderId> {
        let strategy = class.retry_strategy();

        let candidates: Vec<&crate::adapter::ProviderInfo> = available
            .iter()
            .filter(|p| &p.id != failed_provider)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        match strategy {
            RetryStrategy::PreferFaster => {
                candidates.iter()
                    .min_by_key(|p| p.avg_latency_ms)
                    .map(|p| p.id.clone())
            }
            RetryStrategy::PreferAlternative => {
                // Just pick the first available alternative
                candidates.first().map(|p| p.id.clone())
            }
            RetryStrategy::PreferMoreReliable => {
                // Prefer providers that support tools and streaming (more mature)
                candidates.iter()
                    .max_by_key(|p| (p.supports_tools as u8, p.supports_streaming as u8))
                    .map(|p| p.id.clone())
            }
            RetryStrategy::PreferHigherQuality => {
                // Prefer providers with higher output price (quality proxy)
                candidates.iter()
                    .max_by(|a, b| {
                        a.price_per_million_output.partial_cmp(&b.price_per_million_output)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|p| p.id.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocklist_on_timeout() {
        let mut history = FailureHistory::new();
        let provider = ProviderId("openai/gpt-4o-mini".into());

        history.record(provider.clone(), FailureClass::Timeout);

        assert!(history.is_blocklisted(&provider));
        assert_eq!(history.recent_failure_count(&provider), 1);
    }

    #[test]
    fn test_no_blocklist_on_quality_degradation() {
        let mut history = FailureHistory::new();
        let provider = ProviderId("openai/gpt-4o-mini".into());

        history.record(provider.clone(), FailureClass::QualityDegradation);

        assert!(!history.is_blocklisted(&provider));
        assert_eq!(history.recent_failure_count(&provider), 1);
    }

    #[test]
    fn test_critic_classifies_timeout() {
        let critic = DefaultCritic;
        let error = crate::errors::GcError::ProviderTimeout {
            provider: "openai".into(),
            timeout_ms: 5000,
        };
        let class = critic.classify(&error);
        assert_eq!(class, FailureClass::Timeout);
        assert_eq!(class.retry_strategy(), RetryStrategy::PreferFaster);
    }

    #[test]
    fn test_critic_recommends_faster_on_timeout() {
        let critic = DefaultCritic;
        let failed = ProviderId("anthropic/claude-sonnet".into());
        let available = vec![
            crate::adapter::ProviderInfo {
                id: ProviderId("openai/gpt-4o-mini".into()),
                display_name: "GPT-4o mini".into(),
                price_per_million_input: 0.15,
                price_per_million_output: 0.60,
                max_output_tokens: 16384,
                avg_latency_ms: 800,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            crate::adapter::ProviderInfo {
                id: ProviderId("ollama/local".into()),
                display_name: "Ollama".into(),
                price_per_million_input: 0.0,
                price_per_million_output: 0.0,
                max_output_tokens: 4096,
                avg_latency_ms: 2000,
                supports_streaming: true,
                supports_tools: false,
                is_local: true,
            },
        ];

        let recommendation = critic.recommend_retry(&FailureClass::Timeout, &failed, &available);
        assert_eq!(recommendation.unwrap().0, "openai/gpt-4o-mini"); // fastest
    }
}
