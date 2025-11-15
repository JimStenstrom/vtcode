//! Document query pipeline.

use crate::embedder::Embedder;
use crate::types::RetrievedChunk;
use anyhow::Result;
use std::sync::Arc;
use vtcode_vectordb::{Filter, VectorDb};

/// Pipeline for querying documents
pub struct QueryPipeline {
    vectordb: Arc<dyn VectorDb>,
    embedder: Arc<dyn Embedder>,
    collection: String,
}

impl QueryPipeline {
    pub fn new(
        vectordb: Arc<dyn VectorDb>,
        embedder: Arc<dyn Embedder>,
        collection: String,
    ) -> Self {
        Self {
            vectordb,
            embedder,
            collection,
        }
    }

    /// Retrieve relevant chunks for a query
    pub async fn retrieve(
        &self,
        query: &str,
        limit: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<RetrievedChunk>> {
        // 1. Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;

        // 2. Search vectordb
        let results = self
            .vectordb
            .search(&self.collection, query_embedding, limit, filter)
            .await?;

        // 3. Convert to RetrievedChunk
        let chunks = results
            .into_iter()
            .map(|result| {
                let document_id = result.payload["document_id"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let content = result.payload["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                RetrievedChunk {
                    id: result.id,
                    document_id,
                    content,
                    score: result.score,
                    metadata: result.payload,
                }
            })
            .collect();

        Ok(chunks)
    }
}
