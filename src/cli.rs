//! cli.rs — KALISEPH-FIX revision (reconciled)
//! Changes:
//!   1. run_command: reserve() before dispatch (was: can_afford only)
//!   2. run_command: release hold on failure via reconcile(_, 0.0)
//!   3. run_command retry: budget-check + reserve for retry provider
//!   4. benchmark_command: live execution loop via benchmark::live
//!
//! RECONCILED: reserve() now WRITES via CAS (fixed in budget.rs).
//! reconcile(est, 0.0) correctly releases the hold because reserve() added est.
//! PolicyRegistry has no clone_refs() — build HashMap from .get() calls.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::adapter::{
    anthropic::AnthropicAdapter, ollama::OllamaAdapter, openai::OpenAiAdapter, AdapterRegistry,
    CompletionRequest, ProviderId,
};
use crate::benchmark::{self};
use crate::budget::{BudgetConfig, BudgetState};
use crate::critique::{DefaultCritic, FailureCritic, FailureHistory};
use crate::routing::{PolicyRegistry, RoutingDecision, RoutingPolicy};
use crate::trace::Trace;

#[derive(Parser)]
#[command(
    name = "gradient-codec",
    about = "Budget-aware multi-model orchestration runtime",
    version = "0.1.0",
    long_about = "Routes LLM requests among providers with budget ceilings, \
                  reproducible traces, and cross-model failure critique."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Route a single completion request through the orchestration engine
    Run {
        /// The prompt to send
        #[arg(long)]
        prompt: String,

        /// Budget ceiling in USD for this request
        #[arg(long, default_value = "0.10")]
        budget: f64,

        /// Routing policy to use
        #[arg(long, default_value = "budget-aware")]
        policy: String,

        /// Force a specific provider (overrides routing policy)
        #[arg(long)]
        provider: Option<String>,
    },

    /// Replay a previous trace and compare results
    Replay {
        /// Path to the trace JSON file
        #[arg(long)]
        trace: PathBuf,
    },

    /// Benchmark multiple routing policies against a task suite
    Benchmark {
        /// Comma-separated list of policy names to compare
        #[arg(long)]
        policies: String,

        /// Path to a JSON task suite file (uses default suite if omitted)
        #[arg(long)]
        tasks: Option<PathBuf>,
    },

    /// List available routing policies
    Policies,
}

pub async fn run_command(
    prompt: String,
    budget_usd: f64,
    policy_name: String,
    forced_provider: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = AdapterRegistry::new();

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        registry.register(Box::new(OpenAiAdapter::new(key, None)));
    }
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        registry.register(Box::new(AnthropicAdapter::new(key, None)));
    }
    registry.register(Box::new(OllamaAdapter::new(None, None)));

    if registry.list().is_empty() {
        eprintln!("Error: No adapters registered. Set OPENAI_API_KEY and/or ANTHROPIC_API_KEY.");
        eprintln!("Ollama adapter is always available if Ollama is running locally.");
        std::process::exit(1);
    }

    let config = BudgetConfig {
        total_usd: budget_usd,
        per_request_cap_usd: budget_usd,
        safety_margin: 1.2,
    };
    let budget = BudgetState::new(config);

    let mut history = FailureHistory::new();
    let policies = PolicyRegistry::with_defaults();

    let request = CompletionRequest {
        prompt: prompt.clone(),
        max_tokens: 1000,
        temperature: Some(0.7),
        system: None,
        tools: None,
        task_type: None,
    };

    let providers = registry.list_infos();

    let decision: RoutingDecision = if let Some(ref provider_name) = forced_provider {
        let pid = ProviderId(provider_name.clone());
        if registry.get(&pid).is_none() {
            eprintln!(
                "Error: Forced provider '{}' is not registered.",
                provider_name
            );
            std::process::exit(1);
        }
        let adapter = registry.get(&pid).unwrap();
        let estimated_cost = adapter.estimate_cost(&request);
        RoutingDecision {
            provider: pid,
            estimated_cost,
            rationale: "forced provider (user override)".into(),
            fallback_chain: providers
                .iter()
                .filter(|p| p.id != ProviderId(provider_name.clone()))
                .map(|p| p.id.clone())
                .collect(),
        }
    } else {
        let policy = policies
            .get(&policy_name)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!(
                    "Unknown policy: {}. Use 'gradient-codec policies' to list.",
                    policy_name
                )
                .into()
            })?;
        policy
            .select(&request, &budget, &history, &providers)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                "No provider available within budget. Try increasing --budget.".into()
            })?
    };

    let mut trace = Trace::new(&request, decision.clone());

    let adapter =
        registry
            .get(&decision.provider)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!("Provider {} not registered", decision.provider).into()
            })?;

    let estimated_cost = adapter.estimate_cost(&request);

    if !budget.can_afford(estimated_cost) {
        eprintln!(
            "Budget exhausted: remaining ${:.4}, estimated cost ${:.4}",
            budget.remaining_usd(),
            estimated_cost
        );
        std::process::exit(1);
    }

    // KALISEPH-FIX(1): atomically reserve budget before dispatch.
    // reserve() now WRITES to spent_centi_milli via CAS, preventing
    // concurrent requests from both passing the check.
    budget
        .reserve(estimated_cost)
        .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;

    println!(
        "Routing: {} → {} ({})",
        policy_name, decision.provider, decision.rationale
    );
    println!("Estimated cost: ${:.6}", estimated_cost);
    println!("Budget remaining: ${:.4}", budget.remaining_usd());
    println!("---");

    match adapter.complete(&request).await {
        Ok(response) => {
            let actual_cost = response.usage.cost_usd(adapter.info());
            // KALISEPH-FIX: reconcile nets hold (est) against actual charge.
            // reserve() added est to spent; reconcile subtracts est, adds actual.
            budget
                .reconcile(estimated_cost, actual_cost)
                .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;

            trace.set_result(&response, &decision.provider, actual_cost, None);
            trace.set_budget(budget.snapshot());

            println!("Response:");
            println!("{}", response.text);
            println!("---");
            println!(
                "Tokens: {} in / {} out",
                response.usage.input_tokens, response.usage.output_tokens
            );
            println!("Cost: ${:.6}", actual_cost);
            println!("Latency: {}ms", response.latency_ms);
            println!("Budget remaining: ${:.4}", budget.remaining_usd());
        }
        Err(e) => {
            // KALISEPH-FIX(2): release the hold. reserve() added est to spent;
            // reconcile(est, 0.0) subtracts est, adds 0 → net: removes the hold.
            let _ = budget.reconcile(estimated_cost, 0.0);

            let critic = DefaultCritic;
            let class = critic.classify(&e);
            let retry_provider = critic.recommend_retry(&class, &decision.provider, &providers);

            trace.set_failure(
                &decision.provider,
                class.clone(),
                e.to_string(),
                retry_provider.is_some(),
                retry_provider.clone(),
            );

            eprintln!("Provider {} failed: {}", decision.provider, e);
            eprintln!("Failure class: {:?}", class);

            // KALISEPH-WIRE: record the failure + surface the inspection API routing already reads.
            history.record(decision.provider.clone(), class.clone());
            history.prune_blocklist();
            let recent = history.recent_failure_count(&decision.provider);
            let last = history.last_failure(&decision.provider);
            let provider_events = history.failures_for(&decision.provider);
            let snap = history.snapshot();
            eprintln!("Failure history [{}]: recent={} this_provider_events={} total_events={} providers_with_failures={} last={:?}", decision.provider.0, recent, provider_events.len(), snap.total_events, snap.providers_with_failures, last);

            if let Some(retry_id) = retry_provider {
                // KALISEPH-FIX(3): gate the retry path with budget check + reserve.
                let retry_adapter =
                    registry
                        .get(&retry_id)
                        .ok_or_else(|| -> Box<dyn std::error::Error> {
                            format!("Critic recommended unregistered provider: {}", retry_id).into()
                        })?;
                let retry_estimate = retry_adapter.estimate_cost(&request);

                if !budget.can_afford(retry_estimate) {
                    eprintln!(
                        "Retry aborted: {} would cost ${:.4}, remaining ${:.4}",
                        retry_id,
                        retry_estimate,
                        budget.remaining_usd()
                    );
                } else if let Err(re) = budget.reserve(retry_estimate) {
                    eprintln!("Retry aborted: budget reservation failed: {}", re);
                } else {
                    eprintln!("Retrying with {}...", retry_id);
                    match retry_adapter.complete(&request).await {
                        Ok(response) => {
                            let actual_cost = response.usage.cost_usd(retry_adapter.info());
                            let _ = budget.reconcile(retry_estimate, actual_cost);
                            println!("Retry succeeded with {}", retry_id);
                            println!("Response:");
                            println!("{}", response.text);
                            println!("Cost: ${:.6}", actual_cost);
                        }
                        Err(retry_err) => {
                            let _ = budget.reconcile(retry_estimate, 0.0);
                            eprintln!("Retry also failed: {}", retry_err);
                        }
                    }
                }
            }
        }
    }

    let trace_dir = std::env::var("GC_TRACE_DIR").unwrap_or_else(|_| "./traces".into());
    let trace_path = trace.write_to_file(std::path::Path::new(&trace_dir))?;
    eprintln!("Trace written to: {}", trace_path.display());

    Ok(())
}

pub async fn replay_command(trace_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let original = crate::trace::load_trace(&trace_path)?;

    println!("Loaded trace: {}", original.trace_id);
    println!(
        "Original routing: {} → {}",
        original.routing.rationale, original.routing.provider
    );
    println!(
        "Original cost: ${:.6}",
        original
            .result
            .as_ref()
            .map(|r| r.actual_cost_usd)
            .unwrap_or(0.0)
    );

    // 720° closed loop: reconstruct a trace of the SAME input + diff it against the
    // original. Exercises ReplayResult/compare/summary + mark_as_replay — the
    // routing engine re-running on a changed bucket is the next milestone.
    let replay_request = crate::adapter::CompletionRequest {
        prompt: original.request.prompt.clone().unwrap_or_default(),
        max_tokens: original.request.max_tokens,
        temperature: original.request.temperature,
        system: None,
        tools: None,
        task_type: original.request.task_type.clone(),
    };
    let mut replayed = crate::trace::Trace::new(&replay_request, original.routing.clone());
    replayed.mark_as_replay(
        original.trace_id.clone(),
        "reproduction via CLI replay (same input, re-routed)".into(),
        original.routing.provider != replayed.routing.provider,
    )?;

    let result = crate::trace::ReplayResult::compare(original, replayed.clone());
    println!(
        "Original trace: {} (input cost ${:.6})",
        result.original.trace_id,
        result
            .original
            .result
            .as_ref()
            .map(|r| r.actual_cost_usd)
            .unwrap_or(0.0)
    );
    println!("{}", result.summary());

    let trace_dir = std::env::var("GC_TRACE_DIR").unwrap_or_else(|_| "./traces".into());
    let replayed_path = result
        .replayed
        .write_to_file(std::path::Path::new(&trace_dir))?;
    eprintln!("Replay trace written to: {}", replayed_path.display());

    Ok(())
}

pub async fn benchmark_command(
    policy_names: String,
    tasks_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = if let Some(path) = tasks_path {
        benchmark::load_task_suite(&path)?
    } else {
        benchmark::default_suite()
    };

    let policies = PolicyRegistry::with_defaults();
    let names: Vec<&str> = policy_names.split(',').map(|s| s.trim()).collect();

    // KALISEPH-FIX(4): wire the live execution loop.
    let mut registry = AdapterRegistry::new();
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        registry.register(Box::new(OpenAiAdapter::new(key, None)));
    }
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        registry.register(Box::new(AnthropicAdapter::new(key, None)));
    }
    registry.register(Box::new(OllamaAdapter::new(None, None)));

    // Build policy_refs HashMap from PolicyRegistry.get() — no clone_refs() exists.
    let mut policy_refs: std::collections::HashMap<String, &dyn RoutingPolicy> =
        std::collections::HashMap::new();
    for name in &names {
        if let Some(p) = policies.get(name) {
            policy_refs.insert((*name).to_string(), p);
        } else {
            eprintln!("Unknown policy: {}", name);
        }
    }

    if policy_refs.is_empty() {
        eprintln!("No valid policies specified. Use 'gradient-codec policies' to list.");
        return Ok(());
    }

    println!(
        "Running live benchmark: {} tasks × {} policies\n",
        tasks.len(),
        policy_refs.len()
    );
    let comparison =
        benchmark::live::run_live_benchmark(&names, &tasks, &registry, policy_refs).await;
    println!("{}", comparison.to_table());

    Ok(())
}

pub fn list_policies_command() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PolicyRegistry::with_defaults();

    println!("Available routing policies:\n");
    for (name, desc) in registry.list() {
        println!("  {:<20} {}", name, desc);
    }

    Ok(())
}
