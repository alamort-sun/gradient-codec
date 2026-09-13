//! benchmark/live.rs — KALISEPH: live benchmark execution loop
//!
//! New module. Wire it in with:
//!   1. Place at `src/benchmark/live.rs`
//!   2. In `src/benchmark.rs` add: `pub mod live;`
//!   3. cli.rs `benchmark_command` calls `live::run_live_benchmark(...)`
//!
//! RECONCILED against actual repo types:
//!   - No TaskSuite wrapper; default_suite() returns Vec<BenchmarkTask>
//!   - BenchmarkTask: { id, prompt, max_tokens: u32, task_type: String, expected_min_chars, expected_max_chars }
//!   - PolicyBenchmark: { total_tasks: usize, successful_tasks: usize, total_tokens: u32, ... }
//!   - PolicyRegistry.get(name) -> Option<&dyn RoutingPolicy>

use std::collections::HashMap;

use crate::adapter::{AdapterRegistry, CompletionRequest};
use crate::budget::{BudgetConfig, BudgetState};
use crate::critique::{DefaultCritic, FailureCritic, FailureHistory};
use crate::routing::RoutingPolicy;

use super::{BenchmarkComparison, PolicyBenchmark, BenchmarkTask};

/// Live benchmark: run each policy over the task suite with fresh budget + history.
pub async fn run_live_benchmark(
    policy_names: &[&str],
    tasks: &[BenchmarkTask],
    registry: &AdapterRegistry,
    policy_refs: HashMap<String, &dyn RoutingPolicy>,
) -> BenchmarkComparison {
    let providers = registry.list_infos();

    let mut benchmarks = Vec::with_capacity(policy_names.len());

    for name in policy_names {
        let policy = match policy_refs.get(*name) {
            Some(p) => *p,
            None => {
                eprintln!("Skipping unknown policy: {}", name);
                continue;
            }
        };

        println!("── policy: {} ──", name);

        // Fresh budget + fresh failure history per policy so policies
        // don't bias each other.
        let budget = BudgetState::new(BudgetConfig {
            total_usd: 1.0,
            per_request_cap_usd: 0.25,
            safety_margin: 1.2,
        });
        let mut history = FailureHistory::new();
        let critic = DefaultCritic;

        let mut total_cost: f64 = 0.0;
        let mut total_tokens: u32 = 0;
        let mut total_latency_ms: u64 = 0;
        let mut successful: usize = 0;
        let mut quality_sum: f64 = 0.0;

        for (i, task) in tasks.iter().enumerate() {
            let request = CompletionRequest {
                prompt: task.prompt.clone(),
                max_tokens: task.max_tokens,
                temperature: None,
                system: None,
                tools: None,
                task_type: Some(task.task_type.clone()),
            };

            let decision = match policy.select(&request, &budget, &history, &providers) {
                Some(d) => d,
                None => {
                    println!("  task {}: no provider affordable — counted as failed", i + 1);
                    continue;
                }
            };
            let adapter = match registry.get(&decision.provider) {
                Some(a) => a,
                None => {
                    println!("  task {}: provider {:?} vanished", i + 1, decision.provider);
                    continue;
                }
            };

            let estimate = adapter.estimate_cost(&request);
            if !budget.can_afford(estimate) {
                println!("  task {}: over budget cap", i + 1);
                continue;
            }
            if let Err(e) = budget.reserve(estimate) {
                println!("  task {}: reserve failed: {}", i + 1, e);
                continue;
            }

            let t0 = std::time::Instant::now();
            match adapter.complete(&request).await {
                Ok(response) => {
                    let actual = response.usage.cost_usd(adapter.info());
                    let _ = budget.reconcile(estimate, actual);
                    let latency = t0.elapsed().as_millis() as u64;

                    total_cost += actual;
                    total_tokens += response.usage.input_tokens + response.usage.output_tokens;
                    total_latency_ms += latency;
                    successful += 1;

                    let quality = score_task(task, &response.text);
                    quality_sum += quality;

                    println!(
                        "  task {}: ok → {}  ${:.4}  {}ms  q={:.2}",
                        i + 1, decision.provider, actual, latency, quality
                    );
                }
                Err(e) => {
                    let _ = budget.reconcile(estimate, 0.0);
                    let class = critic.classify(&e);
                    history.record(decision.provider.clone(), class.clone());
                    println!("  task {}: FAIL ({:?}) {}", i + 1, class, e);
                }
            }
        }

        let avg_quality = if successful > 0 { quality_sum / successful as f64 } else { 0.0 };
        let avg_latency = if successful > 0 { total_latency_ms as f64 / successful as f64 } else { 0.0 };
        let avg_tokens = if successful > 0 { total_tokens as f64 / successful as f64 } else { 0.0 };

        benchmarks.push(PolicyBenchmark {
            policy_name: (*name).to_string(),
            total_tasks: tasks.len(),
            successful_tasks: successful,
            total_cost_usd: total_cost,
            total_tokens,
            avg_quality_score: avg_quality,
            avg_latency_ms: avg_latency,
            quality_per_token: if avg_tokens > 0.0 {
                avg_quality / (avg_tokens / 1000.0)
            } else { 0.0 },
            tasks_per_dollar: if total_cost > 0.0 {
                successful as f64 / total_cost
            } else { 0.0 },
        });

        println!("  ⇒ {}/{} ok, ${:.4}, q/tok {:.4}, tasks/$ {:.2}\n",
            successful, tasks.len(), total_cost,
            benchmarks.last().map(|b| b.quality_per_token).unwrap_or(0.0),
            benchmarks.last().map(|b| b.tasks_per_dollar).unwrap_or(0.0));
    }

    BenchmarkComparison { policies: benchmarks }
}

/// Quality-scoring shim. Uses the BenchmarkTask's expected_min_chars/max_chars
/// for a basic length-based quality score. Replace with a real rubric when available.
fn score_task(task: &BenchmarkTask, response_text: &str) -> f64 {
    super::score_quality(response_text, task)
}
