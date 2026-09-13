# Gradient Codec

> A Rust-based budget-aware multi-model orchestration runtime with reproducible traces and cross-model failure critique.

## Problem

Existing LLM routers (LiteLLM, rust-genai, inference-gateway) solve provider routing: *which model handles this request*. None of them solve the harder problem: *which model handles this request given a shrinking budget, a quality bar, and a failure history — with evidence you can replay.*

Teams running multi-model workflows in production face four problems no current tool addresses together:

1. **Budget exhaustion is reactive, not predictive.** Routers track cost after the fact. They don't refuse or reroute when remaining budget can't cover the likely cost of a request.
2. **Traces are for debugging, not replay.** When a multi-model pipeline fails, you get logs. You don't get a deterministic replay path that reproduces the exact routing decision, model call, and response.
3. **Failure handling is single-hop.** If model A fails, routers fall back to model B. They don't critique *why* A failed and route the retry to the model best suited to recover from that specific failure class.
4. **No quality-per-token benchmark.** Routers measure latency and cost. They don't measure quality outcomes per token spent, so you can't compare routing policies on value, only on price.

## What Gradient Codec Does

Gradient Codec is a local-first Rust runtime that:

- **Routes tasks among LLM providers** according to quality, latency, token cost, privacy constraints, and failure history
- **Enforces budget ceilings** with predictive cost estimation before dispatch, not after
- **Emits reproducible traces**: every routing decision, model call, token count, and response is logged in a structured format that can be deterministically replayed
- **Routes failures with cross-model critique**: when a model fails, the failure is classified (timeout, content policy, rate limit, quality degradation, malformed output) and the retry is routed to the model best suited to recover from that failure class
- **Benchmarks routing policies**: quality-per-token and successful-task-per-dollar metrics that let you compare policies on value, not just price

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    CLI / API Layer                     │
│         (gradient-codec CLI + HTTP server)            │
├──────────────────────────────────────────────────────┤
│              Orchestration Engine                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │
│  │ Budget   │  │ Routing  │  │ Failure Critique │    │
│  │ Manager  │  │ Policy   │  │ Classifier        │    │
│  └──────────┘  └──────────┘  └──────────────────┘    │
├──────────────────────────────────────────────────────┤
│              Provider Adapters                         │
│  OpenAI · Anthropic · Gemini · DeepSeek · Groq ·     │
│  Ollama · Cohere · vLLM · Custom OpenAI-compatible    │
├──────────────────────────────────────────────────────┤
│              Trace + Replay Layer                      │
│  (Structured JSON traces · Deterministic replay)      │
├──────────────────────────────────────────────────────┤
│              Benchmark Suite                           │
│  (Quality-per-token · Successful-task-per-dollar)     │
└──────────────────────────────────────────────────────┘
```

## v0.1 Scope

- [x] Routing policy interface (provider-agnostic)
- [x] Token/cost ledger with budget ceilings
- [x] Predictive cost estimation before dispatch
- [x] JSON trace output + deterministic replay
- [x] One benchmark suite (quality-per-token, successful-task-per-dollar)
- [x] CLI + minimal HTTP API layer
- [x] Adapters: OpenAI, Anthropic, Ollama (local), one OpenAI-compatible custom

## Quick Start

```bash
# Install
cargo install gradient-codec

# Route a single completion with a budget ceiling
gradient-codec run --prompt "Summarize this article" --budget 0.05

# Run with a specific routing policy
gradient-codec run --prompt "Write a haiku" --policy cheapest-first --budget 0.02

# Replay a previous trace
gradient-codec replay --trace ./traces/2026-09-13T19:30:00Z.json

# Benchmark two routing policies against a task suite
gradient-codec benchmark --policies cheapest-first,quality-first --tasks ./benchmark-suite/
```

## Routing Policy Interface

```rust
pub trait RoutingPolicy: Send + Sync {
    fn select(
        &mut self,
        request: &Request,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> RoutingDecision;

    fn name(&self) -> &str;
}

pub struct RoutingDecision {
    pub provider: ProviderId,
    pub estimated_cost: TokenCost,
    pub rationale: String,
    pub fallback_chain: Vec<ProviderId>,
}
```

## Failure Critique Classifier

```rust
pub enum FailureClass {
    Timeout,
    RateLimit,
    ContentPolicy,
    MalformedOutput,
    QualityDegradation { expected_quality: f64, observed: f64 },
    ProviderError { status_code: u16 },
}

pub trait FailureCritic: Send + Sync {
    fn classify(&self, failure: &Failure) -> FailureClass;
    fn recommend_retry(&self, class: &FailureClass, providers: &[ProviderInfo]) -> Option<ProviderId>;
}
```

## Trace Format

Every routing decision and model call emits a structured trace:

```json
{
  "trace_id": "01J8T24PJDXX69RM7XV24SQT11",
  "timestamp": "2026-09-13T19:30:00Z",
  "request": {
    "prompt_hash": "sha256:abc123...",
    "max_tokens": 1024,
    "task_type": "summarization"
  },
  "routing": {
    "policy": "budget-aware-quality",
    "selected_provider": "anthropic/claude-sonnet",
    "estimated_cost": { "input_tokens": 850, "output_tokens": 300, "usd": 0.008 },
    "rationale": "within budget, highest quality-per-token for summarization",
    "fallback_chain": ["openai/gpt-4o-mini", "deepseek/deepseek-chat"]
  },
  "result": {
    "provider": "anthropic/claude-sonnet",
    "actual_cost": { "input_tokens": 852, "output_tokens": 287, "usd": 0.0078 },
    "latency_ms": 1234,
    "quality_score": 0.92
  },
  "failure": null,
  "budget_after": { "remaining_usd": 0.042, "spent_usd": 0.008 }
}
```

Traces are replayable: `gradient-codec replay` re-executes the routing decision against the same input, budget state, and failure history, producing a new trace for comparison.

## Benchmark Suite

The benchmark suite measures two metrics across a set of predefined tasks:

- **Quality-per-token**: a quality score (0.0-1.0) divided by tokens consumed
- **Successful-task-per-dollar**: fraction of tasks completed successfully divided by USD spent

```bash
$ gradient-codec benchmark --policies cheapest-first,quality-first,budget-aware --tasks ./suite/

Policy              | Quality/Token | Tasks/$  | Avg Latency | Total Cost
cheapest-first      | 0.031          | 8.2      | 891ms       | $0.042
quality-first       | 0.048          | 5.1      | 1452ms      | $0.078
budget-aware        | 0.044          | 7.8      | 1103ms      | $0.051
```

## Differentiation

| Feature | LiteLLM | rust-genai | Gradient Codec |
|--------|---------|-----------|----------------|
| Provider routing | ✅ | ✅ | ✅ |
| Cost tracking | ✅ | ❌ | ✅ |
| Budget ceilings | After-fact | ❌ | **Predictive** |
| Reproducible traces | Logs only | ❌ | **Deterministic replay** |
| Failure classification | Single-hop fallback | ❌ | **Cross-model critique** |
| Quality-per-token benchmarks | ❌ | ❌ | ✅ |
| Local-first (no proxy required) | ❌ (proxy) | ✅ | ✅ |
| Rust core | ✅ (Python SDK) | ✅ | ✅ |

## Roadmap

- **v0.1** (72h): CLI, routing interface, budget manager, 3 adapters, traces, one benchmark
- **v0.2**: Streaming support, more adapters (Gemini, Groq, Cohere, vLLM), failure critique classifier
- **v0.3**: Policy DSL, web dashboard, distributed tracing export (OpenTelemetry)
- **v0.4**: Automated policy tuning from benchmark results, A/B policy comparison
- **v0.5**: Privacy-constrained routing (on-prem models, data residency rules)

## License

MIT or Apache-2.0 (dual-licensed, contributor-friendly)

## Why This Exists

Multi-model orchestration is not routing. It is deciding *which model to trust with a shrinking budget, a quality bar, and a memory of what went wrong last time* — and proving you made the right call with evidence you can replay.

Routers exist. Orchestration runtimes with budget-aware routing, reproducible traces, deterministic replay, and cross-model failure critique do not.

That gap is Gradient Codec.
