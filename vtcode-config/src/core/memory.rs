use anyhow::{Result, ensure};
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

    /// Procedures configuration
    pub procedures: ProceduresConfig,
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
            procedures: ProceduresConfig::default(),
        }
    }
}

impl MemoryConfig {
    /// Validate memory configuration
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.working_memory_limit > 0,
            "memory.working_memory_limit must be greater than 0"
        );

        ensure!(
            self.summary_limit > 0,
            "memory.summary_limit must be greater than 0"
        );

        ensure!(
            self.checkpoint_interval_seconds > 0,
            "memory.checkpoint_interval_seconds must be greater than 0"
        );

        // Reasonable upper bounds to prevent misconfiguration
        ensure!(
            self.working_memory_limit <= 1000,
            "memory.working_memory_limit must be <= 1000 (found: {})",
            self.working_memory_limit
        );

        ensure!(
            self.summary_limit <= 10000,
            "memory.summary_limit must be <= 10000 (found: {})",
            self.summary_limit
        );

        Ok(())
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

impl VectorDbConfig {
    /// Validate vector database configuration
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.backend == "memory" || self.backend == "qdrant",
            "vectordb.backend must be 'memory' or 'qdrant' (found: '{}')",
            self.backend
        );

        ensure!(
            !self.collection_prefix.is_empty(),
            "vectordb.collection_prefix must not be empty"
        );

        ensure!(
            self.embedding_dimensions > 0,
            "vectordb.embedding_dimensions must be greater than 0"
        );

        // Common embedding dimensions: 384, 512, 768, 1024, 1536
        ensure!(
            self.embedding_dimensions <= 4096,
            "vectordb.embedding_dimensions must be <= 4096 (found: {})",
            self.embedding_dimensions
        );

        // Validate Qdrant config if present
        if let Some(ref qdrant) = self.qdrant {
            qdrant.validate()?;
        }

        Ok(())
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

impl QdrantConfig {
    /// Validate Qdrant configuration
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.url.is_empty(),
            "vectordb.qdrant.url must not be empty"
        );

        ensure!(
            self.url.starts_with("http://") || self.url.starts_with("https://"),
            "vectordb.qdrant.url must start with http:// or https:// (found: '{}')",
            self.url
        );

        Ok(())
    }
}

/// Procedures configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProceduresConfig {
    /// Enable procedures system
    pub enabled: bool,

    /// Directories to scan for procedures (relative to workspace root)
    pub paths: Vec<String>,
}

impl Default for ProceduresConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: vec![
                "docs/procedures".to_string(),      // Project-level procedures (version controlled)
                ".vtcode/procedures".to_string(),   // User-specific procedures (gitignored)
            ],
        }
    }
}

impl ProceduresConfig {
    /// Validate procedures configuration
    pub fn validate(&self) -> Result<()> {
        // Paths can be empty (SOPs are optional)
        // No validation needed currently
        Ok(())
    }
}
