//! Indexer configuration

use std::path::{Path, PathBuf};

/// Configuration for [`SimpleIndexer`](crate::SimpleIndexer).
#[derive(Clone, Debug)]
pub struct SimpleIndexerConfig {
    pub(crate) workspace_root: PathBuf,
    pub(crate) index_dir: PathBuf,
    pub(crate) ignore_hidden: bool,
    pub(crate) excluded_dirs: Vec<PathBuf>,
    pub(crate) allowed_dirs: Vec<PathBuf>,
}

impl SimpleIndexerConfig {
    /// Builds a configuration using VTCode's legacy layout as defaults.
    pub fn new(workspace_root: PathBuf) -> Self {
        let index_dir = workspace_root.join(".vtcode").join("index");
        let vtcode_dir = workspace_root.join(".vtcode");
        let external_dir = vtcode_dir.join("external");

        let mut excluded_dirs = vec![
            index_dir.clone(),
            vtcode_dir,
            workspace_root.join("target"),
            workspace_root.join("node_modules"),
        ];

        excluded_dirs.dedup();

        Self {
            workspace_root,
            index_dir,
            ignore_hidden: true,
            excluded_dirs,
            allowed_dirs: vec![external_dir],
        }
    }

    /// Updates the index directory used for persisted metadata.
    pub fn with_index_dir(mut self, index_dir: impl Into<PathBuf>) -> Self {
        let index_dir = index_dir.into();
        self.index_dir = index_dir.clone();
        self.push_unique_excluded(index_dir);
        self
    }

    /// Adds an allowed directory that should be indexed even if hidden or inside an excluded parent.
    pub fn add_allowed_dir(mut self, path: impl Into<PathBuf>) -> Self {
        Self::push_unique(&mut self.allowed_dirs, path.into());
        self
    }

    /// Adds an additional excluded directory to skip during traversal.
    pub fn add_excluded_dir(mut self, path: impl Into<PathBuf>) -> Self {
        Self::push_unique(&mut self.excluded_dirs, path.into());
        self
    }

    /// Toggles whether hidden directories (prefix `.`) are ignored.
    pub fn ignore_hidden(mut self, ignore_hidden: bool) -> Self {
        self.ignore_hidden = ignore_hidden;
        self
    }

    /// Workspace root accessor.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Index directory accessor.
    pub fn index_dir(&self) -> &Path {
        &self.index_dir
    }

    /// Helper to push a path to a vector only if it doesn't already exist.
    fn push_unique(vec: &mut Vec<PathBuf>, path: PathBuf) {
        if !vec.iter().any(|existing| existing == &path) {
            vec.push(path);
        }
    }

    fn push_unique_excluded(&mut self, path: PathBuf) {
        Self::push_unique(&mut self.excluded_dirs, path);
    }
}
