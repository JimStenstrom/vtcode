//! Document chunking strategies.

use crate::types::{Chunk, Document};
use uuid::Uuid;

/// Trait for chunking strategies
pub trait Chunker: Send + Sync {
    /// Split a document into chunks
    fn chunk(&self, document: &Document) -> Vec<Chunk>;
}

/// Fixed-size chunker with overlapping windows
pub struct FixedSizeChunker {
    /// Maximum size of each chunk in characters
    pub chunk_size: usize,
    /// Number of overlapping characters between chunks
    pub overlap: usize,
}

impl Default for FixedSizeChunker {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap: 128,
        }
    }
}

impl Chunker for FixedSizeChunker {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        let content = &document.content;
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < content.len() {
            let end = (start + self.chunk_size).min(content.len());
            let chunk_content = content[start..end].to_string();

            let chunk = Chunk {
                id: Uuid::new_v4().to_string(),
                document_id: document.id.clone(),
                content: chunk_content,
                metadata: document.metadata.clone(),
                chunk_index,
            };

            chunks.push(chunk);

            if end >= content.len() {
                break;
            }

            chunk_index += 1;
            start = end.saturating_sub(self.overlap);
        }

        chunks
    }
}

/// Semantic chunker (by separators like paragraphs)
pub struct SemanticChunker {
    /// Separator string to split on (e.g., "\n\n" for paragraphs)
    pub separator: String,
    /// Maximum size of each chunk in characters
    pub max_chunk_size: usize,
}

impl Default for SemanticChunker {
    fn default() -> Self {
        Self {
            separator: "\n\n".to_string(),
            max_chunk_size: 1000,
        }
    }
}

impl Chunker for SemanticChunker {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        let parts: Vec<&str> = document.content.split(&self.separator).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for part in parts {
            if current_chunk.len() + part.len() > self.max_chunk_size && !current_chunk.is_empty() {
                // Create chunk
                chunks.push(Chunk {
                    id: Uuid::new_v4().to_string(),
                    document_id: document.id.clone(),
                    content: current_chunk.trim().to_string(),
                    metadata: document.metadata.clone(),
                    chunk_index,
                });

                chunk_index += 1;
                current_chunk = String::new();
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str(&self.separator);
            }
            current_chunk.push_str(part);
        }

        // Add final chunk
        if !current_chunk.is_empty() {
            chunks.push(Chunk {
                id: Uuid::new_v4().to_string(),
                document_id: document.id.clone(),
                content: current_chunk.trim().to_string(),
                metadata: document.metadata.clone(),
                chunk_index,
            });
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_size_chunker() {
        let doc = Document::new("test", "a".repeat(1000));
        let chunker = FixedSizeChunker {
            chunk_size: 200,
            overlap: 50,
        };

        let chunks = chunker.chunk(&doc);
        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].content.len(), 200);
    }

    #[test]
    fn test_semantic_chunker() {
        let doc = Document::new("test", "Para 1\n\nPara 2\n\nPara 3");
        let chunker = SemanticChunker {
            separator: "\n\n".to_string(),
            max_chunk_size: 10,  // Small size to force separate chunks
        };

        let chunks = chunker.chunk(&doc);
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::new("test", "");
        let chunker = FixedSizeChunker::default();

        let chunks = chunker.chunk(&doc);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_small_document_no_overlap() {
        let doc = Document::new("test", "Small");
        let chunker = FixedSizeChunker {
            chunk_size: 512,
            overlap: 128,
        };

        let chunks = chunker.chunk(&doc);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Small");
    }

    #[test]
    fn test_chunk_indices() {
        let doc = Document::new("test", "a".repeat(1000));
        let chunker = FixedSizeChunker {
            chunk_size: 200,
            overlap: 50,
        };

        let chunks = chunker.chunk(&doc);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i);
            assert_eq!(chunk.document_id, "test");
        }
    }
}
