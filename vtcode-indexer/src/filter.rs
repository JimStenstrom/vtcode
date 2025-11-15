//! Directory traversal filters

use std::path::{Path, PathBuf};

use crate::config::SimpleIndexerConfig;

/// Directory traversal filter hook for [`SimpleIndexer`](crate::SimpleIndexer).
pub trait TraversalFilter: Send + Sync {
    /// Determine if the indexer should descend into the provided directory.
    fn should_descend(&self, path: &Path, config: &SimpleIndexerConfig) -> bool;

    /// Determine if the indexer should process the provided file.
    fn should_index_file(&self, path: &Path, config: &SimpleIndexerConfig) -> bool;
}

/// Default traversal filter powered by [`SimpleIndexerConfig`].
#[derive(Debug, Default, Clone)]
pub struct ConfigTraversalFilter;

impl TraversalFilter for ConfigTraversalFilter {
    fn should_descend(&self, path: &Path, config: &SimpleIndexerConfig) -> bool {
        !should_skip_dir(path, config)
    }

    fn should_index_file(&self, path: &Path, config: &SimpleIndexerConfig) -> bool {
        if !path.is_file() {
            return false;
        }

        // Skip hidden files when configured.
        if config.ignore_hidden && is_path_hidden(path) {
            return false;
        }

        // Always skip known sensitive files regardless of config.
        if let Some(file_name) = get_file_name_str(path) {
            let is_sensitive = matches!(
                file_name,
                ".env"
                    | ".env.local"
                    | ".env.production"
                    | ".env.development"
                    | ".env.test"
                    | ".git"
                    | ".gitignore"
                    | ".DS_Store"
            ) || file_name.starts_with(".env.");
            if is_sensitive {
                return false;
            }
        }

        true
    }
}

/// Helper to check if a path starts with any path in a collection.
pub(crate) fn path_starts_with_any(path: &Path, paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| path.starts_with(p))
}

/// Helper to check if a path component (file or directory name) is hidden.
pub(crate) fn is_path_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name_str| name_str.starts_with('.'))
}

/// Helper to extract file name as a string.
fn get_file_name_str(path: &Path) -> Option<&str> {
    path.file_name().and_then(|n| n.to_str())
}

fn should_skip_dir(path: &Path, config: &SimpleIndexerConfig) -> bool {
    if path_starts_with_any(path, &config.allowed_dirs) {
        return false;
    }

    if path_starts_with_any(path, &config.excluded_dirs) {
        return true;
    }

    if config.ignore_hidden && is_path_hidden(path) {
        return true;
    }

    false
}
