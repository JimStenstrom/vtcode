use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Memory system configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    /// Enable memory system
    pub enabled: bool,

    /// Maximum turns in working memory
    pub working_memory_limit: usize,

    /// Maximum summaries to retain
    pub summary_limit: usize,

    /// Enable background summarization
    pub enable_background_summarization: bool,

    /// Auto-checkpoint periodically
    pub auto_checkpoint: bool,

    /// Checkpoint interval in seconds
    pub checkpoint_interval_seconds: u64,

    /// Directory for session logs
    pub log_directory: Option<PathBuf>,

    /// Optional: specific model for summarization
    pub summarization_model: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            working_memory_limit: 20,
            summary_limit: 100,
            enable_background_summarization: true,
            auto_checkpoint: true,
            checkpoint_interval_seconds: 300,
            log_directory: None, // Will use default
            summarization_model: None,
        }
    }
}

/// VectorDB configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VectorDbConfig {
    /// Backend to use (memory, qdrant)
    pub backend: String,

    /// Collection name prefix
    pub collection_prefix: String,

    /// Embedding dimensions
    pub embedding_dimensions: usize,

    /// Qdrant-specific config (future)
    pub qdrant: Option<QdrantConfig>,
}

impl Default for VectorDbConfig {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            collection_prefix: "vtcode".to_string(),
            embedding_dimensions: 384,
            qdrant: None,
        }
    }
}

/// Qdrant configuration (for future use)
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    /// Qdrant server URL
    pub url: String,

    /// Optional API key for authentication
    pub api_key: Option<String>,
}
