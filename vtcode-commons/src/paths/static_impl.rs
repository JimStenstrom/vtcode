use std::path::{Path, PathBuf};

use super::WorkspacePaths;

/// Reference implementation of [`WorkspacePaths`] backed by static [`PathBuf`]s.
///
/// This is useful for adopters who want to drive the extracted crates from an
/// existing application without wiring additional indirection layers. The
/// implementation is intentionally straightforward: callers provide the root
/// workspace directory and configuration path up front and can optionally
/// supply cache or telemetry directories.
#[derive(Debug, Clone)]
pub struct StaticWorkspacePaths {
    root: PathBuf,
    config: PathBuf,
    cache: Option<PathBuf>,
    telemetry: Option<PathBuf>,
}

impl StaticWorkspacePaths {
    /// Creates a new [`StaticWorkspacePaths`] with the required workspace and
    /// configuration directories.
    pub fn new(root: impl Into<PathBuf>, config: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            config: config.into(),
            cache: None,
            telemetry: None,
        }
    }

    /// Configures an optional cache directory used by the consumer.
    pub fn with_cache_dir(mut self, cache: impl Into<PathBuf>) -> Self {
        self.cache = Some(cache.into());
        self
    }

    /// Configures an optional telemetry directory used by the consumer.
    pub fn with_telemetry_dir(mut self, telemetry: impl Into<PathBuf>) -> Self {
        self.telemetry = Some(telemetry.into());
        self
    }
}

impl WorkspacePaths for StaticWorkspacePaths {
    fn workspace_root(&self) -> &Path {
        &self.root
    }

    fn config_dir(&self) -> PathBuf {
        self.config.clone()
    }

    fn cache_dir(&self) -> Option<PathBuf> {
        self.cache.clone()
    }

    fn telemetry_dir(&self) -> Option<PathBuf> {
        self.telemetry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn static_paths_exposes_optional_directories() {
        let paths = StaticWorkspacePaths::new("/tmp/work", "/tmp/work/config")
            .with_cache_dir("/tmp/work/cache")
            .with_telemetry_dir("/tmp/work/telemetry");

        assert_eq!(paths.workspace_root(), Path::new("/tmp/work"));
        assert_eq!(paths.config_dir(), PathBuf::from("/tmp/work/config"));
        assert_eq!(paths.cache_dir(), Some(PathBuf::from("/tmp/work/cache")));
        assert_eq!(
            paths.telemetry_dir(),
            Some(PathBuf::from("/tmp/work/telemetry"))
        );
    }
}
