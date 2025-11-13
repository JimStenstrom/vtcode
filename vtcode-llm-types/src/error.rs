/// LLM error types
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Other error: {0}")]
    Other(String),
}

pub type LLMResult<T> = Result<T, LLMError>;

impl From<anyhow::Error> for LLMError {
    fn from(err: anyhow::Error) -> Self {
        LLMError::Other(err.to_string())
    }
}
