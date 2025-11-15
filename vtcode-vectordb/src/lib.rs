//! # vtcode-vectordb
//!
//! Vector database abstraction for VTCode.
//!
//! Provides a unified interface for storing and querying vector embeddings,
//! with support for multiple backends.
//!
//! ## Example
//!
//! ```rust
//! use vtcode_vectordb::{InMemoryVectorDb, VectorDb, VectorPoint, Distance};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() {
//!     let db = InMemoryVectorDb::new();
//!
//!     // Create collection
//!     db.create_collection("docs", 3, Distance::Cosine).await.unwrap();
//!
//!     // Insert points
//!     let points = vec![
//!         VectorPoint::new(
//!             "doc1".to_string(),
//!             vec![0.1, 0.2, 0.3],
//!             json!({"text": "example document"}),
//!         ),
//!     ];
//!     db.upsert("docs", points).await.unwrap();
//!
//!     // Search
//!     let results = db.search("docs", vec![0.1, 0.2, 0.3], 10, None).await.unwrap();
//!     println!("Found {} results", results.len());
//! }
//! ```

pub mod backends;
pub mod error;
pub mod traits;
pub mod types;

// Re-export main types
pub use backends::memory::InMemoryVectorDb;
pub use error::{Result, VectorDbError};
pub use traits::VectorDb;
pub use types::{Condition, Distance, Filter, ScrollResult, SearchResult, VectorPoint};
