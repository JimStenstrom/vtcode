use serde::{Deserialize, Serialize};

/// A point in vector space with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    /// Unique identifier for this point
    pub id: String,

    /// The embedding vector
    pub vector: Vec<f32>,

    /// Arbitrary metadata payload
    pub payload: serde_json::Value,
}

impl VectorPoint {
    pub fn new(id: String, vector: Vec<f32>, payload: serde_json::Value) -> Self {
        Self { id, vector, payload }
    }
}

/// Result of a similarity search
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// ID of the matching point
    pub id: String,

    /// Similarity score (higher = more similar)
    pub score: f32,

    /// Metadata payload
    pub payload: serde_json::Value,
}

/// Distance metric for similarity calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Distance {
    /// Cosine distance (1 - cosine similarity)
    Cosine,

    /// Euclidean (L2) distance
    Euclidean,

    /// Dot product (inner product)
    DotProduct,
}

/// Metadata filtering for searches
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// All conditions must match (AND)
    pub must: Vec<Condition>,

    /// At least one condition must match (OR)
    pub should: Vec<Condition>,

    /// None of these conditions should match (NOT)
    pub must_not: Vec<Condition>,
}

/// A single filter condition
#[derive(Debug, Clone)]
pub enum Condition {
    /// Match exact value
    Match {
        key: String,
        value: serde_json::Value,
    },

    /// Match range of values
    Range {
        key: String,
        gte: Option<f64>,
        lte: Option<f64>,
    },

    /// Check if key exists
    HasKey(String),
}

/// Result of scrolling through points
#[derive(Debug, Clone)]
pub struct ScrollResult {
    /// Retrieved points
    pub points: Vec<VectorPoint>,

    /// Offset for next page (None if no more results)
    pub next_offset: Option<String>,
}

impl ScrollResult {
    pub fn new(points: Vec<VectorPoint>, next_offset: Option<String>) -> Self {
        Self { points, next_offset }
    }
}
