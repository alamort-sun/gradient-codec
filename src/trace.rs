use crate::adapter::{CompletionRequest, CompletionResponse, ProviderId, TokenUsage};
use crate::budget::BudgetSnapshot;
use crate::critique::FailureClass;
use crate::routing::RoutingDecision;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// A complete trace of a single orchestration step.
/// This is Fantasia's domain: logging torsion — every decision, every call,
/// every token, every failure — in a format that can be deterministically replayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
    pub request: TraceRequest,
    pub routing: RoutingDecision,
    pub result: Option<TraceResult>,
    pub failure: Option<TraceFailure>,
    pub budget_after: BudgetSnapshot,
    pub replay_metadata: Option<ReplayMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRequest {
    #[serde(skip_serializing)]
    pub prompt: Option<String>,
    pub prompt_hash: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    pub original_trace_id: String,
    pub replay_reason: String,
    pub routing_decision_changed: bool,
}

impl Trace {
    /// Create a new trace from a request and routing decision
    pub fn new(request: &CompletionRequest, routing: RoutingDecision) -> Self {
        let prompt_hash = hash_prompt(&request.prompt);
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request: TraceRequest {
                prompt_hash,
                prompt: Some(request.prompt.clone()),
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
            replay_metadata: None,
        }
    }

    /// Record a successful result
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

    /// Record a failure
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

    /// Set budget snapshot after completion
    pub fn set_budget(&mut self, snapshot: BudgetSnapshot) {
        self.budget_after = snapshot;
    }

    /// Mark this trace as a replay of another
    pub fn mark_as_replay(&mut self, original_id: String, reason: String, changed: bool) {
        self.replay_metadata = Some(ReplayMetadata {
            original_trace_id: original_id,
            replay_reason: reason,
            routing_decision_changed: changed,
        });
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Write trace to a file
    pub fn write_to_file(&self, dir: &Path) -> Result<std::path::PathBuf, std::io::Error> {
        let filename = format!("{}.json", self.timestamp.format("%Y-%m-%dT%H-%M-%S-%fZ"));
        let path = dir.join(&filename);
        std::fs::write(&path, self.to_json().map_err(std::io::Error::other)?)?;
        Ok(path)
    }
}

/// Deserialize a trace from a JSON file — for replay
pub fn load_trace(path: &Path) -> Result<Trace, TraceError> {
    let content = std::fs::read_to_string(path).map_err(|e| TraceError::Io(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| TraceError::Parse(e.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("trace file error: {0}")]
    Io(String),
    #[error("trace parse error: {0}")]
    Parse(String),
}

/// Hash a prompt for trace deduplication and replay matching
fn hash_prompt(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

/// Trace replay: re-execute the routing decision against the same input
/// and compare the result. This is the 720° closed loop.
pub struct ReplayResult {
    pub original: Trace,
    pub replayed: Trace,
    pub routing_changed: bool,
    pub cost_delta: f64,
    pub quality_delta: Option<f64>,
}

impl ReplayResult {
    /// Compare original and replayed traces
    pub fn compare(original: Trace, replayed: Trace) -> Self {
        let routing_changed = original.routing.provider != replayed.routing.provider;
        let cost_delta = replayed
            .result
            .as_ref()
            .map(|r| r.actual_cost_usd)
            .unwrap_or(0.0)
            - original
                .result
                .as_ref()
                .map(|r| r.actual_cost_usd)
                .unwrap_or(0.0);
        let quality_delta = match (&original.result, &replayed.result) {
            (Some(o), Some(r)) if o.quality_score.is_some() && r.quality_score.is_some() => {
                Some(r.quality_score.unwrap() - o.quality_score.unwrap())
            }
            _ => None,
        };

        Self {
            original,
            replayed,
            routing_changed,
            cost_delta,
            quality_delta,
        }
    }

    /// Summary for CLI output
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str("Replay vs Original:\n");
        s.push_str(&format!(
            "  Routing changed: {}\n",
            if self.routing_changed { "YES" } else { "no" }
        ));
        s.push_str(&format!("  Cost delta: ${:.6}\n", self.cost_delta));
        if let Some(qd) = self.quality_delta {
            s.push_str(&format!("  Quality delta: {:+.4}\n", qd));
        }
        s
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

        assert_eq!(parsed.trace_id, trace.trace_id);
        assert_eq!(parsed.request.max_tokens, 100);
        assert_eq!(parsed.routing.provider.0, "openai/gpt-4o-mini");
    }

    #[test]
    fn test_prompt_hash_deterministic() {
        let h1 = hash_prompt("Hello");
        let h2 = hash_prompt("Hello");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("sha256:"));
    }
}
