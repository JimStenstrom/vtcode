# vtcode-rag

Retrieval-Augmented Generation system for VTCode.

## Usage

```rust
use vtcode_rag::{Document, IndexingPipeline, QueryPipeline, MockEmbedder, FixedSizeChunker};
use vtcode_vectordb::InMemoryVectorDb;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let vectordb = Arc::new(InMemoryVectorDb::new());
    let embedder = Arc::new(MockEmbedder::default());
    let chunker = Box::new(FixedSizeChunker::default());

    // Index documents
    let indexing = IndexingPipeline::new(
        Arc::clone(&vectordb),
        Arc::clone(&embedder),
        chunker,
        "docs".to_string(),
    );

    let doc = Document::new("auth.rs", "JWT validation code...");
    indexing.index_document(doc).await.unwrap();

    // Query
    let query = QueryPipeline::new(vectordb, embedder, "docs".to_string());
    let results = query.retrieve("how to validate tokens?", 5, None).await.unwrap();

    println!("Found {} results", results.len());
}
```

## License

MIT
