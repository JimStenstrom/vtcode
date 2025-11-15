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
}
