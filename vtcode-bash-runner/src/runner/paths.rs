//! Path resolution and workspace validation

use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve a path relative to working directory
pub fn resolve_path(working_dir: &Path, raw: &str) -> PathBuf {
    let candidate = Path::new(raw);
    let joined = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        working_dir.join(candidate)
    };
    joined.clean()
}

/// Resolve and validate existing path
pub fn resolve_existing_path(
    workspace_root: &Path,
    working_dir: &Path,
    raw: &str,
) -> Result<PathBuf> {
    let path = resolve_path(working_dir, raw);
    if !path.exists() {
        bail!("path `{}` does not exist", path.display());
    }

    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize `{}`", path.display()))?;

    ensure_within_workspace(workspace_root, &canonical)?;
    Ok(canonical)
}

/// Ensure path is within workspace boundaries
pub fn ensure_within_workspace(workspace_root: &Path, candidate: &Path) -> Result<()> {
    if !candidate.starts_with(workspace_root) {
        bail!(
            "path `{}` escapes workspace root `{}`",
            candidate.display(),
            workspace_root.display()
        );
    }
    Ok(())
}

/// Ensure mutation target is within workspace (handles symlinks)
pub fn ensure_mutation_target_within_workspace(
    workspace_root: &Path,
    working_dir: &Path,
    candidate: &Path,
) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(candidate)
        && metadata.file_type().is_symlink()
    {
        let canonical = candidate
            .canonicalize()
            .with_context(|| format!("failed to canonicalize `{}`", candidate.display()))?;
        return ensure_within_workspace(workspace_root, &canonical);
    }

    if candidate.exists() {
        let canonical = candidate
            .canonicalize()
            .with_context(|| format!("failed to canonicalize `{}`", candidate.display()))?;
        ensure_within_workspace(workspace_root, &canonical)
    } else {
        let parent = canonicalize_existing_parent(working_dir, candidate)?;
        ensure_within_workspace(workspace_root, &parent)
    }
}

fn canonicalize_existing_parent(working_dir: &Path, candidate: &Path) -> Result<PathBuf> {
    let mut current = candidate.parent();
    while let Some(path) = current {
        if path.exists() {
            return path
                .canonicalize()
                .with_context(|| format!("failed to canonicalize `{}`", path.display()));
        }
        current = path.parent();
    }

    Ok(working_dir.to_path_buf())
}
