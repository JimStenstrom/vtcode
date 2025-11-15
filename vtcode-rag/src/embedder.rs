//! Text embedding generation.

use anyhow::Result;
use async_trait::async_trait;

/// Trait for generating embeddings
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>>;

    /// Dimensionality of embeddings
    fn dimensions(&self) -> usize;
}

/// Mock embedder for testing (returns random-ish vectors)
pub struct MockEmbedder {
    dimensions: usize,
}

impl MockEmbedder {
    /// Create a new mock embedder with the specified dimensionality
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Default for MockEmbedder {
    fn default() -> Self {
        Self::new(384)
    }
}

#[async_trait]
impl Embedder for MockEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Simple hash-based deterministic "embedding"
        let hash = simple_hash(text);
        let mut vec = vec![0.0; self.dimensions];

        for (i, val) in vec.iter_mut().enumerate() {
            *val = ((hash + i) % 100) as f32 / 100.0;
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
        self.dimensions
    }
}

fn simple_hash(text: &str) -> usize {
    text.bytes().map(|b| b as usize).sum()
}
