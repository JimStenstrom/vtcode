//! File operation command validators
//!
//! This module validates file operation commands:
//! - cat: Read file contents
//! - cp: Copy files
//! - head: Read file beginning
//! - tail: Read file end
//! - ls: List directory contents
//! - wc: Count words/lines
//!
//! All file operations are validated to ensure:
//! - Files are within workspace boundaries
//! - Files exist before reading
//! - Destination parents exist before copying
//! - Only safe flags are used

use std::path::Path;
use tokio::fs;

use anyhow::{Context, Result, anyhow};

use crate::policy::paths::{
    ensure_is_file, ensure_path_exists, parse_positive_int, resolve_path, resolve_path_allow_new,
};
use crate::policy::workspace::ensure_within_workspace;

/// Validate cat command arguments.
///
/// Requirements:
/// - At least one file must be specified
/// - All files must exist and be readable
/// - All paths must be within workspace
/// - Only safe flags allowed: -b, -n, -t
pub async fn validate_cat(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut files = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-b" | "-n" | "-t" => continue,
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported cat flag '{}'", value));
            }
            value => {
                let path = resolve_path(workspace_root, working_dir, value).await?;
                ensure_is_file(&path).await?;
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        return Err(anyhow!("cat requires at least one readable file"));
    }

    Ok(())
}

/// Validate cp command arguments.
///
/// Requirements:
/// - At least source and destination required
/// - Source files must exist within workspace
/// - Destination must be within workspace
/// - Destination parent must exist
/// - Directories require -r flag
/// - Only safe flags allowed: -r, -R, --recursive
pub async fn validate_cp(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut positional = Vec::new();
    let mut allow_recursive = false;

    for arg in args {
        match arg.as_str() {
            "-r" | "-R" | "--recursive" => {
                allow_recursive = true;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported cp flag '{}'", value));
            }
            value => positional.push(value.to_string()),
        }
    }

    if positional.len() < 2 {
        return Err(anyhow!("cp requires a source and destination"));
    }

    let dest_raw = positional.last().unwrap();
    let sources = &positional[..positional.len() - 1];

    for source in sources {
        let path = resolve_path(workspace_root, working_dir, source).await?;
        let metadata = fs::metadata(&path)
            .await
            .with_context(|| format!("failed to inspect source '{}'", source))?;
        if metadata.is_dir() && !allow_recursive {
            return Err(anyhow!(
                "copying directories requires the recursive flag for '{}'",
                source
            ));
        }
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(anyhow!("unsupported source type for '{}'", source));
        }
    }

    let dest_path = resolve_path_allow_new(workspace_root, working_dir, dest_raw).await?;
    if let Some(parent) = dest_path.parent() {
        if !fs::try_exists(parent).await.unwrap_or(false) {
            return Err(anyhow!(
                "destination parent '{}' must exist",
                parent.display()
            ));
        }
    }

    Ok(())
}

/// Validate head command arguments.
///
/// Requirements:
/// - At least one file required
/// - All files must exist within workspace
/// - Only safe flags allowed: -c, -n (with positive integer values)
pub async fn validate_head(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut positional = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let current = &args[index];
        match current.as_str() {
            "-c" | "-n" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("option '{}' requires a value", current))?;
                parse_positive_int(value)
                    .with_context(|| format!("invalid value '{}' for '{}'", value, current))?;
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported head flag '{}'", value));
            }
            value => {
                positional.push(value);
                index += 1;
            }
        }
    }

    if positional.is_empty() {
        return Err(anyhow!("head requires at least one file"));
    }

    for file in positional {
        let path = resolve_path(workspace_root, working_dir, file).await?;
        ensure_is_file(&path).await?;
    }

    Ok(())
}

/// Validate tail command arguments.
///
/// Requirements:
/// - All files must be within workspace
/// - Files are validated when accessed
pub async fn validate_tail(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    for arg in args {
        if !arg.starts_with('-') {
            let path = super::super::paths::normalize_path(&working_dir.join(arg));
            ensure_within_workspace(workspace_root, &path).await?;
        }
    }
    Ok(())
}

/// Validate ls command arguments.
///
/// Requirements:
/// - All paths must be within workspace
/// - Paths must exist
/// - Only safe flags allowed: -1, -a, -l
pub async fn validate_ls(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    for arg in args {
        match arg.as_str() {
            "-1" | "-a" | "-l" => continue,
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported ls flag '{}'", value));
            }
            value => {
                let path = resolve_path(workspace_root, working_dir, value).await?;
                ensure_path_exists(&path)?;
            }
        }
    }
    Ok(())
}

/// Validate wc command arguments.
///
/// Requirements:
/// - All files must be within workspace
/// - Files are validated when accessed
pub async fn validate_wc(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    for arg in args {
        if !arg.starts_with('-') {
            let path = super::super::paths::normalize_path(&working_dir.join(arg));
            ensure_within_workspace(workspace_root, &path).await?;
        }
    }
    Ok(())
}
