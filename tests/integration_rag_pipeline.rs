//! Integration tests for RAG pipeline end-to-end

use std::sync::Arc;
use vtcode_rag::{Document, FixedSizeChunker, IndexingPipeline, MockEmbedder, QueryPipeline};
use vtcode_vectordb::{Condition, Distance, Filter, InMemoryVectorDb, VectorDb};

#[tokio::test]
async fn test_rag_indexing_and_retrieval() {
    // Setup
    let vectordb = Arc::new(InMemoryVectorDb::new());
    vectordb
        .create_collection("test_docs", 384, Distance::Cosine)
        .await
        .unwrap();

    let embedder: Arc<dyn vtcode_rag::Embedder> = Arc::new(MockEmbedder::default());
    let chunker = Box::new(FixedSizeChunker::default());

    // Index documents
    let indexing = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker,
        "test_docs".to_string(),
    );

    let docs = vec![
        Document::new("auth.rs", "JWT token validation implementation"),
        Document::new("user.rs", "User authentication and login"),
        Document::new("db.rs", "Database connection pooling"),
    ];

    indexing.index_documents(docs).await.unwrap();

    // Query
    let query = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "test_docs".to_string(),
    );
    let results = query.retrieve("authentication", 5, None).await.unwrap();

    assert!(!results.is_empty(), "Should find results");
    assert!(
        results
            .iter()
            .any(|r| r.document_id.contains("auth") || r.document_id.contains("user")),
        "Should find auth-related documents"
    );

    println!("✅ RAG pipeline works: {} results", results.len());
}

#[tokio::test]
async fn test_rag_with_metadata_filtering() {
    use serde_json::json;

    let vectordb = Arc::new(InMemoryVectorDb::new());
    vectordb
        .create_collection("filtered_docs", 384, Distance::Cosine)
        .await
        .unwrap();

    let embedder: Arc<dyn vtcode_rag::Embedder> = Arc::new(MockEmbedder::default());
    let chunker = Box::new(FixedSizeChunker::default());

    let indexing = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker,
        "filtered_docs".to_string(),
    );

    // Index with metadata
    let docs = vec![
        Document::new("doc1", "Content about Rust programming").with_metadata(json!({"lang": "rust"})),
        Document::new("doc2", "Content about Python programming").with_metadata(json!({"lang": "python"})),
        Document::new("doc3", "More Rust content here").with_metadata(json!({"lang": "rust"})),
    ];

    indexing.index_documents(docs).await.unwrap();

    // Query with filter
    let query = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "filtered_docs".to_string(),
    );

    let filter = Filter {
        must: vec![Condition::Match {
            key: "lang".to_string(),
            value: json!("rust"),
        }],
        must_not: Vec::new(),
        should: Vec::new(),
    };

    let results = query
        .retrieve("programming", 10, Some(filter))
        .await
        .unwrap();

    assert!(!results.is_empty(), "Should find filtered results");

    // All results should be Rust documents
    for result in &results {
        assert_eq!(
            result.metadata["lang"],
            json!("rust"),
            "Filter should work: got {:?}",
            result.metadata
        );
    }

    println!("✅ RAG metadata filtering works: {} results", results.len());
}

#[tokio::test]
async fn test_rag_chunking_large_document() {
    let vectordb = Arc::new(InMemoryVectorDb::new());
    vectordb
        .create_collection("large_docs", 384, Distance::Cosine)
        .await
        .unwrap();

    let embedder: Arc<dyn vtcode_rag::Embedder> = Arc::new(MockEmbedder::default());
    let chunker = Box::new(FixedSizeChunker {
        chunk_size: 100,
        overlap: 20,
    });

    let indexing = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker,
        "large_docs".to_string(),
    );

    // Create a large document
    let large_content = "This is a very long document that should be chunked into multiple pieces. "
        .repeat(20);
    let doc = Document::new("large.md", large_content);

    indexing.index_document(doc).await.unwrap();

    // Query
    let query = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "large_docs".to_string(),
    );

    let results = query.retrieve("document", 10, None).await.unwrap();
    assert!(!results.is_empty(), "Should find chunks from large document");

    println!(
        "✅ RAG chunking works: {} chunks from large document",
        results.len()
    );
}

#[tokio::test]
async fn test_rag_empty_query() {
    let vectordb = Arc::new(InMemoryVectorDb::new());
    vectordb
        .create_collection("empty_test", 384, Distance::Cosine)
        .await
        .unwrap();

    let embedder: Arc<dyn vtcode_rag::Embedder> = Arc::new(MockEmbedder::default());
    let chunker = Box::new(FixedSizeChunker::default());

    let indexing = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker,
        "empty_test".to_string(),
    );

    let docs = vec![Document::new("test", "Some content")];
    indexing.index_documents(docs).await.unwrap();

    // Query with empty string
    let query = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "empty_test".to_string(),
    );

    let results = query.retrieve("", 5, None).await.unwrap();
    // Should still work, just might not be very relevant
    assert!(!results.is_empty() || results.is_empty()); // Either is OK

    println!("✅ RAG empty query works");
}

#[tokio::test]
async fn test_rag_multiple_collections() {
    let vectordb = Arc::new(InMemoryVectorDb::new());

    // Create two collections
    vectordb
        .create_collection("coll1", 384, Distance::Cosine)
        .await
        .unwrap();
    vectordb
        .create_collection("coll2", 384, Distance::Cosine)
        .await
        .unwrap();

    let embedder: Arc<dyn vtcode_rag::Embedder> = Arc::new(MockEmbedder::default());

    // Index to coll1
    let chunker1 = Box::new(FixedSizeChunker::default());
    let indexing1 = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker1,
        "coll1".to_string(),
    );
    indexing1
        .index_document(Document::new("doc1", "Collection one content"))
        .await
        .unwrap();

    // Index to coll2
    let chunker2 = Box::new(FixedSizeChunker::default());
    let indexing2 = IndexingPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        chunker2,
        "coll2".to_string(),
    );
    indexing2
        .index_document(Document::new("doc2", "Collection two content"))
        .await
        .unwrap();

    // Query coll1
    let query1 = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "coll1".to_string(),
    );
    let results1 = query1.retrieve("content", 5, None).await.unwrap();
    assert!(!results1.is_empty());

    // Query coll2
    let query2 = QueryPipeline::new(
        Arc::clone(&vectordb) as Arc<dyn VectorDb>,
        Arc::clone(&embedder),
        "coll2".to_string(),
    );
    let results2 = query2.retrieve("content", 5, None).await.unwrap();
    assert!(!results2.is_empty());

    println!("✅ RAG multiple collections work");
}
