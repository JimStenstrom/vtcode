//! Safety utilities for workspace operations
//!
//! This module provides safety validations for file operations, path validation,
//! and workspace boundary checking to ensure secure and controlled operations.
//!
//! ## Core Traits
//!
//! - [`PathValidator`]: Validates paths are safe and within allowed boundaries
//! - [`WorkspaceBoundary`]: Checks if operations respect workspace boundaries
//!
//! ## Modules
//!
//! - [`validation`]: Path validation and sanitization functions
//! - [`gitignore`]: Gitignore pattern matching and file exclusion

pub mod gitignore;
pub mod validation;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Trait for validating paths are safe and within allowed boundaries
///
/// Implementors provide custom logic for determining if a path should be
/// allowed for read/write operations based on workspace boundaries,
/// security policies, or other constraints.
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_commons::safety::PathValidator;
/// use std::path::Path;
///
/// struct MyValidator {
///     workspace_root: PathBuf,
/// }
///
/// impl PathValidator for MyValidator {
///     fn validate_path(&self, path: &Path) -> anyhow::Result<PathBuf> {
///         // Custom validation logic
///         if path.starts_with(&self.workspace_root) {
///             Ok(path.to_path_buf())
///         } else {
///             Err(anyhow::anyhow!("Path outside workspace"))
///         }
///     }
/// }
/// ```
pub trait PathValidator {
    /// Validates a path and returns a canonicalized safe path if valid
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The validated, canonicalized path
    /// * `Err(_)` - If the path is invalid or unsafe
    fn validate_path(&self, path: &Path) -> Result<PathBuf>;

    /// Check if a path is within allowed boundaries without canonicalizing
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` if the path is within boundaries
    /// * `false` otherwise
    fn is_path_allowed(&self, path: &Path) -> bool {
        self.validate_path(path).is_ok()
    }

    /// Validate multiple paths at once
    ///
    /// # Arguments
    ///
    /// * `paths` - Iterator of paths to validate
    ///
    /// # Returns
    ///
    /// A vector of successfully validated paths. Invalid paths are filtered out.
    fn validate_paths<'a, I>(&self, paths: I) -> Vec<PathBuf>
    where
        I: IntoIterator<Item = &'a Path>,
    {
        paths
            .into_iter()
            .filter_map(|p| self.validate_path(p).ok())
            .collect()
    }
}

/// Trait for checking workspace boundaries
///
/// Implementors define what constitutes a workspace boundary and provide
/// methods to check if paths, operations, or resources are within those
/// boundaries.
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_commons::safety::WorkspaceBoundary;
/// use std::path::{Path, PathBuf};
///
/// struct WorkspaceChecker {
///     root: PathBuf,
/// }
///
/// impl WorkspaceBoundary for WorkspaceChecker {
///     fn workspace_root(&self) -> &Path {
///         &self.root
///     }
///
///     fn is_within_workspace(&self, path: &Path) -> bool {
///         path.starts_with(&self.root)
///     }
/// }
/// ```
pub trait WorkspaceBoundary {
    /// Returns the root directory of the workspace
    fn workspace_root(&self) -> &Path;

    /// Check if a path is within the workspace boundaries
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` if the path is within the workspace
    /// * `false` otherwise
    fn is_within_workspace(&self, path: &Path) -> bool;

    /// Resolve a path relative to the workspace root
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to workspace root
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The absolute path within workspace
    /// * `Err(_)` - If the path would escape the workspace
    fn resolve_workspace_path(&self, relative_path: &Path) -> Result<PathBuf> {
        let absolute = self.workspace_root().join(relative_path);

        if self.is_within_workspace(&absolute) {
            Ok(absolute)
        } else {
            Err(anyhow::anyhow!(
                "Path '{}' would escape workspace boundaries",
                relative_path.display()
            ))
        }
    }

    /// Get a path relative to the workspace root
    ///
    /// # Arguments
    ///
    /// * `absolute_path` - An absolute path to make relative
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The path relative to workspace root
    /// * `Err(_)` - If the path is not within the workspace
    fn make_relative(&self, absolute_path: &Path) -> Result<PathBuf> {
        if !self.is_within_workspace(absolute_path) {
            return Err(anyhow::anyhow!(
                "Path '{}' is not within workspace",
                absolute_path.display()
            ));
        }

        absolute_path
            .strip_prefix(self.workspace_root())
            .map(|p| p.to_path_buf())
            .map_err(|e| anyhow::anyhow!("Failed to make path relative: {}", e))
    }
}

/// Simple implementation of PathValidator that checks workspace boundaries
#[derive(Debug, Clone)]
pub struct SimplePathValidator {
    workspace_root: PathBuf,
}

impl SimplePathValidator {
    /// Create a new SimplePathValidator with the given workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl PathValidator for SimplePathValidator {
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Resolve the path (handle relative paths)
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Canonicalize to resolve symlinks and ".." components
        let canonical = resolved
            .canonicalize()
            .or_else(|_| Ok::<_, anyhow::Error>(resolved.clone()))?;

        // Check if the canonical path is within workspace
        if !canonical.starts_with(&self.workspace_root) {
            return Err(anyhow::anyhow!(
                "Path '{}' is outside workspace boundaries",
                path.display()
            ));
        }

        Ok(canonical)
    }
}

impl WorkspaceBoundary for SimplePathValidator {
    fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    fn is_within_workspace(&self, path: &Path) -> bool {
        path.starts_with(&self.workspace_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_validator() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let validator = SimplePathValidator::new(workspace.clone());

        // Create a test file
        let test_file = workspace.join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Valid path within workspace
        assert!(validator.validate_path(&test_file).is_ok());
        assert!(validator.is_path_allowed(&test_file));

        // Relative path should be resolved to workspace
        let relative = Path::new("test.txt");
        assert!(validator.validate_path(relative).is_ok());
    }

    #[test]
    fn test_workspace_boundary() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let validator = SimplePathValidator::new(workspace.clone());

        // Path within workspace
        let inside = workspace.join("file.txt");
        assert!(validator.is_within_workspace(&inside));

        // Path outside workspace
        let outside = Path::new("/tmp/outside.txt");
        assert!(!validator.is_within_workspace(outside));
    }

    #[test]
    fn test_resolve_workspace_path() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let validator = SimplePathValidator::new(workspace.clone());

        // Resolve relative path
        let relative = Path::new("subdir/file.txt");
        let resolved = validator.resolve_workspace_path(relative).unwrap();
        assert_eq!(resolved, workspace.join("subdir/file.txt"));
    }

    #[test]
    fn test_make_relative() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let validator = SimplePathValidator::new(workspace.clone());

        // Make absolute path relative
        let absolute = workspace.join("subdir/file.txt");
        let relative = validator.make_relative(&absolute).unwrap();
        assert_eq!(relative, Path::new("subdir/file.txt"));
    }
}
