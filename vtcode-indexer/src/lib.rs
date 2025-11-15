//! Workspace-friendly file indexer extracted from VTCode.
//!
//! `vtcode-indexer` offers a lightweight alternative to heavyweight
//! search/indexing stacks. It recursively walks a workspace, computes
//! hashes, and stores per-file metadata in Markdown-friendly summaries
//! so changes remain easy to audit in git.

use serde::{Deserialize, Serialize};

// Module declarations
pub mod config;
pub mod filter;
pub mod indexer;
pub mod search;
pub mod storage;
mod traversal;

// Tests
#[cfg(test)]
mod tests;

// Re-export primary types for convenience
pub use config::SimpleIndexerConfig;
pub use filter::{ConfigTraversalFilter, TraversalFilter};
pub use indexer::SimpleIndexer;
pub use storage::{IndexStorage, MarkdownIndexStorage};

/// Simple file index entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    /// File path.
    pub path: String,
    /// File content hash for change detection.
    pub hash: String,
    /// Last modified timestamp.
    pub modified: u64,
    /// File size.
    pub size: u64,
    /// Language/extension.
    pub language: String,
    /// Simple tags.
    pub tags: Vec<String>,
}

/// Simple search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub matches: Vec<String>,
}
