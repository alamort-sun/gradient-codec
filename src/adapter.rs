use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::errors::{GcError, GcResult};

/// A provider identifier: e.g. "anthropic/claude-sonnet", "openai/gpt-4o-mini"
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Static info about a provider, used by routing policies for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub display_name: String,
    /// USD per 1M input tokens
    pub price_per_million_input: f64,
    /// USD per 1M output tokens
    pub price_per_million_output: f64,
    /// Max output tokens per request
    pub max_output_tokens: u32,
    /// Average latency in ms (rolling estimate)
    pub avg_latency_ms: u64,
    /// Supports streaming
    pub supports_streaming: bool,
    /// Supports tool/function calling
    pub supports_tools: bool,
    /// Is this a local/on-prem provider? (privacy routing)
    pub is_local: bool,
}

/// A completion request — provider-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: Option<f64>,
    pub system: Option<String>,
    pub tools: Option<Vec<ToolSpec>>,
    pub task_type: Option<String>,
}

/// A completion response from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub usage: TokenUsage,
    pub latency_ms: u64,
    pub raw_provider_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl TokenUsage {
    /// Calculate cost in USD given provider pricing
    pub fn cost_usd(&self, provider: &ProviderInfo) -> f64 {
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * provider.price_per_million_input;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * provider.price_per_million_output;
        input_cost + output_cost
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// The adapter trait — each provider implements this.
/// Adapters handle: authentication, request transformation, API call, response parsing, error mapping.
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn info(&self) -> &ProviderInfo;

    async fn complete(&self, request: &CompletionRequest) -> GcResult<CompletionResponse>;

    /// Estimate token count for a prompt before dispatch.
    /// Default: rough heuristic (4 chars ≈ 1 token). Override for provider-specific tokenizers.
    fn estimate_input_tokens(&self, request: &CompletionRequest) -> u32 {
        let char_count = request.prompt.chars().count()
            + request.system.as_ref().map(|s| s.chars().count()).unwrap_or(0);
        ((char_count as f64) / 4.0).ceil() as u32
    }

    /// Predictive cost estimation for a request
    fn estimate_cost(&self, request: &CompletionRequest) -> f64 {
        let input_tokens = self.estimate_input_tokens(request);
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.info().price_per_million_input;
        let output_cost = (request.max_tokens as f64 / 1_000_000.0) * self.info().price_per_million_output;
        input_cost + output_cost
    }
}

/// Registry of all available adapters
pub struct AdapterRegistry {
    adapters: HashMap<ProviderId, Box<dyn ProviderAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn ProviderAdapter>) {
        let id = adapter.info().id.clone();
        self.adapters.insert(id, adapter);
    }

    pub fn get(&self, id: &ProviderId) -> Option<&dyn ProviderAdapter> {
        self.adapters.get(id).map(|b| b.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn ProviderAdapter> {
        self.adapters.values().map(|b| b.as_ref()).collect()
    }

    pub fn list_infos(&self) -> Vec<ProviderInfo> {
        self.adapters.values().map(|b| b.info().clone()).collect()
    }
}

// ─── OpenAI Adapter ───────────────────────────────────────────

pub mod openai {
    use super::*;
    use reqwest::Client;

    pub struct OpenAiAdapter {
        info: ProviderInfo,
        client: Client,
        api_key: String,
        base_url: String,
    }

    impl OpenAiAdapter {
        pub fn new(api_key: String, base_url: Option<String>) -> Self {
            Self {
                info: ProviderInfo {
                    id: ProviderId("openai/gpt-4o-mini".into()),
                    display_name: "OpenAI GPT-4o mini".into(),
                    price_per_million_input: 0.15,
                    price_per_million_output: 0.60,
                    max_output_tokens: 16384,
                    avg_latency_ms: 800,
                    supports_streaming: true,
                    supports_tools: true,
                    is_local: false,
                },
                client: Client::new(),
                api_key,
                base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".into()),
            }
        }
    }

    #[async_trait]
    impl ProviderAdapter for OpenAiAdapter {
        fn info(&self) -> &ProviderInfo {
            &self.info
        }

        async fn complete(&self, request: &CompletionRequest) -> GcResult<CompletionResponse> {
            let body = serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": build_messages(request),
                "max_tokens": request.max_tokens,
                "temperature": request.temperature.unwrap_or(1.0),
            });

            let start = std::time::Instant::now();
            let resp = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| GcError::Adapter(format!("OpenAI request failed: {e}")))?;

            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(GcError::ProviderError {
                    provider: "openai".into(),
                    message: text,
                    status_code: status.as_u16(),
                });
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| GcError::Adapter(format!("OpenAI parse failed: {e}")))?;

            let text = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let usage = TokenUsage {
                input_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            };

            Ok(CompletionResponse {
                text,
                usage,
                latency_ms: start.elapsed().as_millis() as u64,
                raw_provider_response: Some(json),
            })
        }
    }

    fn build_messages(request: &CompletionRequest) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();
        if let Some(system) = &request.system {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }
        messages.push(serde_json::json!({"role": "user", "content": &request.prompt}));
        messages
    }
}

// ─── Anthropic Adapter ───────────────────────────────────────

pub mod anthropic {
    use super::*;
    use reqwest::Client;

    pub struct AnthropicAdapter {
        info: ProviderInfo,
        client: Client,
        api_key: String,
        base_url: String,
    }

    impl AnthropicAdapter {
        pub fn new(api_key: String, base_url: Option<String>) -> Self {
            Self {
                info: ProviderInfo {
                    id: ProviderId("anthropic/claude-sonnet".into()),
                    display_name: "Anthropic Claude Sonnet".into(),
                    price_per_million_input: 3.00,
                    price_per_million_output: 15.00,
                    max_output_tokens: 8192,
                    avg_latency_ms: 1200,
                    supports_streaming: true,
                    supports_tools: true,
                    is_local: false,
                },
                client: Client::new(),
                api_key,
                base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".into()),
            }
        }
    }

    #[async_trait]
    impl ProviderAdapter for AnthropicAdapter {
        fn info(&self) -> &ProviderInfo {
            &self.info
        }

        async fn complete(&self, request: &CompletionRequest) -> GcResult<CompletionResponse> {
            let mut body = serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": request.max_tokens,
                "messages": [{"role": "user", "content": &request.prompt}],
            });
            if let Some(system) = &request.system {
                body["system"] = serde_json::Value::String(system.clone());
            }
            if let Some(temp) = request.temperature {
                body["temperature"] = serde_json::json!(temp);
            }

            let start = std::time::Instant::now();
            let resp = self
                .client
                .post(format!("{}/v1/messages", self.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(|e| GcError::Adapter(format!("Anthropic request failed: {e}")))?;

            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(GcError::ProviderError {
                    provider: "anthropic".into(),
                    message: text,
                    status_code: status.as_u16(),
                });
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| GcError::Adapter(format!("Anthropic parse failed: {e}")))?;

            let text = json["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let usage = TokenUsage {
                input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            };

            Ok(CompletionResponse {
                text,
                usage,
                latency_ms: start.elapsed().as_millis() as u64,
                raw_provider_response: Some(json),
            })
        }
    }
}

// ─── Ollama (local) Adapter ───────────────────────────────────

pub mod ollama {
    use super::*;
    use reqwest::Client;

    pub struct OllamaAdapter {
        info: ProviderInfo,
        client: Client,
        base_url: String,
        model: String,
    }

    impl OllamaAdapter {
        pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
            Self {
                info: ProviderInfo {
                    id: ProviderId("ollama/local".into()),
                    display_name: "Ollama (local)".into(),
                    price_per_million_input: 0.0, // local = free
                    price_per_million_output: 0.0,
                    max_output_tokens: 4096,
                    avg_latency_ms: 2000, // local models are slower but free
                    supports_streaming: true,
                    supports_tools: false,
                    is_local: true,
                },
                client: Client::new(),
                base_url: base_url.unwrap_or_else(|| "http://localhost:11434".into()),
                model: model.unwrap_or_else(|| "llama3.2".into()),
            }
        }
    }

    #[async_trait]
    impl ProviderAdapter for OllamaAdapter {
        fn info(&self) -> &ProviderInfo {
            &self.info
        }

        async fn complete(&self, request: &CompletionRequest) -> GcResult<CompletionResponse> {
            let body = serde_json::json!({
                "model": &self.model,
                "messages": [{"role": "user", "content": &request.prompt}],
                "stream": false,
                "options": {
                    "num_predict": request.max_tokens,
                    "temperature": request.temperature.unwrap_or(0.8),
                }
            });

            let start = std::time::Instant::now();
            let resp = self
                .client
                .post(format!("{}/api/chat", self.base_url))
                .json(&body)
                .send()
                .await
                .map_err(|e| GcError::Adapter(format!("Ollama request failed: {e}")))?;

            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(GcError::ProviderError {
                    provider: "ollama".into(),
                    message: text,
                    status_code: status.as_u16(),
                });
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| GcError::Adapter(format!("Ollama parse failed: {e}")))?;

            let text = json["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            // Ollama returns eval_count for total tokens
            let eval_count = json["eval_count"].as_u64().unwrap_or(0) as u32;
            let prompt_eval_count = json["prompt_eval_count"].as_u64().unwrap_or(0) as u32;

            let usage = TokenUsage {
                input_tokens: prompt_eval_count,
                output_tokens: eval_count,
            };

            Ok(CompletionResponse {
                text,
                usage,
                latency_ms: start.elapsed().as_millis() as u64,
                raw_provider_response: Some(json),
            })
        }
    }
}
