use crate::chunker::Chunker;
use crate::embedder::Embedder;
use crate::types::Document;
use anyhow::Result;
use std::sync::Arc;
use vtcode_vectordb::{Distance, VectorDb, VectorPoint};

/// Pipeline for indexing documents
pub struct IndexingPipeline {
    vectordb: Arc<dyn VectorDb>,
    embedder: Arc<dyn Embedder>,
    chunker: Box<dyn Chunker>,
    collection: String,
}

impl IndexingPipeline {
    pub fn new(
        vectordb: Arc<dyn VectorDb>,
        embedder: Arc<dyn Embedder>,
        chunker: Box<dyn Chunker>,
        collection: String,
    ) -> Self {
        Self {
            vectordb,
            embedder,
            chunker,
            collection,
        }
    }

    /// Ensure collection exists
    pub async fn ensure_collection(&self) -> Result<()> {
        if !self.vectordb.collection_exists(&self.collection).await? {
            self.vectordb
                .create_collection(
                    &self.collection,
                    self.embedder.dimensions(),
                    Distance::Cosine,
                )
                .await?;
            tracing::info!("Created collection: {}", self.collection);
        }
        Ok(())
    }

    /// Index a single document
    pub async fn index_document(&self, document: Document) -> Result<()> {
        self.ensure_collection().await?;

        // 1. Chunk document
        let chunks = self.chunker.chunk(&document);
        tracing::debug!("Chunked document {} into {} chunks", document.id, chunks.len());

        // 2. Generate embeddings
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedder.embed_batch(texts).await?;

        // 3. Create vector points
        let mut points = Vec::new();
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            let mut metadata = chunk.metadata.clone();
            metadata["document_id"] = serde_json::json!(chunk.document_id);
            metadata["chunk_index"] = serde_json::json!(chunk.chunk_index);
            metadata["content"] = serde_json::json!(chunk.content);

            points.push(VectorPoint::new(
                chunk.id.clone(),
                embedding.clone(),
                metadata,
            ));
        }

        // 4. Upsert to vectordb
        self.vectordb.upsert(&self.collection, points).await?;

        tracing::info!("Indexed document: {}", document.id);
        Ok(())
    }

    /// Index multiple documents
    pub async fn index_documents(&self, documents: Vec<Document>) -> Result<()> {
        for doc in documents {
            self.index_document(doc).await?;
        }
        Ok(())
    }
}
