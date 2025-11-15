//! Path validation and normalization
//!
//! This module provides path validation functions to ensure all file paths are:
//! - Within workspace boundaries
//! - Normalized to prevent traversal attacks
//! - Validated before operations
//!
//! # Security Features
//!
//! - Path normalization to resolve ../ and ./
//! - Workspace boundary enforcement
//! - Symlink traversal protection
//! - Path existence validation

use std::env;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

use anyhow::{Context, Result, anyhow};

/// Resolve a path relative to working directory, requiring it to exist.
///
/// # Arguments
///
/// * `workspace_root` - The root workspace directory
/// * `working_dir` - The current working directory
/// * `value` - The path to resolve (relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` - The normalized absolute path within workspace
/// * `Err(_)` - If path doesn't exist or escapes workspace
pub async fn resolve_path(
    workspace_root: &Path,
    working_dir: &Path,
    value: &str,
) -> Result<PathBuf> {
    let base = build_candidate_path(workspace_root, working_dir, value).await?;
    if !fs::try_exists(&base).await.unwrap_or(false) {
        return Err(anyhow!("path '{}' does not exist", value));
    }
    if !base.starts_with(workspace_root) {
        return Err(anyhow!("path '{}' is outside the workspace root", value));
    }
    Ok(base)
}

/// Resolve path allowing non-existent files (for create operations).
///
/// # Arguments
///
/// * `workspace_root` - The root workspace directory
/// * `working_dir` - The current working directory
/// * `value` - The path to resolve (relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` - The normalized absolute path within workspace
/// * `Err(_)` - If path escapes workspace
pub async fn resolve_path_allow_new(
    workspace_root: &Path,
    working_dir: &Path,
    value: &str,
) -> Result<PathBuf> {
    let candidate = build_candidate_path(workspace_root, working_dir, value).await?;
    if !candidate.starts_with(workspace_root) {
        return Err(anyhow!("path '{}' is outside the workspace root", value));
    }
    Ok(candidate)
}

/// Resolve path allowing directories (for search operations).
///
/// # Arguments
///
/// * `workspace_root` - The root workspace directory
/// * `working_dir` - The current working directory
/// * `value` - The path to resolve (relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` - The normalized absolute path within workspace
/// * `Err(_)` - If path escapes workspace
pub async fn resolve_path_allow_dir(
    workspace_root: &Path,
    working_dir: &Path,
    value: &str,
) -> Result<PathBuf> {
    let candidate = build_candidate_path(workspace_root, working_dir, value).await?;
    if !candidate.starts_with(workspace_root) {
        return Err(anyhow!("path '{}' is outside the workspace root", value));
    }
    Ok(candidate)
}

/// Build and validate a candidate path.
///
/// This function:
/// 1. Normalizes workspace root and working directory
/// 2. Resolves the path (absolute or relative)
/// 3. Normalizes the result to eliminate ../ and ./
/// 4. Validates workspace boundaries
/// 5. Checks for symlink escapes
pub async fn build_candidate_path(
    workspace_root: &Path,
    working_dir: &Path,
    value: &str,
) -> Result<PathBuf> {
    let normalized_root = normalize_workspace_root(workspace_root)?;
    let normalized_working = normalize_path(working_dir);
    let raw_path = Path::new(value);
    let candidate = if raw_path.is_absolute() {
        normalize_path(raw_path)
    } else {
        normalize_path(&normalized_working.join(raw_path))
    };

    if !candidate.starts_with(&normalized_root) {
        return Err(anyhow!("path '{}' escapes the workspace root", value));
    }
    super::workspace::ensure_within_workspace(&normalized_root, &candidate).await?;
    Ok(candidate)
}

/// Normalize a workspace root path to an absolute path.
///
/// If the workspace root is relative, it's resolved against the current working directory.
pub fn normalize_workspace_root(workspace_root: &Path) -> Result<PathBuf> {
    if workspace_root.is_absolute() {
        return Ok(normalize_path(workspace_root));
    }

    let cwd = env::current_dir().context("failed to resolve current working directory")?;
    Ok(normalize_path(&cwd.join(workspace_root)))
}

/// Normalize a path by resolving ../ and ./ components.
///
/// This function performs lexical path normalization without accessing the filesystem.
/// It eliminates:
/// - Current directory components (.)
/// - Parent directory components (..)
/// - Redundant separators
///
/// # Example
///
/// ```ignore
/// let path = Path::new("/workspace/project/../src/./main.rs");
/// let normalized = normalize_path(path);
/// assert_eq!(normalized, PathBuf::from("/workspace/src/main.rs"));
/// ```
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

/// Ensure a path exists.
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `Ok(())` - If the path exists
/// * `Err(_)` - If the path doesn't exist
pub fn ensure_path_exists(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(anyhow!("path '{}' does not exist", path.display()))
    }
}

/// Ensure a path is a file (not a directory or other type).
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `Ok(())` - If the path is a file
/// * `Err(_)` - If the path is not a file or doesn't exist
pub async fn ensure_is_file(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)
        .await
        .with_context(|| format!("failed to inspect '{}'", path.display()))?;
    if metadata.is_file() {
        Ok(())
    } else {
        Err(anyhow!("'{}' is not a file", path.display()))
    }
}

/// Parse a positive integer value.
///
/// Used for validating numeric arguments like line counts, depths, etc.
///
/// # Arguments
///
/// * `value` - The string to parse
///
/// # Returns
///
/// * `Ok(u64)` - The parsed positive integer
/// * `Err(_)` - If the value is not a positive integer
pub fn parse_positive_int(value: &str) -> Result<u64> {
    let parsed: u64 = value.parse()?;
    if parsed == 0 {
        return Err(anyhow!("value must be greater than zero"));
    }
    Ok(parsed)
}
