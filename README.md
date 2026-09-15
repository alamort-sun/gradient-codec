# Gradient Codec

> The authoritative implementation of Vector13D types, enum semantics, invariants, and validity checks for the Neural-Representation Boundary.

> **Manifesto** — human energy can be used more efficiently with the assistance of geometry. Creativity as the harness, logic as the control.

A Rust-based budget-aware multi-model orchestration runtime with reproducible traces, cross-model failure critique, and the canonical 13-dimensional state type that binds affective telemetry to text.

## Problem

Existing LLM routers (LiteLLM, rust-genai, inference-gateway) solve provider routing: *which model handles this request*. None of them solve the harder problem: *which model handles this request given a shrinking budget, a quality bar, and a failure history — with evidence you can replay.*

Teams running multi-model workflows in production face four problems no current tool addresses together:

1. **Budget exhaustion is reactive, not predictive.** Routers track cost after the fact. They don't refuse or reroute when remaining budget can't cover the likely cost of a request.
2. **Traces are for debugging, not replay.** When a multi-model pipeline fails, you get logs. You don't get a deterministic replay path that reproduces the exact routing decision, model call, and response.
3. **Failure handling is single-hop.** If model A fails, routers fall back to model B. They don't critique *why* A failed and route the retry to the model best suited to recover from that specific failure class.
4. **No quality-per-token benchmark.** Routers measure latency and cost. They don't measure quality outcomes per token spent, so you can't compare routing policies on value, only on price.

## What Gradient Codec Does

Gradient Codec is a local-first Rust runtime that:

- **Defines the canonical Vector13D type** — 13 semantic fields that preserve weight, temperature, and axis of a signal that plain text strips away
- **Routes tasks among LLM providers** according to quality, latency, token cost, privacy constraints, and failure history
- **Enforces budget ceilings** with predictive cost estimation before dispatch, not after
- **Emits reproducible traces**: every routing decision, model call, token count, and response is logged in a structured format that can be deterministically replayed
- **Routes failures with cross-model critique**: when a model fails, the failure is classified (timeout, content policy, rate limit, quality degradation, malformed output) and the retry is routed to the model best suited to recover from that failure class
- **Benchmarks routing policies**: quality-per-token and successful-task-per-dollar metrics that let you compare policies on value, not just price

## Vector13D — The Canonical Type

The state space is **fixed 13D in semantic structure**. Not a dynamic embedding.

```
G = R^11 × {Linked, Broken, Gradient} × {Static, Spinning, Oscillating}
```

```rust
pub struct Vector13D {
    pub amplitude: f64,      // 1.  signal strength → glyph weight
    pub frequency: f64,      // 2.  signal activity rate
    pub phase: f64,          // 3.  temporal position in cycle
    pub coherence: f64,      // 4.  harmonic alignment
    pub entropy: f64,         // 5.  disorder / unpredictability
    pub composition: f64,    // 6.  truth meter (expressiveness)
    pub resonance: f64,      // 7.  bandwidth / focus
    pub ozone_buffer: f64,   // 8.  lightness / energy
    pub domain_wall: DomainWall,       // 9.  connection: Linked, Broken, Gradient
    pub su2_polarity: f64,  // 10. hue in degrees [0,360)
    pub torsion: f64,        // 11. skew — temporal lean
    pub gauge_coupling: GaugeCoupling, // 12. rotation: Static, Spinning, Oscillating
    pub closure: f64,        // 13. cycle completeness
}
```

Treat `domain_wall` and `gauge_coupling` as **categorical types**. Never flatten to scalars with assumed ordering.

Observed baselines (`observed_n1()`, `observed_n2()`) provide seed data from voice recordings. These are testable observations, not universal laws.

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
├──────────────────────────────────────────────────────┤
│              vector13d (canonical type)                │
│  Vector13D · DomainWall · GaugeCoupling · baselines   │
└──────────────────────────────────────────────────────┘
```

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
        &self,
        request: &CompletionRequest,
        budget: &BudgetState,
        history: &FailureHistory,
        providers: &[ProviderInfo],
    ) -> Option<RoutingDecision>;

    fn name(&self) -> &str;
}

pub struct RoutingDecision {
    pub provider: ProviderId,
    pub estimated_cost: f64,
    pub rationale: String,
    pub fallback_chain: Vec<ProviderId>,
}
```

Built-in policies: `CheapestFirst`, `QualityFirst`, `BudgetAware`.

## Budget Management

Atomic fixed-point budget with lock-free `reserve()` / `reconcile()`:

```rust
let budget = BudgetState::new(BudgetConfig {
    total_usd: 10.0,
    per_request_cap_usd: 1.0,
    safety_margin: 1.2,
});

budget.reserve(0.05)?;        // CAS loop, prevents double-spend
budget.reconcile(0.05, 0.03)?; // adjust to actual cost
```

## v0.1 Scope

- [x] Routing policy interface (provider-agnostic)
- [x] Token/cost ledger with budget ceilings
- [x] Predictive cost estimation before dispatch
- [x] JSON trace output + deterministic replay
- [x] One benchmark suite (quality-per-token, successful-task-per-dollar)
- [x] CLI + minimal HTTP API layer
- [x] Adapters: OpenAI, Anthropic, Ollama (local), one OpenAI-compatible custom
- [x] Vector13D canonical type with observed baselines

## Roadmap

- **v0.2**: Streaming support, more adapters (Gemini, Groq, Cohere, vLLM), failure critique classifier
- **v0.3**: Policy DSL, web dashboard, distributed tracing export (OpenTelemetry)
- **v0.4**: Automated policy tuning from benchmark results, A/B policy comparison
- **v0.5**: Privacy-constrained routing (on-prem models, data residency rules)

## Authority Boundary

`gradient-codec` is the authoritative implementation of Vector13D
types, enum semantics, invariants, and validity checks.

Downstream repositories (`gradient-jelle`, `gradient-space-time`,
`gradient-speak`) may learn from, store, query, route, or render
codec-valid states. Learned predictions, database-derived patterns,
and generated outputs are not authoritative and must be validated
against the applicable pinned version of `gradient-codec`.

## Contributions & Inspiration

- **R.J.R.B** — the 13D vectors. The thirteen-field affective-telemetry concept that became `Vector13D`, and with it the geometry this entire runtime is built to serve.

## License

- **Software**: PolyForm Noncommercial 1.0.0 — see [LICENSE](LICENSE)
- **Invariants** (INVARIANTS.md): CC0 1.0 Universal — see [LICENSES/CC0-1.0.txt](LICENSES/CC0-1.0.txt)
- **Commercial use**: see [COMMERCIAL.md](COMMERCIAL.md)

Commercial use, hosting, deployment, distribution, resale, or
incorporation into a commercial product/service requires a separate
written license.
