use thiserror::Error;

/// Memory system errors
#[derive(Error, Debug)]
pub enum MemoryError {
    /// Failed to serialize/deserialize memory data
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Failed to perform I/O operation
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to summarize turn
    #[error("Summarization failed: {0}")]
    SummarizationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Memory limit exceeded
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),

    /// Background task error
    #[error("Background task error: {0}")]
    BackgroundTaskError(String),
}

/// Type alias for Result with MemoryError
pub type Result<T> = std::result::Result<T, MemoryError>;
