use thiserror::Error;

/// Errors that can occur during vector database operations
#[derive(Error, Debug)]
pub enum VectorDbError {
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionAlreadyExists(String),

    #[error("Invalid vector dimensions: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Point not found: {0}")]
    PointNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),
}

/// Result type for vector database operations
pub type Result<T> = std::result::Result<T, VectorDbError>;
