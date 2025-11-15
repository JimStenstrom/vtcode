//! Core types for RAG operations.

use serde::{Deserialize, Serialize};

/// A document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for the document
    pub id: String,
    /// Full text content of the document
    pub content: String,
    /// Arbitrary metadata associated with the document
    pub metadata: serde_json::Value,
}

impl Document {
    /// Create a new document with the given ID and content
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: serde_json::json!({}),
        }
    }

    /// Add metadata to this document
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier for the chunk
    pub id: String,
    /// ID of the parent document
    pub document_id: String,
    /// Text content of this chunk
    pub content: String,
    /// Metadata inherited from the document
    pub metadata: serde_json::Value,
    /// Index of this chunk within the document
    pub chunk_index: usize,
}

/// A retrieved chunk with relevance score
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    /// Unique identifier for the chunk
    pub id: String,
    /// ID of the parent document
    pub document_id: String,
    /// Text content of this chunk
    pub content: String,
    /// Relevance score (higher = more relevant)
    pub score: f32,
    /// Associated metadata
    pub metadata: serde_json::Value,
}
