//! Core indexer implementation

use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use crate::config::SimpleIndexerConfig;
use crate::filter::{path_starts_with_any, ConfigTraversalFilter, TraversalFilter};
use crate::storage::{IndexStorage, MarkdownIndexStorage};
use crate::{FileIndex, SearchResult};

/// Simple file indexer.
pub struct SimpleIndexer {
    pub(crate) config: SimpleIndexerConfig,
    pub(crate) index_cache: HashMap<String, FileIndex>,
    pub(crate) storage: Arc<dyn IndexStorage>,
    pub(crate) filter: Arc<dyn TraversalFilter>,
}

impl SimpleIndexer {
    /// Create a new simple indexer with default VTCode paths.
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self::with_components(
            SimpleIndexerConfig::new(workspace_root),
            Arc::new(MarkdownIndexStorage),
            Arc::new(ConfigTraversalFilter),
        )
    }

    /// Create a simple indexer with the provided configuration.
    pub fn with_config(config: SimpleIndexerConfig) -> Self {
        Self::with_components(
            config,
            Arc::new(MarkdownIndexStorage),
            Arc::new(ConfigTraversalFilter),
        )
    }

    /// Create a new simple indexer using a custom index directory.
    pub fn with_index_dir(
        workspace_root: std::path::PathBuf,
        index_dir: std::path::PathBuf,
    ) -> Self {
        let config = SimpleIndexerConfig::new(workspace_root).with_index_dir(index_dir);
        Self::with_config(config)
    }

    /// Create an indexer with explicit storage and traversal filter implementations.
    pub fn with_components(
        config: SimpleIndexerConfig,
        storage: Arc<dyn IndexStorage>,
        filter: Arc<dyn TraversalFilter>,
    ) -> Self {
        Self {
            config,
            index_cache: HashMap::new(),
            storage,
            filter,
        }
    }

    /// Replace the storage backend used to persist index entries.
    pub fn with_storage(self, storage: Arc<dyn IndexStorage>) -> Self {
        Self { storage, ..self }
    }

    /// Replace the traversal filter used to decide which files and directories are indexed.
    pub fn with_filter(self, filter: Arc<dyn TraversalFilter>) -> Self {
        Self { filter, ..self }
    }

    /// Initialize the index directory.
    pub fn init(&self) -> Result<()> {
        self.storage.init(self.config.index_dir())
    }

    /// Get the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        self.config.workspace_root()
    }

    /// Get the index directory used for persisted metadata.
    pub fn index_dir(&self) -> &Path {
        self.config.index_dir()
    }

    /// Index a single file.
    pub fn index_file(&mut self, file_path: &Path) -> Result<()> {
        if !file_path.exists() || !self.filter.should_index_file(file_path, &self.config) {
            return Ok(());
        }

        let content = match fs::read_to_string(file_path) {
            Ok(text) => text,
            Err(err) => {
                if err.kind() == ErrorKind::InvalidData {
                    return Ok(());
                }
                return Err(err.into());
            }
        };
        let hash = calculate_hash(&content);
        let modified = self.get_modified_time(file_path)?;
        let size = content.len() as u64;
        let language = self.detect_language(file_path);

        let index = FileIndex {
            path: file_path.to_string_lossy().to_string(),
            hash,
            modified,
            size,
            language,
            tags: vec![],
        };

        self.index_cache
            .insert(file_path.to_string_lossy().to_string(), index.clone());

        self.storage.persist(self.config.index_dir(), &index)?;

        Ok(())
    }

    /// Index all files in directory recursively.
    /// Respects .gitignore, .ignore, and other ignore files.
    /// SECURITY: By default skips hidden files and sensitive data (.env, .git, etc.)
    pub fn index_directory(&mut self, dir_path: &Path) -> Result<()> {
        let walker = WalkBuilder::new(dir_path)
            .hidden(self.config.ignore_hidden) // Skip hidden files based on config
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .ignore(true) // Respect .ignore files
            .parents(true) // Check parent directories for ignore files
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Only index files, not directories
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                // Additional check: skip if in excluded dirs
                let should_skip = path_starts_with_any(path, &self.config.excluded_dirs);

                if !should_skip && self.filter.should_index_file(path, &self.config) {
                    self.index_file(path)?;
                }
            }
        }

        Ok(())
    }

    /// Search files using regex pattern.
    pub fn search(&self, pattern: &str, path_filter: Option<&str>) -> Result<Vec<SearchResult>> {
        crate::search::search(&self.index_cache, pattern, path_filter)
    }

    /// Find files by name pattern.
    pub fn find_files(&self, pattern: &str) -> Result<Vec<String>> {
        crate::search::find_files(&self.index_cache, pattern)
    }

    /// Get all indexed files without pattern matching.
    /// This is more efficient than using find_files(".*").
    pub fn all_files(&self) -> Vec<String> {
        crate::search::all_files(&self.index_cache)
    }

    /// Get file content with line numbers.
    pub fn get_file_content(
        &self,
        file_path: &str,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<String> {
        crate::search::get_file_content(file_path, start_line, end_line)
    }

    /// List files in directory (like ls).
    pub fn list_files(&self, dir_path: &str, show_hidden: bool) -> Result<Vec<String>> {
        crate::search::list_files(dir_path, show_hidden)
    }

    /// Grep-like search (like grep command).
    pub fn grep(&self, pattern: &str, file_pattern: Option<&str>) -> Result<Vec<SearchResult>> {
        crate::search::grep(&self.index_cache, pattern, file_pattern)
    }

    fn get_modified_time(&self, file_path: &Path) -> Result<u64> {
        let metadata = fs::metadata(file_path)?;
        let modified = metadata.modified()?;
        Ok(modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
    }

    fn detect_language(&self, file_path: &Path) -> String {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

impl Clone for SimpleIndexer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            index_cache: self.index_cache.clone(),
            storage: self.storage.clone(),
            filter: self.filter.clone(),
        }
    }
}

fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
