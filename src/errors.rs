use thiserror::Error;

#[derive(Error, Debug)]
pub enum GcError {
    #[error("budget exhausted: remaining ${remaining:.4}, estimated cost ${estimated:.4}")]
    BudgetExhausted { remaining: f64, estimated: f64 },

    #[error("no provider available for request: {reason}")]
    NoProvider { reason: String },

    #[error("provider {provider} error: {message} (status {status_code})")]
    ProviderError {
        provider: String,
        message: String,
        status_code: u16,
    },

    #[error("provider {provider} timed out after {timeout_ms}ms")]
    ProviderTimeout { provider: String, timeout_ms: u64 },

    #[error("rate limited on {provider}: retry after {retry_after_s}s")]
    RateLimited {
        provider: String,
        retry_after_s: u64,
    },

    #[error("content policy violation on {provider}: {reason}")]
    ContentPolicy { provider: String, reason: String },

    #[error("malformed output from {provider}: {detail}")]
    MalformedOutput { provider: String, detail: String },

    #[error("trace file error: {0}")]
    TraceFile(String),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Generic(String),
}

pub type GcResult<T> = Result<T, GcError>;
