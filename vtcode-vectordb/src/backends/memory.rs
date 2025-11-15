use crate::error::{Result, VectorDbError};
use crate::traits::VectorDb;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Collection metadata
#[derive(Debug, Clone)]
struct Collection {
    vector_size: usize,
    distance: Distance,
    points: HashMap<String, VectorPoint>,
}

/// In-memory vector database implementation
///
/// Good for development and testing. Not persistent.
pub struct InMemoryVectorDb {
    collections: Arc<RwLock<HashMap<String, Collection>>>,
}

impl InMemoryVectorDb {
    /// Create a new in-memory vector database
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryVectorDb {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorDb for InMemoryVectorDb {
    async fn create_collection(
        &self,
        name: &str,
        vector_size: usize,
        distance: Distance,
    ) -> Result<()> {
        let mut collections = self.collections.write().await;

        if collections.contains_key(name) {
            return Err(VectorDbError::CollectionAlreadyExists(name.to_string()));
        }

        collections.insert(
            name.to_string(),
            Collection {
                vector_size,
                distance,
                points: HashMap::new(),
            },
        );

        tracing::debug!("Created collection '{}' (dim: {})", name, vector_size);
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write().await;

        if collections.remove(name).is_none() {
            return Err(VectorDbError::CollectionNotFound(name.to_string()));
        }

        tracing::debug!("Deleted collection '{}'", name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = self.collections.read().await;
        Ok(collections.contains_key(name))
    }

    async fn upsert(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()> {
        let mut collections = self.collections.write().await;

        let coll = collections
            .get_mut(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        for point in points {
            // Validate vector dimensions
            if point.vector.len() != coll.vector_size {
                return Err(VectorDbError::DimensionMismatch {
                    expected: coll.vector_size,
                    actual: point.vector.len(),
                });
            }

            coll.points.insert(point.id.clone(), point);
        }

        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<Filter>,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.read().await;

        let coll = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        // Validate query vector dimensions
        if query_vector.len() != coll.vector_size {
            return Err(VectorDbError::DimensionMismatch {
                expected: coll.vector_size,
                actual: query_vector.len(),
            });
        }

        // Calculate similarities
        let mut results: Vec<SearchResult> = coll
            .points
            .values()
            .filter(|point| apply_filter(point, &filter))
            .map(|point| SearchResult {
                id: point.id.clone(),
                score: calculate_similarity(&query_vector, &point.vector, coll.distance),
                payload: point.payload.clone(),
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(limit);

        Ok(results)
    }

    async fn get(&self, collection: &str, id: &str) -> Result<Option<VectorPoint>> {
        let collections = self.collections.read().await;

        let coll = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        Ok(coll.points.get(id).cloned())
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let mut collections = self.collections.write().await;

        let coll = collections
            .get_mut(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        for id in ids {
            coll.points.remove(&id);
        }

        Ok(())
    }

    async fn scroll(
        &self,
        collection: &str,
        limit: usize,
        offset: Option<String>,
    ) -> Result<ScrollResult> {
        let collections = self.collections.read().await;

        let coll = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        // Simple pagination: use offset as starting index
        let start_idx = offset
            .and_then(|o| o.parse::<usize>().ok())
            .unwrap_or(0);

        let points: Vec<_> = coll
            .points
            .values()
            .skip(start_idx)
            .take(limit)
            .cloned()
            .collect();

        let next_offset = if start_idx + points.len() < coll.points.len() {
            Some((start_idx + points.len()).to_string())
        } else {
            None
        };

        Ok(ScrollResult::new(points, next_offset))
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let collections = self.collections.read().await;

        let coll = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound(collection.to_string()))?;

        Ok(coll.points.len())
    }
}

/// Calculate similarity between two vectors
fn calculate_similarity(a: &[f32], b: &[f32], distance: Distance) -> f32 {
    match distance {
        Distance::Cosine => {
            let dot = dot_product(a, b);
            let norm_a = magnitude(a);
            let norm_b = magnitude(b);
            if norm_a == 0.0 || norm_b == 0.0 {
                0.0
            } else {
                dot / (norm_a * norm_b)
            }
        }
        Distance::Euclidean => {
            // Convert distance to similarity (inverse)
            let dist = euclidean_distance(a, b);
            1.0 / (1.0 + dist)
        }
        Distance::DotProduct => dot_product(a, b),
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Apply metadata filter to a point
fn apply_filter(point: &VectorPoint, filter: &Option<Filter>) -> bool {
    let Some(filter) = filter else {
        return true;
    };

    // Check MUST conditions (AND)
    for condition in &filter.must {
        if !matches_condition(point, condition) {
            return false;
        }
    }

    // Check MUST_NOT conditions (NOT)
    for condition in &filter.must_not {
        if matches_condition(point, condition) {
            return false;
        }
    }

    // Check SHOULD conditions (OR)
    if !filter.should.is_empty()
        && !filter.should.iter().any(|c| matches_condition(point, c))
    {
        return false;
    }

    true
}

fn matches_condition(point: &VectorPoint, condition: &Condition) -> bool {
    match condition {
        Condition::Match { key, value } => {
            point.payload.get(key) == Some(value)
        }
        Condition::Range { key, gte, lte } => {
            if let Some(v) = point.payload.get(key).and_then(|v| v.as_f64()) {
                if let Some(min) = gte {
                    if v < *min {
                        return false;
                    }
                }
                if let Some(max) = lte {
                    if v > *max {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        }
        Condition::HasKey(key) => point.payload.get(key).is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_collection() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();
        assert!(db.collection_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0, 0.0], json!({"text": "hello"})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0, 0.0], json!({"text": "world"})),
        ];

        db.upsert("test", points).await.unwrap();

        let results = db.search("test", vec![1.0, 0.0, 0.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "1"); // Most similar
    }

    #[tokio::test]
    async fn test_filtering() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"category": "A"})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"category": "B"})),
        ];

        db.upsert("test", points).await.unwrap();

        let filter = Filter {
            must: vec![Condition::Match {
                key: "category".to_string(),
                value: json!("A"),
            }],
            ..Default::default()
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
    }

    // ========== Edge Case Tests ==========

    #[tokio::test]
    async fn test_empty_collection_search() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        // Search in empty collection should return empty results
        let results = db.search("test", vec![1.0, 0.0, 0.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_upsert_update_existing() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        // Insert initial point
        db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"version": 1}))
        ]).await.unwrap();

        // Update same ID
        db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![0.0, 1.0], json!({"version": 2}))
        ]).await.unwrap();

        // Should have only 1 point with updated values
        let count = db.count("test").await.unwrap();
        assert_eq!(count, 1);

        let point = db.get("test", "1").await.unwrap().unwrap();
        assert_eq!(point.vector, vec![0.0, 1.0]);
        assert_eq!(point.payload["version"], 2);
    }

    #[tokio::test]
    async fn test_zero_vectors() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        // Insert zero vector
        db.upsert("test", vec![
            VectorPoint::new("zero".to_string(), vec![0.0, 0.0, 0.0], json!({"type": "zero"}))
        ]).await.unwrap();

        // Search with zero vector (should handle gracefully)
        let results = db.search("test", vec![0.0, 0.0, 0.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    // ========== Error Handling Tests ==========

    #[tokio::test]
    async fn test_dimension_mismatch_on_upsert() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        // Try to insert vector with wrong dimension
        let result = db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({}))  // 2D instead of 3D
        ]).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            VectorDbError::DimensionMismatch { expected, actual } => {
                assert_eq!(expected, 3);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected DimensionMismatch error"),
        }
    }

    #[tokio::test]
    async fn test_dimension_mismatch_on_search() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        // Try to search with wrong dimension
        let result = db.search("test", vec![1.0, 0.0], 10, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            VectorDbError::DimensionMismatch { expected, actual } => {
                assert_eq!(expected, 3);
                assert_eq!(actual, 2);
            }
            _ => panic!("Expected DimensionMismatch error"),
        }
    }

    #[tokio::test]
    async fn test_collection_not_found() {
        let db = InMemoryVectorDb::new();

        // Try to operate on non-existent collection
        let result = db.upsert("nonexistent", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({}))
        ]).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorDbError::CollectionNotFound(_)));
    }

    #[tokio::test]
    async fn test_duplicate_collection_creation() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 3, Distance::Cosine).await.unwrap();

        // Try to create same collection again
        let result = db.create_collection("test", 3, Distance::Cosine).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorDbError::CollectionAlreadyExists(_)));
    }

    // ========== CRUD Operation Tests ==========

    #[tokio::test]
    async fn test_get_operation() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        // Insert point
        db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"name": "test"}))
        ]).await.unwrap();

        // Get existing point
        let point = db.get("test", "1").await.unwrap();
        assert!(point.is_some());
        let point = point.unwrap();
        assert_eq!(point.id, "1");
        assert_eq!(point.vector, vec![1.0, 0.0]);

        // Get non-existent point
        let point = db.get("test", "nonexistent").await.unwrap();
        assert!(point.is_none());
    }

    #[tokio::test]
    async fn test_delete_operation() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        // Insert points
        db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({})),
        ]).await.unwrap();

        assert_eq!(db.count("test").await.unwrap(), 3);

        // Delete some points
        db.delete("test", vec!["1".to_string(), "3".to_string()]).await.unwrap();

        assert_eq!(db.count("test").await.unwrap(), 1);
        assert!(db.get("test", "1").await.unwrap().is_none());
        assert!(db.get("test", "2").await.unwrap().is_some());
        assert!(db.get("test", "3").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_scroll_pagination() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        // Insert multiple points
        let points: Vec<_> = (0..10).map(|i| {
            VectorPoint::new(
                format!("point_{}", i),
                vec![i as f32, 0.0],
                json!({"index": i})
            )
        }).collect();
        db.upsert("test", points).await.unwrap();

        // First page
        let result = db.scroll("test", 3, None).await.unwrap();
        assert_eq!(result.points.len(), 3);
        assert!(result.next_offset.is_some());

        // Second page
        let result = db.scroll("test", 3, result.next_offset).await.unwrap();
        assert_eq!(result.points.len(), 3);
        assert!(result.next_offset.is_some());

        // Last page
        let offset = result.next_offset;
        let result = db.scroll("test", 5, offset).await.unwrap();
        assert_eq!(result.points.len(), 4); // Only 4 left
        assert!(result.next_offset.is_none()); // No more pages
    }

    #[tokio::test]
    async fn test_count_operation() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        assert_eq!(db.count("test").await.unwrap(), 0);

        db.upsert("test", vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({})),
        ]).await.unwrap();
        assert_eq!(db.count("test").await.unwrap(), 1);

        db.upsert("test", vec![
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({})),
        ]).await.unwrap();
        assert_eq!(db.count("test").await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_delete_collection() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        assert!(db.collection_exists("test").await.unwrap());

        db.delete_collection("test").await.unwrap();

        assert!(!db.collection_exists("test").await.unwrap());

        // Try to delete non-existent collection
        let result = db.delete_collection("test").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorDbError::CollectionNotFound(_)));
    }

    // ========== Advanced Filter Tests ==========

    #[tokio::test]
    async fn test_filter_should_conditions() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"category": "A"})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"category": "B"})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({"category": "C"})),
        ];
        db.upsert("test", points).await.unwrap();

        // SHOULD: match either A or B (OR logic)
        let filter = Filter {
            should: vec![
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("A"),
                },
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("B"),
                },
            ],
            ..Default::default()
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"2"));
    }

    #[tokio::test]
    async fn test_filter_must_not_conditions() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"category": "A"})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"category": "B"})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({"category": "C"})),
        ];
        db.upsert("test", points).await.unwrap();

        // MUST_NOT: exclude category A
        let filter = Filter {
            must_not: vec![
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("A"),
                },
            ],
            ..Default::default()
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(!ids.contains(&"1"));
        assert!(ids.contains(&"2"));
        assert!(ids.contains(&"3"));
    }

    #[tokio::test]
    async fn test_filter_range_conditions() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"score": 10})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"score": 25})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({"score": 50})),
        ];
        db.upsert("test", points).await.unwrap();

        // Range: 20 <= score <= 60
        let filter = Filter {
            must: vec![
                Condition::Range {
                    key: "score".to_string(),
                    gte: Some(20.0),
                    lte: Some(60.0),
                },
            ],
            ..Default::default()
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"2"));
        assert!(ids.contains(&"3"));
    }

    #[tokio::test]
    async fn test_filter_has_key_condition() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"name": "Alice"})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"age": 25})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({"name": "Bob", "age": 30})),
        ];
        db.upsert("test", points).await.unwrap();

        // Has key: must have "name" field
        let filter = Filter {
            must: vec![Condition::HasKey("name".to_string())],
            ..Default::default()
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"3"));
    }

    #[tokio::test]
    async fn test_filter_combined_conditions() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Cosine).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({"category": "A", "score": 10})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({"category": "A", "score": 25})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({"category": "B", "score": 30})),
            VectorPoint::new("4".to_string(), vec![0.5, 0.5], json!({"category": "C", "score": 40})),
        ];
        db.upsert("test", points).await.unwrap();

        // Complex filter: (category = A OR category = B) AND score >= 20 AND NOT category = C
        let filter = Filter {
            must: vec![
                Condition::Range {
                    key: "score".to_string(),
                    gte: Some(20.0),
                    lte: None,
                },
            ],
            should: vec![
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("A"),
                },
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("B"),
                },
            ],
            must_not: vec![
                Condition::Match {
                    key: "category".to_string(),
                    value: json!("C"),
                },
            ],
        };

        let results = db.search("test", vec![1.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 2); // Only points 2 and 3
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"2"));
        assert!(ids.contains(&"3"));
    }

    // ========== Distance Metric Tests ==========

    #[tokio::test]
    async fn test_euclidean_distance() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::Euclidean).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![0.0, 0.0], json!({})),
            VectorPoint::new("2".to_string(), vec![3.0, 4.0], json!({})),
            VectorPoint::new("3".to_string(), vec![1.0, 0.0], json!({})),
        ];
        db.upsert("test", points).await.unwrap();

        // Query from origin: closest should be point 1, then 3, then 2
        let results = db.search("test", vec![0.0, 0.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, "1"); // Distance 0
        assert_eq!(results[1].id, "3"); // Distance 1
        assert_eq!(results[2].id, "2"); // Distance 5
    }

    #[tokio::test]
    async fn test_dot_product_distance() {
        let db = InMemoryVectorDb::new();
        db.create_collection("test", 2, Distance::DotProduct).await.unwrap();

        let points = vec![
            VectorPoint::new("1".to_string(), vec![1.0, 0.0], json!({})),
            VectorPoint::new("2".to_string(), vec![0.0, 1.0], json!({})),
            VectorPoint::new("3".to_string(), vec![1.0, 1.0], json!({})),
        ];
        db.upsert("test", points).await.unwrap();

        // Query [1.0, 1.0]: highest dot product should be point 3 (2.0)
        let results = db.search("test", vec![1.0, 1.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, "3"); // Dot product: 2.0
    }
}
