//! Memory system configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Memory system configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    /// Enable or disable the memory system
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Number of recent turns to keep in working memory (full fidelity)
    #[serde(default = "default_working_memory_limit")]
    pub working_memory_limit: usize,

    /// Maximum number of summaries to retain
    #[serde(default = "default_summary_limit")]
    pub summary_limit: usize,

    /// Enable background summarization of old turns
    #[serde(default = "default_enable_background_summarization")]
    pub enable_background_summarization: bool,

    /// Automatically checkpoint (save) sessions periodically
    #[serde(default = "default_auto_checkpoint")]
    pub auto_checkpoint: bool,

    /// Checkpoint interval in seconds
    #[serde(default = "default_checkpoint_interval_seconds")]
    pub checkpoint_interval_seconds: u64,

    /// Directory for session log storage
    #[serde(default = "default_log_directory")]
    pub log_directory: PathBuf,
}

fn default_enabled() -> bool {
    true
}

fn default_working_memory_limit() -> usize {
    20
}

fn default_summary_limit() -> usize {
    100
}

fn default_enable_background_summarization() -> bool {
    true
}

fn default_auto_checkpoint() -> bool {
    true
}

fn default_checkpoint_interval_seconds() -> u64 {
    300
}

fn default_log_directory() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vtcode")
        .join("sessions")
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            working_memory_limit: default_working_memory_limit(),
            summary_limit: default_summary_limit(),
            enable_background_summarization: default_enable_background_summarization(),
            auto_checkpoint: default_auto_checkpoint(),
            checkpoint_interval_seconds: default_checkpoint_interval_seconds(),
            log_directory: default_log_directory(),
        }
    }
}

/// Vector database configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorDbConfig {
    /// Backend type: "memory", "qdrant", etc.
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Collection name prefix
    #[serde(default = "default_collection_prefix")]
    pub collection_prefix: String,

    /// Embedding vector dimensions
    #[serde(default = "default_embedding_dimensions")]
    pub embedding_dimensions: usize,

    /// Qdrant-specific configuration (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qdrant: Option<QdrantConfig>,
}

fn default_backend() -> String {
    "memory".to_string()
}

fn default_collection_prefix() -> String {
    "vtcode".to_string()
}

fn default_embedding_dimensions() -> usize {
    384
}

impl Default for VectorDbConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            collection_prefix: default_collection_prefix(),
            embedding_dimensions: default_embedding_dimensions(),
            qdrant: None,
        }
    }
}

/// Qdrant-specific configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QdrantConfig {
    /// Qdrant server URL
    pub url: String,

    /// Optional API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_defaults() {
        let config = MemoryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.working_memory_limit, 20);
        assert_eq!(config.summary_limit, 100);
        assert!(config.enable_background_summarization);
        assert!(config.auto_checkpoint);
        assert_eq!(config.checkpoint_interval_seconds, 300);
    }

    #[test]
    fn test_vectordb_config_defaults() {
        let config = VectorDbConfig::default();
        assert_eq!(config.backend, "memory");
        assert_eq!(config.collection_prefix, "vtcode");
        assert_eq!(config.embedding_dimensions, 384);
        assert!(config.qdrant.is_none());
    }

    #[test]
    fn test_memory_config_serde() {
        let toml_str = r#"
enabled = true
working_memory_limit = 30
summary_limit = 150
"#;
        let config: MemoryConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.working_memory_limit, 30);
        assert_eq!(config.summary_limit, 150);
    }
}
