//! Standard Operating Procedures (SOP) System
//!
//! This module provides functionality for loading, indexing, and retrieving
//! Standard Operating Procedures (SOPs) to guide LLM behavior on common workflows.
//!
//! ## Overview
//!
//! SOPs are markdown documents stored in configured directories that contain
//! procedural guidance for the LLM. They are indexed using RAG (Retrieval-Augmented
//! Generation) and can be semantically retrieved based on the current context.
//!
//! ## Architecture
//!
//! - **Loading**: SOPs are loaded from directories specified in config
//! - **Indexing**: Documents are chunked and embedded into a vector database
//! - **Retrieval**: Semantic search retrieves relevant SOPs based on context
//!
//! ## Example
//!
//! ```no_run
//! use vtcode_core::sop::SopManager;
//! use vtcode_config::SopConfig;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SopConfig::default();
//!     let manager = SopManager::new(config).await.unwrap();
//!
//!     // Retrieve relevant SOPs
//!     let sops = manager.get_relevant_sops("how to edit files", 3).await.unwrap();
//!     for sop in sops {
//!         println!("{}", sop);
//!     }
//! }
//! ```

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// RAG and vector database imports
use vtcode_rag::{
    load_sops_from_dir, Embedder, IndexingPipeline, QueryPipeline, RetrievedChunk, SemanticChunker,
};
use vtcode_vectordb::{InMemoryVectorDb, VectorDb};

// Config import
use vtcode_config::SopConfig;

/// Mock embedder for testing (uses deterministic vectors)
///
/// In production, this would be replaced with a real embedding model
/// (e.g., sentence-transformers, OpenAI embeddings, etc.)
pub struct MockEmbedder {
    dimension: usize,
}

impl MockEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl Embedder for MockEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Simple deterministic embedding based on text length and characters
        // This is just for testing - production should use real embeddings
        let mut vec = vec![0.0; self.dimension];
        let bytes = text.as_bytes();

        for (i, byte) in bytes.iter().enumerate() {
            let idx = i % self.dimension;
            vec[idx] += (*byte as f32) / 255.0;
        }

        // Normalize
        let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            vec.iter_mut().for_each(|x| *x /= magnitude);
        }

        Ok(vec)
    }

    async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimension
    }
}

/// Manages Standard Operating Procedures (SOPs)
///
/// The SopManager is responsible for:
/// - Loading SOP markdown files from configured directories
/// - Indexing SOPs into a vector database for semantic search
/// - Providing retrieval functionality to get relevant SOPs
pub struct SopManager {
    config: SopConfig,
    query_pipeline: QueryPipeline,
    /// Number of SOPs loaded
    sop_count: usize,
}

impl SopManager {
    /// Create a new SOP manager and load SOPs from configured directories
    ///
    /// # Arguments
    /// * `config` - SOP configuration specifying directories to load from
    ///
    /// # Returns
    /// A new SopManager instance with SOPs loaded and indexed
    pub async fn new(config: SopConfig) -> Result<Self> {
        // Create vector database (as Arc<dyn VectorDb>)
        let vectordb: Arc<dyn VectorDb> = Arc::new(InMemoryVectorDb::new());

        // Create embedder (MockEmbedder for now, should be configurable in production)
        let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::new(384));

        // Create chunker
        let chunker = Box::new(SemanticChunker::default());

        // Create indexing pipeline
        let indexing_pipeline = IndexingPipeline::new(
            Arc::clone(&vectordb),
            Arc::clone(&embedder),
            chunker,
            "sops".to_string(),
        );

        // Create query pipeline
        let query_pipeline = QueryPipeline::new(
            Arc::clone(&vectordb),
            Arc::clone(&embedder),
            "sops".to_string(),
        );

        let mut manager = Self {
            config,
            query_pipeline,
            sop_count: 0,
        };

        // Load and index SOPs
        manager.load_and_index(indexing_pipeline).await?;

        Ok(manager)
    }

    /// Load and index all SOPs from configured directories
    async fn load_and_index(&mut self, indexing_pipeline: IndexingPipeline) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("SOP system is disabled");
            return Ok(());
        }

        let mut all_sops = Vec::new();

        // Load SOPs from each configured path
        for path_str in &self.config.paths {
            let path = PathBuf::from(path_str);

            if !path.exists() {
                tracing::warn!("SOP directory does not exist: {:?}", path);
                continue;
            }

            match load_sops_from_dir(&path) {
                Ok(sops) => {
                    tracing::info!("Loaded {} SOPs from {:?}", sops.len(), path);
                    all_sops.extend(sops);
                }
                Err(e) => {
                    tracing::warn!("Failed to load SOPs from {:?}: {}", path, e);
                }
            }
        }

        if all_sops.is_empty() {
            tracing::warn!("No SOPs loaded from any configured path");
            return Ok(());
        }

        // Index all SOPs
        self.sop_count = all_sops.len();
        indexing_pipeline.index_documents(all_sops).await
            .context("Failed to index SOPs into vector database")?;

        tracing::info!("Successfully indexed {} SOPs", self.sop_count);

        Ok(())
    }

    /// Get relevant SOPs based on a query
    ///
    /// # Arguments
    /// * `query` - The search query (e.g., "how to edit files")
    /// * `top_k` - Number of top results to return
    ///
    /// # Returns
    /// A vector of relevant SOP content chunks
    pub async fn get_relevant_sops(&self, query: &str, top_k: usize) -> Result<Vec<String>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Use QueryPipeline to retrieve chunks
        let chunks = self.query_pipeline.retrieve(query, top_k, None).await
            .context("Failed to retrieve SOPs")?;

        // Extract content from chunks
        let sops = chunks.into_iter().map(|chunk| chunk.content).collect();

        Ok(sops)
    }

    /// Get relevant SOPs with metadata
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `top_k` - Number of top results to return
    ///
    /// # Returns
    /// A vector of RetrievedChunk containing content and metadata
    pub async fn get_relevant_sops_with_metadata(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Use QueryPipeline to retrieve chunks with metadata
        let chunks = self.query_pipeline.retrieve(query, top_k, None).await
            .context("Failed to retrieve SOPs")?;

        Ok(chunks)
    }

    /// Get statistics about loaded SOPs
    pub fn stats(&self) -> SopStats {
        SopStats {
            enabled: self.config.enabled,
            total_sops: self.sop_count,
            configured_paths: self.config.paths.clone(),
        }
    }
}

/// Statistics about loaded SOPs
#[derive(Debug, Clone)]
pub struct SopStats {
    pub enabled: bool,
    pub total_sops: usize,
    pub configured_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    async fn create_test_sop_dir() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // Create a test SOP
        let sop_content = r#"---
type: standard-operating-procedure
id: test-sop
---

# Test SOP

This is a test SOP for unit testing.

## Usage

Use this SOP when testing the SOP manager.
"#;

        fs::write(temp_dir.path().join("test.md"), sop_content)?;

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_creates_sop_manager() {
        let temp_dir = create_test_sop_dir().await.unwrap();

        let config = SopConfig {
            enabled: true,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = SopManager::new(config).await.unwrap();
        let stats = manager.stats();

        assert!(stats.enabled);
        assert_eq!(stats.total_sops, 1);
    }

    #[tokio::test]
    async fn test_retrieves_relevant_sops() {
        let temp_dir = create_test_sop_dir().await.unwrap();

        let config = SopConfig {
            enabled: true,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = SopManager::new(config).await.unwrap();
        let results = manager.get_relevant_sops("testing", 3).await.unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_disabled_manager_returns_empty() {
        let temp_dir = create_test_sop_dir().await.unwrap();

        let config = SopConfig {
            enabled: false,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = SopManager::new(config).await.unwrap();
        let results = manager.get_relevant_sops("testing", 3).await.unwrap();

        assert!(results.is_empty());
    }
}
