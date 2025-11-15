//! Workspace boundary enforcement
//!
//! This module provides functions to ensure operations stay within workspace boundaries.
//! It prevents:
//! - Path traversal attacks via ../
//! - Symlink escapes outside workspace
//! - Access to files outside the workspace root
//!
//! # Security Model
//!
//! All paths are validated to ensure they:
//! 1. Are within the workspace root after normalization
//! 2. Don't escape via symlinks
//! 3. Don't traverse through files as directories

use std::io;
use std::path::{Path, PathBuf};
use tokio::fs;

use anyhow::{Context, Result, anyhow};

use super::paths::normalize_path;

/// Ensure a path is within the workspace boundaries.
///
/// This function performs deep validation to prevent workspace escapes via:
/// - Symlinks that point outside the workspace
/// - Path components that resolve outside the workspace
/// - Invalid path traversal through files
///
/// # Arguments
///
/// * `normalized_root` - The normalized workspace root path
/// * `candidate` - The candidate path to validate
///
/// # Returns
///
/// * `Ok(())` - If the path is within workspace boundaries
/// * `Err(_)` - If the path escapes the workspace
///
/// # Security
///
/// This is a CRITICAL security function. It validates each path component
/// to detect symlink escapes and path traversal attacks.
pub async fn ensure_within_workspace(normalized_root: &Path, candidate: &Path) -> Result<()> {
    let canonical_root = fs::canonicalize(normalized_root).await.with_context(|| {
        format!(
            "failed to canonicalize workspace root '{}'",
            normalized_root.display()
        )
    })?;

    // Workspace root itself is always valid
    if normalized_root == candidate {
        return Ok(());
    }

    // Extract relative path
    let relative = candidate
        .strip_prefix(normalized_root)
        .map_err(|_| anyhow!("path '{}' escapes the workspace root", candidate.display()))?;

    // Validate each component to detect symlink escapes
    let mut prefix = normalized_root.to_path_buf();
    let mut components = relative.components().peekable();

    while let Some(component) = components.next() {
        prefix.push(component.as_os_str());

        // Get metadata for this component
        let metadata = match fs::symlink_metadata(&prefix).await {
            Ok(metadata) => metadata,
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    // Path doesn't exist yet - that's okay for new files
                    break;
                }
                return Err(error).with_context(|| {
                    format!("failed to inspect path component '{}'", prefix.display())
                });
            }
        };

        // Check symlinks - they could escape the workspace
        if metadata.file_type().is_symlink() {
            let resolved = fs::canonicalize(&prefix).await.with_context(|| {
                format!(
                    "failed to canonicalize path component '{}'",
                    prefix.display()
                )
            })?;
            if !resolved.starts_with(&canonical_root) {
                return Err(anyhow!(
                    "path '{}' escapes the workspace root via symlink '{}'",
                    candidate.display(),
                    prefix.display()
                ));
            }
        } else {
            // Even non-symlinks need validation - canonicalize resolves mount points etc.
            let resolved = fs::canonicalize(&prefix).await.with_context(|| {
                format!(
                    "failed to canonicalize path component '{}'",
                    prefix.display()
                )
            })?;
            if !resolved.starts_with(&canonical_root) {
                return Err(anyhow!(
                    "path '{}' escapes the workspace root via component '{}'",
                    candidate.display(),
                    prefix.display()
                ));
            }

            // Don't allow traversal through files
            if metadata.is_file() && components.peek().is_some() {
                return Err(anyhow!(
                    "path '{}' traverses through file component '{}'",
                    candidate.display(),
                    prefix.display()
                ));
            }
        }
    }

    Ok(())
}

/// Normalize and validate a working directory relative to the workspace root.
///
/// This function ensures the working directory is within workspace boundaries
/// and normalizes the path to prevent traversal attacks.
///
/// # Arguments
///
/// * `workspace_root` - The root workspace directory
/// * `working_dir` - The working directory (relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` with the normalized absolute path
/// * `Err(_)` if the path escapes the workspace
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_execution::policy::sanitize_working_dir;
/// use std::path::Path;
///
/// let workspace = Path::new("/workspace");
///
/// // Valid path
/// let dir = sanitize_working_dir(workspace, Some("./project")).await?;
/// assert!(dir.starts_with(workspace));
///
/// // Escape attempt - rejected
/// let result = sanitize_working_dir(workspace, Some("../../etc")).await;
/// assert!(result.is_err());
/// ```
pub async fn sanitize_working_dir(
    workspace_root: &Path,
    working_dir: Option<&str>,
) -> Result<PathBuf> {
    let normalized_root = super::paths::normalize_workspace_root(workspace_root)?;
    if let Some(dir) = working_dir {
        if dir.trim().is_empty() {
            return Ok(normalized_root);
        }
        let candidate = normalize_path(&normalized_root.join(dir));
        if !candidate.starts_with(&normalized_root) {
            return Err(anyhow!(
                "working directory '{}' escapes the workspace root",
                dir
            ));
        }
        ensure_within_workspace(&normalized_root, &candidate).await?;
        Ok(candidate)
    } else {
        Ok(normalized_root)
    }
}
