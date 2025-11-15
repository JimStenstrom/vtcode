//! # vtcode-rag
//!
//! Retrieval-Augmented Generation (RAG) for VTCode.
//!
//! Provides document chunking, embedding, and semantic retrieval.

pub mod chunker;
pub mod embedder;
pub mod indexing;
pub mod query;
pub mod types;

// Re-exports
pub use chunker::{Chunker, FixedSizeChunker, SemanticChunker};
pub use embedder::{Embedder, MockEmbedder};
pub use indexing::IndexingPipeline;
pub use query::QueryPipeline;
pub use types::{Chunk, Document, RetrievedChunk};
