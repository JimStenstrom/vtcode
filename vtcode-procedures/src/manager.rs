//! Procedure Manager
//!
//! Manages loading, indexing, and retrieval of Standard Operating Procedures.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

// RAG and vector database imports
use vtcode_rag::{Embedder, IndexingPipeline, QueryPipeline, RetrievedChunk, SemanticChunker};
use vtcode_vectordb::{InMemoryVectorDb, VectorDb};

// Config import
use vtcode_config::ProceduresConfig;

use crate::loader::load_procedures_from_dir;

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

/// Manages Standard Operating Procedures
///
/// The ProcedureManager is responsible for:
/// - Loading procedure markdown files from configured directories
/// - Indexing procedures into a vector database for semantic search
/// - Providing retrieval functionality to get relevant procedures
pub struct ProcedureManager {
    config: ProceduresConfig,
    query_pipeline: QueryPipeline,
    /// Number of procedures loaded
    procedure_count: usize,
}

impl ProcedureManager {
    /// Create a new procedure manager and load procedures from configured directories
    ///
    /// # Arguments
    /// * `config` - Procedure configuration specifying directories to load from
    ///
    /// # Returns
    /// A new ProcedureManager instance with procedures loaded and indexed
    pub async fn new(config: ProceduresConfig) -> Result<Self> {
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
            "procedures".to_string(),
        );

        // Create query pipeline
        let query_pipeline = QueryPipeline::new(
            Arc::clone(&vectordb),
            Arc::clone(&embedder),
            "procedures".to_string(),
        );

        let mut manager = Self {
            config,
            query_pipeline,
            procedure_count: 0,
        };

        // Load and index procedures
        manager.load_and_index(indexing_pipeline).await?;

        Ok(manager)
    }

    /// Load and index all procedures from configured directories
    async fn load_and_index(&mut self, indexing_pipeline: IndexingPipeline) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Procedures system is disabled");
            return Ok(());
        }

        let mut all_procedures = Vec::new();

        // Load procedures from each configured path
        for path_str in &self.config.paths {
            let path = PathBuf::from(path_str);

            if !path.exists() {
                tracing::warn!("Procedure directory does not exist: {:?}", path);
                continue;
            }

            match load_procedures_from_dir(&path) {
                Ok(procedures) => {
                    tracing::info!("Loaded {} procedures from {:?}", procedures.len(), path);
                    all_procedures.extend(procedures);
                }
                Err(e) => {
                    tracing::warn!("Failed to load procedures from {:?}: {}", path, e);
                }
            }
        }

        if all_procedures.is_empty() {
            tracing::warn!("No procedures loaded from any configured path");
            return Ok(());
        }

        // Index all procedures
        self.procedure_count = all_procedures.len();
        indexing_pipeline
            .index_documents(all_procedures)
            .await
            .context("Failed to index procedures into vector database")?;

        tracing::info!("Successfully indexed {} procedures", self.procedure_count);

        Ok(())
    }

    /// Get relevant procedures based on a query
    ///
    /// # Arguments
    /// * `query` - The search query (e.g., "how to edit files")
    /// * `top_k` - Number of top results to return
    ///
    /// # Returns
    /// A vector of relevant procedure content chunks
    pub async fn get_relevant_procedures(&self, query: &str, top_k: usize) -> Result<Vec<String>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Use QueryPipeline to retrieve chunks
        let chunks = self
            .query_pipeline
            .retrieve(query, top_k, None)
            .await
            .context("Failed to retrieve procedures")?;

        // Extract content from chunks
        let procedures = chunks.into_iter().map(|chunk| chunk.content).collect();

        Ok(procedures)
    }

    /// Get relevant procedures with metadata
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `top_k` - Number of top results to return
    ///
    /// # Returns
    /// A vector of RetrievedChunk containing content and metadata
    pub async fn get_relevant_procedures_with_metadata(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Use QueryPipeline to retrieve chunks with metadata
        let chunks = self
            .query_pipeline
            .retrieve(query, top_k, None)
            .await
            .context("Failed to retrieve procedures")?;

        Ok(chunks)
    }

    /// Get statistics about loaded procedures
    pub fn stats(&self) -> ProcedureStats {
        ProcedureStats {
            enabled: self.config.enabled,
            total_procedures: self.procedure_count,
            configured_paths: self.config.paths.clone(),
        }
    }
}

/// Statistics about loaded procedures
#[derive(Debug, Clone)]
pub struct ProcedureStats {
    pub enabled: bool,
    pub total_procedures: usize,
    pub configured_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_procedure_dir() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // Create a test procedure
        let procedure_content = r#"---
type: standard-operating-procedure
id: test-procedure
---

# Test Procedure

This is a test procedure for unit testing.

## Usage

Use this procedure when testing the procedure manager.
"#;

        fs::write(temp_dir.path().join("test.md"), procedure_content)?;

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_creates_procedure_manager() {
        let temp_dir = create_test_procedure_dir().await.unwrap();

        let config = ProceduresConfig {
            enabled: true,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = ProcedureManager::new(config).await.unwrap();
        let stats = manager.stats();

        assert!(stats.enabled);
        assert_eq!(stats.total_procedures, 1);
    }

    #[tokio::test]
    async fn test_retrieves_relevant_procedures() {
        let temp_dir = create_test_procedure_dir().await.unwrap();

        let config = ProceduresConfig {
            enabled: true,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = ProcedureManager::new(config).await.unwrap();
        let results = manager
            .get_relevant_procedures("testing", 3)
            .await
            .unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_disabled_manager_returns_empty() {
        let temp_dir = create_test_procedure_dir().await.unwrap();

        let config = ProceduresConfig {
            enabled: false,
            paths: vec![temp_dir.path().to_string_lossy().to_string()],
        };

        let manager = ProcedureManager::new(config).await.unwrap();
        let results = manager
            .get_relevant_procedures("testing", 3)
            .await
            .unwrap();

        assert!(results.is_empty());
    }
}
