//! Test infrastructure

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

use crate::config::SimpleIndexerConfig;
use crate::filter::{ConfigTraversalFilter, TraversalFilter};
use crate::indexer::SimpleIndexer;
use crate::storage::IndexStorage;
use crate::FileIndex;

#[test]
fn skips_hidden_directories_by_default() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path();
    let hidden_dir = workspace.join(".private");
    fs::create_dir_all(&hidden_dir)?;
    fs::write(hidden_dir.join("secret.txt"), "classified")?;

    let visible_dir = workspace.join("src");
    fs::create_dir_all(&visible_dir)?;
    fs::write(visible_dir.join("lib.rs"), "fn main() {}")?;

    let mut indexer = SimpleIndexer::new(workspace.to_path_buf());
    indexer.init()?;
    indexer.index_directory(workspace)?;

    assert!(indexer.find_files("secret\\.txt$")?.is_empty());
    assert!(!indexer.find_files("lib\\.rs$")?.is_empty());

    Ok(())
}

#[test]
fn can_include_hidden_directories_when_configured() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path();
    let hidden_dir = workspace.join(".cache");
    fs::create_dir_all(&hidden_dir)?;
    fs::write(hidden_dir.join("data.log"), "details")?;

    let config = SimpleIndexerConfig::new(workspace.to_path_buf()).ignore_hidden(false);
    let mut indexer = SimpleIndexer::with_config(config);
    indexer.init()?;
    indexer.index_directory(workspace)?;

    let results = indexer.find_files("data\\.log$")?;
    assert_eq!(results.len(), 1);

    Ok(())
}

#[test]
fn supports_custom_storage_backends() -> Result<()> {
    #[derive(Clone, Default)]
    struct MemoryStorage {
        records: Arc<Mutex<Vec<FileIndex>>>,
    }

    impl MemoryStorage {
        fn new(records: Arc<Mutex<Vec<FileIndex>>>) -> Self {
            Self { records }
        }
    }

    impl IndexStorage for MemoryStorage {
        fn init(&self, _index_dir: &Path) -> Result<()> {
            Ok(())
        }

        fn persist(&self, _index_dir: &Path, entry: &FileIndex) -> Result<()> {
            let mut guard = self.records.lock().expect("lock poisoned");
            guard.push(entry.clone());
            Ok(())
        }
    }

    let temp = tempdir()?;
    let workspace = temp.path();
    fs::write(workspace.join("notes.txt"), "remember this")?;

    let records: Arc<Mutex<Vec<FileIndex>>> = Arc::new(Mutex::new(Vec::new()));
    let storage = MemoryStorage::new(records.clone());

    let config = SimpleIndexerConfig::new(workspace.to_path_buf());
    let mut indexer = SimpleIndexer::with_config(config).with_storage(Arc::new(storage));
    indexer.init()?;
    indexer.index_directory(workspace)?;

    let entries = records.lock().expect("lock poisoned");
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].path,
        workspace.join("notes.txt").to_string_lossy().to_string()
    );

    Ok(())
}

#[test]
fn custom_filters_can_skip_files() -> Result<()> {
    #[derive(Default)]
    struct SkipRustFilter {
        inner: ConfigTraversalFilter,
    }

    impl TraversalFilter for SkipRustFilter {
        fn should_descend(&self, path: &Path, config: &SimpleIndexerConfig) -> bool {
            self.inner.should_descend(path, config)
        }

        fn should_index_file(&self, path: &Path, config: &SimpleIndexerConfig) -> bool {
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            {
                return false;
            }

            self.inner.should_index_file(path, config)
        }
    }

    let temp = tempdir()?;
    let workspace = temp.path();
    fs::write(workspace.join("lib.rs"), "fn main() {}")?;
    fs::write(workspace.join("README.md"), "# Notes")?;

    let config = SimpleIndexerConfig::new(workspace.to_path_buf());
    let mut indexer =
        SimpleIndexer::with_config(config).with_filter(Arc::new(SkipRustFilter::default()));
    indexer.init()?;
    indexer.index_directory(workspace)?;

    assert!(indexer.find_files("lib\\.rs$")?.is_empty());
    assert!(!indexer.find_files("README\\.md$")?.is_empty());

    Ok(())
}
