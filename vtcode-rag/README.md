# vtcode-rag

RAG (Retrieval-Augmented Generation) system for vtcode providing document indexing, chunking, embedding, and semantic retrieval.

## Overview

Complete pipeline for indexing documents and retrieving relevant content:

- **Document chunking**: Split large documents into manageable pieces
- **Embedding generation**: Convert text to vector representations
- **Indexing pipeline**: Store documents in vector database
- **Query pipeline**: Semantic search with metadata filtering

## Features

- Flexible chunking strategies (fixed-size, semantic)
- Pluggable embedding models
- Metadata-aware retrieval
- Backend-agnostic (uses vtcode-vectordb trait)
- Async/await throughout

## Usage

### Basic Indexing and Retrieval

```rust
use vtcode_rag::{Document, IndexingPipeline, QueryPipeline, FixedSizeChunker, MockEmbedder};
use vtcode_vectordb::{InMemoryVectorDb, Distance};
use std::sync::Arc;

// Setup
let vectordb = Arc::new(InMemoryVectorDb::new());
vectordb.create_collection("docs", 384, Distance::Cosine).await?;

let embedder = Arc::new(MockEmbedder::default());
let chunker = Box::new(FixedSizeChunker::default());

// Index documents
let indexing = IndexingPipeline::new(
    vectordb.clone(),
    embedder.clone(),
    chunker,
    "docs".to_string(),
);

let docs = vec![
    Document::new("auth.rs", "JWT token validation implementation"),
    Document::new("user.rs", "User authentication and login"),
];

indexing.index_documents(docs).await?;

// Query
let query = QueryPipeline::new(vectordb, embedder, "docs".to_string());
let results = query.retrieve("authentication", 5, None).await?;

for result in results {
    println!("Found: {} (score: {})", result.document_id, result.score);
}
```

### With Metadata Filtering

```rust
use serde_json::json;
use vtcode_vectordb::{Filter, Condition};

// Add metadata when indexing
let doc = Document::new("doc.rs", "Content here")
    .with_metadata(json!({
        "lang": "rust",
        "author": "alice",
        "updated": 1234567890
    }));

indexing.index_document(doc).await?;

// Filter during retrieval
let filter = Filter {
    must: vec![
        Condition::Match { key: "lang".into(), value: json!("rust") },
        Condition::Range {
            key: "updated".into(),
            gte: Some(1234560000.0),
            lte: None,
        },
    ],
    ..Default::default()
};

let results = query.retrieve("content", 10, Some(filter)).await?;
```

## Chunking Strategies

### FixedSizeChunker

Split documents by character count with overlap:

```rust
let chunker = FixedSizeChunker {
    chunk_size: 512,
    overlap: 64,
};
```

**Best for**: General text, even distribution

### SemanticChunker (Future)

Split by semantic boundaries (sentences, paragraphs):

```rust
let chunker = SemanticChunker {
    max_chunk_size: 1024,
    min_chunk_size: 128,
    boundary_type: BoundaryType::Sentence,
};
```

**Best for**: Natural language, preserving meaning

## Embedders

### MockEmbedder

Simple deterministic embedder for testing:

```rust
let embedder = MockEmbedder::default();  // 384 dimensions
```

### OpenAIEmbedder (Future)

Use OpenAI's embedding API:

```rust
let embedder = OpenAIEmbedder::new(
    "text-embedding-3-small",
    api_key,
);  // 1536 dimensions
```

### LocalEmbedder (Future)

Run embedding models locally:

```rust
let embedder = LocalEmbedder::new("BAAI/bge-small-en-v1.5");  // 384 dimensions
```

## Architecture

```
Document → Chunker → [Chunk, Chunk, ...]
    ↓
Embedder → [Vector, Vector, ...]
    ↓
VectorDb → Indexed

Query → Embedder → Vector
    ↓
VectorDb Search → [Result, Result, ...]
    ↓
RetrievalResults
```

## Testing

```bash
cargo test --package vtcode-rag
```

Integration tests in `tests/integration_rag_pipeline.rs`.

## Future Enhancements

- **Graph RAG**: Hybrid vector + graph retrieval
- **Reranking**: Post-retrieval reranking with cross-encoders
- **Hybrid search**: Combine keyword and semantic search
- **Streaming indexing**: Index documents as they're created
- **Multi-vector search**: Multiple query vectors with fusion

## Related Crates

- `vtcode-vectordb`: Vector database abstraction
- `vtcode-memory`: Memory system integration
- `vtcode-core`: Context management
