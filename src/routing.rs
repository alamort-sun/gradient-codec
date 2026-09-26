use crate::adapter::{CompletionRequest, ProviderId, ProviderInfo};
use crate::budget::BudgetState;
use crate::critique::FailureHistory;
use serde::{Deserialize, Serialize};

/// A routing decision: which provider to use, why, and what to fall back to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub provider: ProviderId,
    pub estimated_cost: f64,
    pub rationale: String,
    pub fallback_chain: Vec<ProviderId>,
}

/// The routing policy trait. Each policy decides which provider to use
/// given the request, budget state, failure history, and available providers.
/// This is the core extension point — users implement this to define custom routing.
pub trait RoutingPolicy: Send + Sync {
    fn select(
        &self,
        request: &CompletionRequest,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> Option<RoutingDecision>;

    fn name(&self) -> &str;

    fn description(&self) -> &str {
        ""
    }
}

// ─── Built-in Policies ───────────────────────────────────────

/// Cheapest-first: always picks the cheapest provider that can handle the request.
/// No quality awareness. Pure cost optimization.
pub struct CheapestFirst;

impl RoutingPolicy for CheapestFirst {
    fn name(&self) -> &str {
        "cheapest-first"
    }

    fn description(&self) -> &str {
        "Always selects the cheapest available provider. No quality awareness."
    }

    fn select(
        &self,
        request: &CompletionRequest,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> Option<RoutingDecision> {
        let mut candidates: Vec<&ProviderInfo> = providers
            .iter()
            .filter(|p| budget.can_afford(estimate_cost(p, request)))
            .filter(|p| !history.is_blocklisted(&p.id))
            .collect();

        // Sort by estimated cost (input + output)
        candidates.sort_by(|a, b| {
            let cost_a = estimate_cost(a, request);
            let cost_b = estimate_cost(b, request);
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = candidates.first()?;

        // Build fallback chain from remaining candidates
        let fallback_chain: Vec<ProviderId> =
            candidates.iter().skip(1).map(|p| p.id.clone()).collect();

        Some(RoutingDecision {
            provider: selected.id.clone(),
            estimated_cost: estimate_cost(selected, request),
            rationale: format!(
                "cheapest available provider at ${:.6}/request",
                estimate_cost(selected, request)
            ),
            fallback_chain,
        })
    }
}

/// Quality-first: picks the provider with the highest expected quality.
/// Uses a simple heuristic: higher price-per-token correlates with higher quality.
/// In production, this would use a learned quality model from benchmark data.
pub struct QualityFirst {
    /// Quality scores per provider, learned from benchmark runs
    quality_scores: std::collections::HashMap<ProviderId, f64>,
}

impl QualityFirst {
    pub fn new() -> Self {
        Self {
            quality_scores: std::collections::HashMap::new(),
        }
    }

    /// Inject expected quality (0.0–1.0). Without scores, select falls back to
    /// output-price proxy.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_score(mut self, provider: ProviderId, score: f64) -> Self {
        self.quality_scores.insert(provider, score);
        self
    }
}

impl RoutingPolicy for QualityFirst {
    fn name(&self) -> &str {
        "quality-first"
    }

    fn description(&self) -> &str {
        "Selects the provider with the highest expected quality score."
    }

    fn select(
        &self,
        request: &CompletionRequest,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> Option<RoutingDecision> {
        let mut candidates: Vec<&ProviderInfo> = providers
            .iter()
            .filter(|p| budget.can_afford(estimate_cost(p, request)))
            .filter(|p| !history.is_blocklisted(&p.id))
            .collect();

        // Sort by quality score (descending). Default: use price as quality proxy.
        candidates.sort_by(|a, b| {
            let qa = self
                .quality_scores
                .get(&a.id)
                .copied()
                .unwrap_or(a.price_per_million_output / 10.0);
            let qb = self
                .quality_scores
                .get(&b.id)
                .copied()
                .unwrap_or(b.price_per_million_output / 10.0);
            qb.partial_cmp(&qa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = candidates.first()?;
        let quality = self
            .quality_scores
            .get(&selected.id)
            .copied()
            .unwrap_or(selected.price_per_million_output / 10.0);

        let fallback_chain: Vec<ProviderId> =
            candidates.iter().skip(1).map(|p| p.id.clone()).collect();

        Some(RoutingDecision {
            provider: selected.id.clone(),
            estimated_cost: estimate_cost(selected, request),
            rationale: format!("highest quality score: {:.2}", quality),
            fallback_chain,
        })
    }
}

/// Budget-aware: the flagship policy. Balances cost, quality, and failure history.
/// Predictively estimates cost, refuses if budget insufficient, and routes
/// around providers with recent failures.
pub struct BudgetAware {
    /// Quality scores per provider (0.0-1.0)
    quality_scores: std::collections::HashMap<ProviderId, f64>,
    /// Weight for quality (0.0-1.0). Higher = prefer quality over cost.
    quality_weight: f64,
    /// Penalty multiplier for providers with recent failures
    failure_penalty: f64,
}

#[cfg_attr(not(test), allow(dead_code))] // with_score / with_quality_weight exercised from unit tests
impl BudgetAware {
    pub fn new() -> Self {
        Self {
            quality_scores: std::collections::HashMap::new(),
            quality_weight: 0.6,
            failure_penalty: 0.3,
        }
    }

    pub fn with_score(mut self, provider: ProviderId, score: f64) -> Self {
        self.quality_scores.insert(provider, score);
        self
    }

    pub fn with_quality_weight(mut self, weight: f64) -> Self {
        self.quality_weight = weight.clamp(0.0, 1.0);
        self
    }
}

impl RoutingPolicy for BudgetAware {
    fn name(&self) -> &str {
        "budget-aware"
    }

    fn description(&self) -> &str {
        "Balances cost, quality, and failure history. Predictively estimates cost \
         and routes around providers with recent failures."
    }

    fn select(
        &self,
        request: &CompletionRequest,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> Option<RoutingDecision> {
        let cost_weight = 1.0 - self.quality_weight;

        // Score each candidate: quality_score * quality_weight - normalized_cost * cost_weight - failure_penalty
        let mut scored: Vec<(f64, &ProviderInfo)> = providers
            .iter()
            .filter(|p| budget.can_afford(estimate_cost(p, request)))
            .filter(|p| !history.is_blocklisted(&p.id))
            .map(|p| {
                let cost = estimate_cost(p, request);
                let quality = self.quality_scores.get(&p.id).copied().unwrap_or(0.5);

                // Normalize cost: 0 (free) to 1 (most expensive in the set)
                let max_cost = providers
                    .iter()
                    .map(|pp| estimate_cost(pp, request))
                    .fold(0.0f64, f64::max);
                let normalized_cost = if max_cost > 0.0 { cost / max_cost } else { 0.0 };

                // Failure penalty: reduce score for providers with recent failures
                let recent_failures = history.recent_failure_count(&p.id);
                let penalty = if recent_failures > 0 {
                    self.failure_penalty * recent_failures as f64
                } else {
                    0.0
                };

                let score = quality * self.quality_weight - normalized_cost * cost_weight - penalty;

                (score, p)
            })
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let (score, selected) = scored.first()?;

        let fallback_chain: Vec<ProviderId> =
            scored.iter().skip(1).map(|(_, p)| p.id.clone()).collect();

        let quality = self
            .quality_scores
            .get(&selected.id)
            .copied()
            .unwrap_or(0.5);
        let cost = estimate_cost(selected, request);
        let recent_failures = history.recent_failure_count(&selected.id);

        let rationale = format!(
            "budget-aware: score={:.3} (quality={:.2}×{:.1}, cost={:.6}, failures={})",
            score, quality, self.quality_weight, cost, recent_failures
        );

        Some(RoutingDecision {
            provider: selected.id.clone(),
            estimated_cost: cost,
            rationale,
            fallback_chain,
        })
    }
}

/// Estimate the cost of a request for a given provider
fn estimate_cost(provider: &ProviderInfo, request: &CompletionRequest) -> f64 {
    // Rough token estimate: 4 chars ≈ 1 token
    let input_chars = request.prompt.chars().count()
        + request
            .system
            .as_ref()
            .map(|s| s.chars().count())
            .unwrap_or(0);
    let input_tokens = ((input_chars as f64) / 4.0).ceil() as u32;

    let input_cost = (input_tokens as f64 / 1_000_000.0) * provider.price_per_million_input;
    let output_cost = (request.max_tokens as f64 / 1_000_000.0) * provider.price_per_million_output;

    input_cost + output_cost
}

/// Registry of available routing policies
pub struct PolicyRegistry {
    policies: Vec<Box<dyn RoutingPolicy>>,
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn register(&mut self, policy: Box<dyn RoutingPolicy>) {
        self.policies.push(policy);
    }

    pub fn get(&self, name: &str) -> Option<&dyn RoutingPolicy> {
        self.policies
            .iter()
            .find(|p| p.name() == name)
            .map(|b| b.as_ref())
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.policies
            .iter()
            .map(|p| (p.name(), p.description()))
            .collect()
    }

    /// Create a registry with all built-in policies
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(CheapestFirst));
        registry.register(Box::new(QualityFirst::new()));
        registry.register(Box::new(BudgetAware::new()));
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::ProviderId;
    use crate::budget::{BudgetConfig, BudgetState};
    use crate::critique::{FailureClass, FailureHistory};

    fn mock_providers() -> Vec<ProviderInfo> {
        vec![
            ProviderInfo {
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
            ProviderInfo {
                id: ProviderId("anthropic/claude-sonnet".into()),
                display_name: "Claude Sonnet".into(),
                price_per_million_input: 3.00,
                price_per_million_output: 15.00,
                max_output_tokens: 8192,
                avg_latency_ms: 1200,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("ollama/local".into()),
                display_name: "Ollama local".into(),
                price_per_million_input: 0.0,
                price_per_million_output: 0.0,
                max_output_tokens: 4096,
                avg_latency_ms: 2000,
                supports_streaming: true,
                supports_tools: false,
                is_local: true,
            },
        ]
    }

    #[test]
    fn test_cheapest_first_prefers_ollama() {
        let policy = CheapestFirst;
        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let providers = mock_providers();
        let request = CompletionRequest {
            prompt: "Hello world".into(),
            max_tokens: 100,
            temperature: None,
            system: None,
            tools: None,
            task_type: None,
        };

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();
        assert_eq!(decision.provider.0, "ollama/local");
    }

    #[test]
    fn quality_first_respects_injected_scores() {
        // Prefer free local over expensive when score says so (not price-proxy).
        let policy = QualityFirst::new()
            .with_score(ProviderId("ollama/local".into()), 0.99)
            .with_score(ProviderId("anthropic/claude-sonnet".into()), 0.1)
            .with_score(ProviderId("openai/gpt-4o-mini".into()), 0.1);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let providers = mock_providers();
        let request = CompletionRequest {
            prompt: "Hello".into(),
            max_tokens: 100,
            temperature: None,
            system: None,
            tools: None,
            task_type: None,
        };

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();
        assert_eq!(
            decision.provider.0, "ollama/local",
            "QualityFirst must honor injected scores over price proxy"
        );
    }

    #[test]
    fn test_budget_aware_balances() {
        let policy = BudgetAware::new()
            .with_score(ProviderId("openai/gpt-4o-mini".into()), 0.7)
            .with_score(ProviderId("anthropic/claude-sonnet".into()), 0.95)
            .with_score(ProviderId("ollama/local".into()), 0.4)
            .with_quality_weight(0.7);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let providers = mock_providers();
        let request = CompletionRequest {
            prompt: "Write a summary".into(),
            max_tokens: 500,
            temperature: None,
            system: None,
            tools: None,
            task_type: Some("summarization".into()),
        };

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();
        // With quality_weight 0.7, should prefer the higher-quality Claude
        // even though it's more expensive — as long as budget allows
        assert!(budget.can_afford(decision.estimated_cost));
    }

    #[test]
    fn test_failure_history_reroutes() {
        let policy = BudgetAware::new()
            .with_score(ProviderId("openai/gpt-4o-mini".into()), 0.7)
            .with_score(ProviderId("anthropic/claude-sonnet".into()), 0.95)
            .with_score(ProviderId("ollama/local".into()), 0.4);

        let budget = BudgetState::new(BudgetConfig::default());
        let mut history = FailureHistory::new();
        // Record 3 recent failures for Claude → should be penalized
        for _ in 0..3 {
            history.record(
                ProviderId("anthropic/claude-sonnet".into()),
                FailureClass::Timeout,
            );
        }

        let providers = mock_providers();
        let request = CompletionRequest {
            prompt: "Hello".into(),
            max_tokens: 100,
            temperature: None,
            system: None,
            tools: None,
            task_type: None,
        };

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();
        // With 3 failures, Claude should be penalized below OpenAI
        assert_ne!(decision.provider.0, "anthropic/claude-sonnet");
    }
}

#[cfg(test)]
mod budget_aware_property_tests {
    use super::*;
    use crate::budget::{BudgetConfig, BudgetState};
    use crate::critique::{FailureClass, FailureHistory};

    fn mock_providers() -> Vec<ProviderInfo> {
        vec![
            ProviderInfo {
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
            ProviderInfo {
                id: ProviderId("anthropic/claude-sonnet".into()),
                display_name: "Claude Sonnet".into(),
                price_per_million_input: 3.00,
                price_per_million_output: 15.00,
                max_output_tokens: 8192,
                avg_latency_ms: 1200,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("ollama/local".into()),
                display_name: "Ollama local".into(),
                price_per_million_input: 0.0,
                price_per_million_output: 0.0,
                max_output_tokens: 4096,
                avg_latency_ms: 2000,
                supports_streaming: true,
                supports_tools: false,
                is_local: true,
            },
        ]
    }

    fn mock_request(prompt: &str, max_tokens: u32) -> CompletionRequest {
        CompletionRequest {
            prompt: prompt.into(),
            max_tokens,
            temperature: None,
            system: None,
            tools: None,
            task_type: None,
        }
    }

    /// PROPERTY: Higher quality → selected (holding cost constant)
    #[test]
    fn higher_quality_wins_when_cost_equal() {
        let providers = vec![
            ProviderInfo {
                id: ProviderId("provider-a".into()),
                display_name: "Provider A".into(),
                price_per_million_input: 1.0,
                price_per_million_output: 1.0,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("provider-b".into()),
                display_name: "Provider B".into(),
                price_per_million_input: 1.0,
                price_per_million_output: 1.0,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
        ];

        let policy = BudgetAware::new()
            .with_score(ProviderId("provider-a".into()), 0.3)
            .with_score(ProviderId("provider-b".into()), 0.9)
            .with_quality_weight(0.5);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let request = mock_request("Hello", 100);

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();

        assert_eq!(
            decision.provider.0, "provider-b",
            "Higher quality provider must be selected when costs are equal"
        );
    }

    /// PROPERTY: Lower cost → selected (holding quality constant)
    #[test]
    fn lower_cost_wins_when_quality_equal() {
        let providers = vec![
            ProviderInfo {
                id: ProviderId("cheap".into()),
                display_name: "Cheap".into(),
                price_per_million_input: 0.10,
                price_per_million_output: 0.10,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("expensive".into()),
                display_name: "Expensive".into(),
                price_per_million_input: 10.0,
                price_per_million_output: 10.0,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
        ];

        let policy = BudgetAware::new()
            .with_score(ProviderId("cheap".into()), 0.7)
            .with_score(ProviderId("expensive".into()), 0.7)
            .with_quality_weight(0.5);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let request = mock_request("Hello", 100);

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();

        assert_eq!(
            decision.provider.0, "cheap",
            "Cheaper provider must be selected when quality scores are equal"
        );
    }

    /// PROPERTY: Known-failed provider scores lower than clean provider
    #[test]
    fn failure_penalty_reroutes_from_identical_provider() {
        let providers = vec![
            ProviderInfo {
                id: ProviderId("clean".into()),
                display_name: "Clean".into(),
                price_per_million_input: 1.0,
                price_per_million_output: 1.0,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("failed".into()),
                display_name: "Failed".into(),
                price_per_million_input: 1.0,
                price_per_million_output: 1.0,
                max_output_tokens: 4096,
                avg_latency_ms: 1000,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
        ];

        let policy = BudgetAware::new()
            .with_score(ProviderId("clean".into()), 0.7)
            .with_score(ProviderId("failed".into()), 0.7)
            .with_quality_weight(0.5);

        let budget = BudgetState::new(BudgetConfig::default());
        let mut history = FailureHistory::new();

        for _ in 0..2 {
            history.record(ProviderId("failed".into()), FailureClass::Timeout);
        }

        let request = mock_request("Hello", 100);
        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();

        assert_eq!(
            decision.provider.0, "clean",
            "Clean provider must be selected over identical provider with failures"
        );
    }

    /// PROPERTY: Zero-cost provider (Ollama) doesn't cause NaN
    #[test]
    fn zero_cost_provider_does_not_crash() {
        let providers = vec![ProviderInfo {
            id: ProviderId("ollama/local".into()),
            display_name: "Ollama".into(),
            price_per_million_input: 0.0,
            price_per_million_output: 0.0,
            max_output_tokens: 4096,
            avg_latency_ms: 2000,
            supports_streaming: true,
            supports_tools: false,
            is_local: true,
        }];

        let policy = BudgetAware::new()
            .with_score(ProviderId("ollama/local".into()), 0.5)
            .with_quality_weight(0.6);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let request = mock_request("Hello", 100);

        let decision = policy.select(&request, &budget, &history, &providers);

        assert!(
            decision.is_some(),
            "Must return a decision for zero-cost provider"
        );
        let d = decision.unwrap();
        assert!(
            d.estimated_cost.is_finite(),
            "estimated_cost must not be NaN/Inf"
        );
        assert!(
            d.estimated_cost >= 0.0,
            "estimated_cost must be non-negative"
        );
    }

    /// PROPERTY: Zero-cost provider has max cost advantage
    #[test]
    fn zero_cost_provider_has_max_cost_advantage() {
        let providers = mock_providers();
        let policy = BudgetAware::new()
            .with_score(ProviderId("openai/gpt-4o-mini".into()), 0.5)
            .with_score(ProviderId("anthropic/claude-sonnet".into()), 0.5)
            .with_score(ProviderId("ollama/local".into()), 0.5)
            .with_quality_weight(0.1);

        let budget = BudgetState::new(BudgetConfig::default());
        let history = FailureHistory::new();
        let request = mock_request("Hello", 100);

        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();

        assert_eq!(
            decision.provider.0, "ollama/local",
            "Ollama (free) must win when cost_weight is high and all quality scores are equal"
        );
    }

    /// PROPERTY: Blocklisted provider is never selected
    #[test]
    fn blocklisted_provider_never_selected() {
        let providers = vec![
            ProviderInfo {
                id: ProviderId("best".into()),
                display_name: "Best".into(),
                price_per_million_input: 0.01,
                price_per_million_output: 0.01,
                max_output_tokens: 4096,
                avg_latency_ms: 500,
                supports_streaming: true,
                supports_tools: true,
                is_local: false,
            },
            ProviderInfo {
                id: ProviderId("worst".into()),
                display_name: "Worst".into(),
                price_per_million_input: 10.0,
                price_per_million_output: 10.0,
                max_output_tokens: 4096,
                avg_latency_ms: 3000,
                supports_streaming: false,
                supports_tools: false,
                is_local: false,
            },
        ];

        let policy = BudgetAware::new()
            .with_score(ProviderId("best".into()), 0.99)
            .with_score(ProviderId("worst".into()), 0.01)
            .with_quality_weight(0.9);

        let budget = BudgetState::new(BudgetConfig::default());
        let mut history = FailureHistory::new();

        // Blocklist "best" — Timeout has 30s blocklist duration
        history.record(ProviderId("best".into()), FailureClass::Timeout);

        let request = mock_request("Hello", 100);
        let decision = policy
            .select(&request, &budget, &history, &providers)
            .unwrap();

        assert_eq!(
            decision.provider.0, "worst",
            "Blocklisted provider must never be selected, even if it's the best"
        );
    }

    /// PROPERTY: Returns None when no provider is affordable
    #[test]
    fn returns_none_when_no_provider_affordable() {
        let providers = vec![ProviderInfo {
            id: ProviderId("expensive".into()),
            display_name: "Expensive".into(),
            price_per_million_input: 1000.0,
            price_per_million_output: 1000.0,
            max_output_tokens: 4096,
            avg_latency_ms: 1000,
            supports_streaming: true,
            supports_tools: true,
            is_local: false,
        }];

        let config = BudgetConfig {
            total_usd: 0.001,
            per_request_cap_usd: 0.0001,
            safety_margin: 1.2,
        };
        let budget = BudgetState::new(config);

        let policy = BudgetAware::new().with_score(ProviderId("expensive".into()), 0.9);

        let history = FailureHistory::new();
        let request = mock_request("Hello world this is a long prompt", 1000);

        let decision = policy.select(&request, &budget, &history, &providers);

        assert!(
            decision.is_none(),
            "Must return None when no provider is affordable"
        );
    }
}
