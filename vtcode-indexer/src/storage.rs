//! Index storage backends

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::FileIndex;

/// Persistence backend for [`SimpleIndexer`](crate::SimpleIndexer).
pub trait IndexStorage: Send + Sync {
    /// Prepare any directories or resources required for persistence.
    fn init(&self, index_dir: &Path) -> Result<()>;

    /// Persist an indexed file entry.
    fn persist(&self, index_dir: &Path, entry: &FileIndex) -> Result<()>;
}

/// Markdown-backed [`IndexStorage`] implementation.
#[derive(Debug, Default, Clone)]
pub struct MarkdownIndexStorage;

impl IndexStorage for MarkdownIndexStorage {
    fn init(&self, index_dir: &Path) -> Result<()> {
        fs::create_dir_all(index_dir)?;
        Ok(())
    }

    fn persist(&self, index_dir: &Path, entry: &FileIndex) -> Result<()> {
        let file_name = format!("{}.md", calculate_hash(&entry.path));
        let index_path = index_dir.join(file_name);

        let markdown = format!(
            "# File Index: {}\n\n\
            - **Path**: {}\n\
            - **Hash**: {}\n\
            - **Modified**: {}\n\
            - **Size**: {} bytes\n\
            - **Language**: {}\n\
            - **Tags**: {}\n\n",
            entry.path,
            entry.path,
            entry.hash,
            entry.modified,
            entry.size,
            entry.language,
            entry.tags.join(", ")
        );

        fs::write(index_path, markdown)?;
        Ok(())
    }
}

fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
