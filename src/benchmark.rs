pub mod live;

use crate::adapter::{ProviderId, TokenUsage};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single benchmark task: prompt + expected quality criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub task_type: String,
    /// Expected output length range (for basic quality checking)
    pub expected_min_chars: usize,
    pub expected_max_chars: usize,
}

/// Results for one policy on one task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(test), allow(dead_code))] // TaskResult only instantiated inside #[cfg(test)]
pub struct TaskResult {
    pub task_id: String,
    pub policy_name: String,
    pub provider: ProviderId,
    pub usage: TokenUsage,
    pub cost_usd: f64,
    pub latency_ms: u64,
    pub quality_score: f64,
    pub success: bool,
}

/// Aggregated benchmark results for a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBenchmark {
    pub policy_name: String,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub total_cost_usd: f64,
    pub total_tokens: u32,
    pub avg_quality_score: f64,
    pub avg_latency_ms: f64,
    /// Quality per token: avg_quality_score / avg_tokens_per_task
    pub quality_per_token: f64,
    /// Successful tasks per dollar: successful_tasks / total_cost_usd
    pub tasks_per_dollar: f64,
}

impl PolicyBenchmark {
    #[cfg_attr(not(test), allow(dead_code))] // from_results called only from tests
    pub fn from_results(policy_name: &str, results: &[TaskResult]) -> Self {
        let total_tasks = results.len();
        let successful_tasks = results.iter().filter(|r| r.success).count();
        let total_cost: f64 = results.iter().map(|r| r.cost_usd).sum();
        let total_tokens: u32 = results
            .iter()
            .map(|r| r.usage.input_tokens + r.usage.output_tokens)
            .sum();
        let avg_quality: f64 = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.quality_score).sum::<f64>() / results.len() as f64
        };
        let avg_latency: f64 = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.latency_ms as f64).sum::<f64>() / results.len() as f64
        };
        let avg_tokens = if results.is_empty() {
            0.0
        } else {
            total_tokens as f64 / results.len() as f64
        };

        Self {
            policy_name: policy_name.into(),
            total_tasks,
            successful_tasks,
            total_cost_usd: total_cost,
            total_tokens,
            avg_quality_score: avg_quality,
            avg_latency_ms: avg_latency,
            quality_per_token: if avg_tokens > 0.0 {
                avg_quality / avg_tokens
            } else {
                0.0
            },
            tasks_per_dollar: if total_cost > 0.0 {
                successful_tasks as f64 / total_cost
            } else {
                0.0
            },
        }
    }
}

/// A comparison table across multiple policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub policies: Vec<PolicyBenchmark>,
}

impl BenchmarkComparison {
    /// Render as a formatted table for CLI output
    pub fn to_table(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "{:<20} | {:>14} | {:>8} | {:>12} | {:>10}\n",
            "Policy", "Quality/Token", "Tasks/$", "Avg Latency", "Total Cost"
        ));
        s.push_str(&format!(
            "{}-+-{}-+-{}-+-{}-+-{}\n",
            "-".repeat(20),
            "-".repeat(14),
            "-".repeat(8),
            "-".repeat(12),
            "-".repeat(10)
        ));

        for p in &self.policies {
            s.push_str(&format!(
                "{:<20} | {:.6} | {:>6.1} | {:>8.0}ms | ${:.4}\n",
                p.policy_name,
                p.quality_per_token,
                p.tasks_per_dollar,
                p.avg_latency_ms,
                p.total_cost_usd
            ));
        }

        s
    }
}

/// Load a benchmark task suite from a JSON file
pub fn load_task_suite(path: &Path) -> Result<Vec<BenchmarkTask>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(std::io::Error::other)
}

/// Simple quality scoring: check if output is within expected length range
/// and has reasonable content density.
/// In production, this would use a learned quality model or human eval.
pub fn score_quality(output: &str, task: &BenchmarkTask) -> f64 {
    let char_count = output.chars().count();

    // Length score: 1.0 if within range, decaying outside
    let length_score = if char_count < task.expected_min_chars {
        char_count as f64 / task.expected_min_chars as f64
    } else if char_count > task.expected_max_chars {
        let excess = char_count - task.expected_max_chars;
        let max = task.expected_max_chars as f64;
        (1.0 - (excess as f64 / max).min(1.0)).max(0.0)
    } else {
        1.0
    };

    // Content density: ratio of non-whitespace characters
    let non_ws = output.chars().filter(|c| !c.is_whitespace()).count();
    let density_score = if char_count > 0 {
        non_ws as f64 / char_count as f64
    } else {
        0.0
    };

    // Combined score: 60% length, 40% density
    0.6 * length_score + 0.4 * density_score
}

/// Default benchmark task suite (for quick testing)
pub fn default_suite() -> Vec<BenchmarkTask> {
    vec![
        BenchmarkTask {
            id: "summarize-1".into(),
            prompt: "Summarize the following text in 2-3 sentences: The quick brown fox jumps over the lazy dog. This pangram contains every letter of the English alphabet. It has been used for font preview since the typewriter era.".into(),
            max_tokens: 150,
            task_type: "summarization".into(),
            expected_min_chars: 100,
            expected_max_chars: 500,
        },
        BenchmarkTask {
            id: "code-explain-1".into(),
            prompt: "Explain what this Rust code does in 3 sentences: fn main() { let v: Vec<i32> = (1..=10).filter(|x| x % 2 == 0).collect(); println!(\"{:?}\", v); }".into(),
            max_tokens: 200,
            task_type: "code-explanation".into(),
            expected_min_chars: 150,
            expected_max_chars: 600,
        },
        BenchmarkTask {
            id: "creative-1".into(),
            prompt: "Write a haiku about Rust programming.".into(),
            max_tokens: 50,
            task_type: "creative".into(),
            expected_min_chars: 30,
            expected_max_chars: 200,
        },
        BenchmarkTask {
            id: "reasoning-1".into(),
            prompt: "If a train travels 60 mph for 2.5 hours, then 40 mph for 1.5 hours, what is the total distance? Show your work.".into(),
            max_tokens: 200,
            task_type: "reasoning".into(),
            expected_min_chars: 50,
            expected_max_chars: 400,
        },
        BenchmarkTask {
            id: "translation-1".into(),
            prompt: "Translate to French: 'The weather is nice today and I want to go for a walk in the park.'".into(),
            max_tokens: 100,
            task_type: "translation".into(),
            expected_min_chars: 40,
            expected_max_chars: 200,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_in_range() {
        let task = &default_suite()[0];
        let output = "The pangram about a fox contains every English letter. It serves as a font preview tool. Used since typewriter times.";
        let score = score_quality(output, task);
        assert!(score > 0.7); // good length and density
    }

    #[test]
    fn test_quality_score_too_short() {
        let task = &default_suite()[0];
        let output = "";
        let score = score_quality(output, task);
        assert!(score < 0.4); // length penalty pulls it down below ideal
    }

    #[test]
    fn test_policy_benchmark_aggregation() {
        let results = vec![
            TaskResult {
                task_id: "t1".into(),
                policy_name: "cheapest-first".into(),
                provider: ProviderId("ollama/local".into()),
                usage: TokenUsage {
                    input_tokens: 50,
                    output_tokens: 100,
                },
                cost_usd: 0.0,
                latency_ms: 2000,
                quality_score: 0.7,
                success: true,
            },
            TaskResult {
                task_id: "t2".into(),
                policy_name: "cheapest-first".into(),
                provider: ProviderId("ollama/local".into()),
                usage: TokenUsage {
                    input_tokens: 80,
                    output_tokens: 150,
                },
                cost_usd: 0.0,
                latency_ms: 1800,
                quality_score: 0.8,
                success: true,
            },
        ];

        let bench = PolicyBenchmark::from_results("cheapest-first", &results);
        assert_eq!(bench.successful_tasks, 2);
        assert_eq!(bench.total_cost_usd, 0.0);
        assert!((bench.avg_quality_score - 0.75).abs() < 0.01);
        // tasks_per_dollar is infinite when cost is 0, but we guard against division by zero
        assert_eq!(bench.tasks_per_dollar, 0.0); // cost is 0 � 0.0 (guarded)
    }
}
