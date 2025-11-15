# vtcode-vectordb

Vector database abstraction for VTCode.

## Features

- Unified trait-based interface (`VectorDb`)
- In-memory backend for development
- Support for metadata filtering
- Multiple distance metrics (Cosine, Euclidean, Dot Product)
- Async/await API

## Usage

```rust
use vtcode_vectordb::{InMemoryVectorDb, VectorDb, VectorPoint, Distance};
use serde_json::json;

#[tokio::main]
async fn main() {
    let db = InMemoryVectorDb::new();

    // Create collection
    db.create_collection("embeddings", 384, Distance::Cosine).await.unwrap();

    // Insert vectors
    let points = vec![
        VectorPoint::new(
            "doc1".to_string(),
            vec![0.1; 384],  // 384-dimensional vector
            json!({"text": "Hello world"}),
        ),
    ];
    db.upsert("embeddings", points).await.unwrap();

    // Search
    let query = vec![0.1; 384];
    let results = db.search("embeddings", query, 10, None).await.unwrap();

    for result in results {
        println!("ID: {}, Score: {}", result.id, result.score);
    }
}
```

## Backends

- **InMemoryVectorDb**: In-memory storage (not persistent)
- **Future**: Qdrant, LanceDB, etc.

## License

MIT
