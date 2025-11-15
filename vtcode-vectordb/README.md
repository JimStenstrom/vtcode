# vtcode-vectordb

Abstract vector database interface for vtcode with pluggable backends.

## Overview

Provides a unified interface for vector similarity search with support for multiple backends:

- **InMemoryVectorDb**: Fast in-memory implementation for development and testing
- **QdrantBackend** (future): Production-grade vector database

## Features

- Backend-agnostic trait-based design
- Multiple distance metrics (Cosine, Euclidean, Dot Product)
- Metadata filtering with complex conditions (must, must_not, should)
- Dimension validation
- Collection management (create, delete)
- CRUD operations (upsert, search, delete)

## Usage

```rust
use vtcode_vectordb::{InMemoryVectorDb, VectorDb, VectorPoint, Distance, Filter, Condition};
use serde_json::json;
use std::sync::Arc;

// Create database
let db = Arc::new(InMemoryVectorDb::new());

// Create collection
db.create_collection("docs", 384, Distance::Cosine).await?;

// Insert vectors
let points = vec![
    VectorPoint::new(
        "doc1",
        vec![0.1, 0.2, 0.3, /* ... 384 dims */],
        json!({"title": "Introduction", "lang": "en"}),
    ),
];
db.upsert("docs", points).await?;

// Search with filtering
let filter = Filter {
    must: vec![Condition::Match {
        key: "lang".to_string(),
        value: json!("en"),
    }],
    must_not: vec![],
    should: vec![],
};

let query_vector = vec![0.1, 0.2, 0.3, /* ... */];
let results = db.search("docs", query_vector, 10, Some(filter)).await?;

for result in results {
    println!("ID: {}, Score: {}, Metadata: {:?}",
        result.id, result.score, result.metadata);
}
```

## Distance Metrics

- **Cosine**: Measures angle between vectors (0-1, higher is more similar)
- **Euclidean**: Measures straight-line distance (lower is more similar)
- **DotProduct**: Inner product of vectors (higher is more similar)

## Metadata Filtering

Complex filters with boolean logic:

```rust
Filter {
    must: vec![
        Condition::Match { key: "lang", value: json!("en") },
        Condition::Range { key: "score", gte: Some(0.8), lte: None },
    ],
    must_not: vec![
        Condition::Match { key: "archived", value: json!(true) },
    ],
    should: vec![
        Condition::Match { key: "featured", value: json!(true) },
    ],
}
```

## Backends

### InMemoryVectorDb

Fast in-memory implementation suitable for:
- Development and testing
- Small datasets (< 100K vectors)
- Stateless workloads

### Qdrant (Future)

Production backend supporting:
- Distributed deployment
- Persistent storage
- Large-scale datasets (millions of vectors)
- Advanced filtering and search

## Testing

```bash
cargo test --package vtcode-vectordb
```

Integration tests in `tests/integration_rag_pipeline.rs`.

## Configuration

See `VectorDbConfig` in vtcode-config:

```toml
[vectordb]
backend = "memory"
collection_prefix = "vtcode"
embedding_dimensions = 384

[vectordb.qdrant]  # Future
url = "http://localhost:6333"
api_key = "your-api-key"
```

## Related Crates

- `vtcode-rag`: RAG pipeline built on top of this interface
- `vtcode-config`: Configuration types
