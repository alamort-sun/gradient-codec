use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::adapter::{
    AdapterRegistry, anthropic::AnthropicAdapter,
    ollama::OllamaAdapter, openai::OpenAiAdapter,
    CompletionRequest, ProviderId,
};
use crate::budget::{BudgetConfig, BudgetState};
use crate::critique::{DefaultCritic, FailureCritic, FailureHistory};
use crate::routing::{PolicyRegistry, RoutingPolicy, RoutingDecision};
use crate::trace::Trace;
use crate::benchmark::{self, BenchmarkComparison};

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
    // Build adapter registry
    let mut registry = AdapterRegistry::new();

    // Register adapters from env vars (skip if key not set)
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        registry.register(Box::new(OpenAiAdapter::new(key, None)));
    }
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        registry.register(Box::new(AnthropicAdapter::new(key, None)));
    }
    // Ollama is local — no key needed
    registry.register(Box::new(OllamaAdapter::new(None, None)));

    if registry.list().is_empty() {
        eprintln!("Error: No adapters registered. Set OPENAI_API_KEY and/or ANTHROPIC_API_KEY.");
        eprintln!("Ollama adapter is always available if Ollama is running locally.");
        std::process::exit(1);
    }

    // Build budget
    let config = BudgetConfig {
        total_usd: budget_usd,
        per_request_cap_usd: budget_usd,
        safety_margin: 1.2,
    };
    let budget = BudgetState::new(config);

    // Build failure history
    let history = FailureHistory::new();

    // Build policy registry
    let policies = PolicyRegistry::with_defaults();

    // Build request
    let request = CompletionRequest {
        prompt: prompt.clone(),
        max_tokens: 1000,
        temperature: Some(0.7),
        system: None,
        tools: None,
        task_type: None,
    };

    let providers = registry.list_infos();

    // Route: either forced provider or policy-selected
    let decision: RoutingDecision = if let Some(ref provider_name) = forced_provider {
        let pid = ProviderId(provider_name.clone());
        // Verify provider is registered
        if registry.get(&pid).is_none() {
            eprintln!("Error: Forced provider '{}' is not registered.", provider_name);
            std::process::exit(1);
        }
        let adapter = registry.get(&pid).unwrap();
        let estimated_cost = adapter.estimate_cost(&request);
        RoutingDecision {
            provider: pid,
            estimated_cost,
            rationale: "forced provider (user override)".into(),
            fallback_chain: providers.iter()
                .filter(|p| &p.id != &ProviderId(provider_name.clone()))
                .map(|p| p.id.clone())
                .collect(),
        }
    } else {
        let policy = policies.get(&policy_name)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!("Unknown policy: {}. Use 'gradient-codec policies' to list.", policy_name).into()
            })?;
        policy.select(&request, &budget, &history, &providers)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                "No provider available within budget. Try increasing --budget.".into()
            })?
    };

    // Create trace
    let mut trace = Trace::new(&request, decision.clone());

    // Get adapter and estimate cost
    let adapter = registry.get(&decision.provider)
        .ok_or_else(|| -> Box<dyn std::error::Error> {
            format!("Provider {} not registered", decision.provider).into()
        })?;

    let estimated_cost = adapter.estimate_cost(&request);

    // Budget check (predictive)
    if !budget.can_afford(estimated_cost) {
        eprintln!(
            "Budget exhausted: remaining ${:.4}, estimated cost ${:.4}",
            budget.remaining_usd(), estimated_cost
        );
        std::process::exit(1);
    }

    println!("Routing: {} → {} ({})", policy_name, decision.provider, decision.rationale);
    println!("Estimated cost: ${:.6}", estimated_cost);
    println!("Budget remaining: ${:.4}", budget.remaining_usd());
    println!("---");

    // Execute
    match adapter.complete(&request).await {
        Ok(response) => {
            let actual_cost = response.usage.cost_usd(adapter.info());
            budget.reconcile(estimated_cost, actual_cost)
                .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;

            trace.set_result(&response, &decision.provider, actual_cost, None);
            trace.set_budget(budget.snapshot());

            println!("Response:");
            println!("{}", response.text);
            println!("---");
            println!("Tokens: {} in / {} out", response.usage.input_tokens, response.usage.output_tokens);
            println!("Cost: ${:.6}", actual_cost);
            println!("Latency: {}ms", response.latency_ms);
            println!("Budget remaining: ${:.4}", budget.remaining_usd());
        }
        Err(e) => {
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

            if let Some(retry_id) = retry_provider {
                eprintln!("Retrying with {}...", retry_id);
                if let Some(retry_adapter) = registry.get(&retry_id) {
                    match retry_adapter.complete(&request).await {
                        Ok(response) => {
                            let actual_cost = response.usage.cost_usd(retry_adapter.info());
                            println!("Retry succeeded with {}", retry_id);
                            println!("Response:");
                            println!("{}", response.text);
                            println!("Cost: ${:.6}", actual_cost);
                        }
                        Err(retry_err) => {
                            eprintln!("Retry also failed: {}", retry_err);
                        }
                    }
                }
            }
        }
    }

    // Write trace
    let trace_dir = std::env::var("GC_TRACE_DIR")
        .unwrap_or_else(|_| "./traces".into());
    let trace_path = trace.write_to_file(std::path::Path::new(&trace_dir))?;
    eprintln!("Trace written to: {}", trace_path.display());

    Ok(())
}

pub async fn replay_command(trace_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let original = crate::trace::load_trace(&trace_path)?;

    println!("Loaded trace: {}", original.trace_id);
    println!("Original routing: {} → {}", original.routing.rationale, original.routing.provider);
    println!("Original cost: ${:.6}",
        original.result.as_ref().map(|r| r.actual_cost_usd).unwrap_or(0.0));

    // In a full implementation, we would:
    // 1. Reconstruct the request from the prompt hash + stored parameters
    // 2. Re-run the routing policy with the same budget state and failure history
    // 3. Compare the new routing decision and result to the original
    // 4. Output the ReplayResult

    println!("\nReplay not yet fully implemented in v0.1 scaffold.");
    println!("The trace format supports replay — the replay engine is the next milestone.");

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

    println!("Running benchmark with {} tasks against policies: {}\n", tasks.len(), policy_names);

    let policies = PolicyRegistry::with_defaults();
    let names: Vec<&str> = policy_names.split(',').map(|s| s.trim()).collect();

    // In a full implementation, each policy would actually execute every task
    // through the orchestration engine. For the scaffold, we show the structure.
    for name in &names {
        if policies.get(name).is_none() {
            eprintln!("Unknown policy: {}", name);
            continue;
        }
        println!("Running policy: {} ({} tasks)...", name, tasks.len());
        // Actual execution would go here — for now, placeholder
        eprintln!("  (execution requires API keys + live providers)");
    }

    // Show the expected output format
    println!("\nExpected output format:\n");
    let example = BenchmarkComparison {
        policies: vec![
            benchmark::PolicyBenchmark {
                policy_name: "cheapest-first".into(),
                total_tasks: 5,
                successful_tasks: 4,
                total_cost_usd: 0.042,
                total_tokens: 3200,
                avg_quality_score: 0.72,
                avg_latency_ms: 1450.0,
                quality_per_token: 0.031,
                tasks_per_dollar: 8.2,
            },
            benchmark::PolicyBenchmark {
                policy_name: "quality-first".into(),
                total_tasks: 5,
                successful_tasks: 5,
                total_cost_usd: 0.078,
                total_tokens: 3800,
                avg_quality_score: 0.89,
                avg_latency_ms: 2100.0,
                quality_per_token: 0.048,
                tasks_per_dollar: 5.1,
            },
            benchmark::PolicyBenchmark {
                policy_name: "budget-aware".into(),
                total_tasks: 5,
                successful_tasks: 5,
                total_cost_usd: 0.051,
                total_tokens: 3500,
                avg_quality_score: 0.85,
                avg_latency_ms: 1100.0,
                quality_per_token: 0.044,
                tasks_per_dollar: 7.8,
            },
        ],
    };

    println!("{}", example.to_table());

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
