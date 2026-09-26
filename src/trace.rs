//! Ephemeral orchestration trace (in-memory only).
//!
//! Durable joinable identity (`trace_id` / `original_trace_id` / `prompt_hash`)
//! and the write/replay path (`write_to_file`, `load_trace`, CLI `replay`) were
//! removed (Susano B2 / dry-audit v1.3 + A2 residual cut). Content-addressed
//! replacement: Saraswati A2 `ClosureReceipt` + `ActDigest` in `gradient-plane`.

use crate::adapter::{CompletionRequest, CompletionResponse, ProviderId, TokenUsage};
use crate::budget::BudgetSnapshot;
use crate::critique::FailureClass;
use crate::routing::RoutingDecision;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete in-memory trace of a single orchestration step.
/// Not durable: no joinable identity fields, no filesystem write/replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub timestamp: DateTime<Utc>,
    pub request: TraceRequest,
    pub routing: RoutingDecision,
    pub result: Option<TraceResult>,
    pub failure: Option<TraceFailure>,
    pub budget_after: BudgetSnapshot,
}

/// Ephemeral request slice. No content hash — hashes are join keys (R2 / A5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRequest {
    pub max_tokens: u32,
    pub temperature: Option<f64>,
    pub task_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResult {
    pub provider: ProviderId,
    pub text: String,
    pub usage: TokenUsage,
    pub actual_cost_usd: f64,
    pub latency_ms: u64,
    pub quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFailure {
    pub provider: ProviderId,
    pub class: FailureClass,
    pub error_message: String,
    pub retry_attempted: bool,
    pub retry_provider: Option<ProviderId>,
}

impl Trace {
    /// Create a new in-memory trace from a request and routing decision.
    pub fn new(request: &CompletionRequest, routing: RoutingDecision) -> Self {
        Self {
            timestamp: Utc::now(),
            request: TraceRequest {
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                task_type: request.task_type.clone(),
            },
            routing,
            result: None,
            failure: None,
            budget_after: BudgetSnapshot {
                remaining_usd: 0.0,
                spent_usd: 0.0,
                total_usd: 0.0,
            },
        }
    }

    /// Record a successful result.
    pub fn set_result(
        &mut self,
        response: &CompletionResponse,
        provider: &ProviderId,
        cost_usd: f64,
        quality_score: Option<f64>,
    ) {
        self.result = Some(TraceResult {
            provider: provider.clone(),
            text: response.text.clone(),
            usage: response.usage.clone(),
            actual_cost_usd: cost_usd,
            latency_ms: response.latency_ms,
            quality_score,
        });
    }

    /// Record a failure.
    pub fn set_failure(
        &mut self,
        provider: &ProviderId,
        class: FailureClass,
        error_message: String,
        retry: bool,
        retry_provider: Option<ProviderId>,
    ) {
        self.failure = Some(TraceFailure {
            provider: provider.clone(),
            class,
            error_message,
            retry_attempted: retry,
            retry_provider,
        });
    }

    /// Set budget snapshot after completion.
    pub fn set_budget(&mut self, snapshot: BudgetSnapshot) {
        self.budget_after = snapshot;
    }

    /// Serialize to JSON string (ephemeral / debug — not a durable write API).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::ProviderId;
    use crate::routing::RoutingDecision;

    #[test]
    fn test_trace_round_trip() {
        let request = CompletionRequest {
            prompt: "Hello world".into(),
            max_tokens: 100,
            temperature: Some(0.7),
            system: None,
            tools: None,
            task_type: Some("chat".into()),
        };

        let routing = RoutingDecision {
            provider: ProviderId("openai/gpt-4o-mini".into()),
            estimated_cost: 0.001,
            rationale: "cheapest".into(),
            fallback_chain: vec![ProviderId("ollama/local".into())],
        };

        let mut trace = Trace::new(&request, routing);
        trace.set_budget(BudgetSnapshot {
            remaining_usd: 9.99,
            spent_usd: 0.01,
            total_usd: 10.0,
        });

        let json = trace.to_json().unwrap();
        let parsed: Trace = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.request.max_tokens, 100);
        assert_eq!(parsed.routing.provider.0, "openai/gpt-4o-mini");
        assert_eq!(parsed.request.task_type.as_deref(), Some("chat"));
        // Joinable durable identity must stay inexpressible on Trace.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("trace_id").is_none());
        assert!(v.get("replay_metadata").is_none());
        assert!(v.get("original_trace_id").is_none());
        assert!(v.get("prompt_hash").is_none());
        assert!(v["request"].get("prompt_hash").is_none());
    }

    #[test]
    fn trace_request_has_no_joinable_hash_field() {
        // Parse only the TraceRequest struct body (avoid matching this test's own text).
        let src = include_str!("trace.rs");
        let start = src
            .find("pub struct TraceRequest {")
            .expect("TraceRequest struct");
        let rest = &src[start..];
        let end = rest.find('}').expect("TraceRequest closing brace");
        let body = &rest[..end];
        assert!(
            !body.contains("prompt_hash"),
            "TraceRequest must not declare prompt_hash (A5 / R2): {body}"
        );
    }
}
