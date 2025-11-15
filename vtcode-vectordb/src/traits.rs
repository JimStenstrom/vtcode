use crate::error::Result;
use crate::types::*;
use async_trait::async_trait;

/// Core vector database operations
#[async_trait]
pub trait VectorDb: Send + Sync {
    /// Create a new collection with specified parameters
    ///
    /// # Arguments
    /// * `name` - Collection name
    /// * `vector_size` - Dimensionality of vectors
    /// * `distance` - Distance metric to use
    async fn create_collection(
        &self,
        name: &str,
        vector_size: usize,
        distance: Distance,
    ) -> Result<()>;

    /// Delete a collection
    async fn delete_collection(&self, name: &str) -> Result<()>;

    /// Check if a collection exists
    async fn collection_exists(&self, name: &str) -> Result<bool>;

    /// Insert or update vector points
    ///
    /// If a point with the same ID exists, it will be updated.
    async fn upsert(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()>;

    /// Search for similar vectors
    ///
    /// # Arguments
    /// * `collection` - Collection to search in
    /// * `query_vector` - Query vector
    /// * `limit` - Maximum number of results
    /// * `filter` - Optional metadata filter
    async fn search(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<SearchResult>>;

    /// Get a specific point by ID
    async fn get(&self, collection: &str, id: &str) -> Result<Option<VectorPoint>>;

    /// Delete points by IDs
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()>;

    /// Paginate through all points
    ///
    /// # Arguments
    /// * `collection` - Collection to scroll
    /// * `limit` - Points per page
    /// * `offset` - Continuation token from previous scroll (None for first page)
    async fn scroll(
        &self,
        collection: &str,
        limit: usize,
        offset: Option<String>,
    ) -> Result<ScrollResult>;

    /// Get count of points in collection
    async fn count(&self, collection: &str) -> Result<usize>;
}
