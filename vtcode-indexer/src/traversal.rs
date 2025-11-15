//! Directory traversal operations

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::SimpleIndexerConfig;
use crate::filter::{path_starts_with_any, TraversalFilter};

/// Walk directory recursively with custom callback.
#[allow(dead_code)]
pub(crate) fn walk_directory<F>(
    dir_path: &Path,
    config: &SimpleIndexerConfig,
    filter: &dyn TraversalFilter,
    callback: &mut F,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    if !dir_path.exists() {
        return Ok(());
    }

    walk_directory_internal(dir_path, config, filter, callback)
}

#[allow(dead_code)]
fn walk_directory_internal<F>(
    dir_path: &Path,
    config: &SimpleIndexerConfig,
    filter: &dyn TraversalFilter,
    callback: &mut F,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if is_allowed_dir(&path, config) {
                walk_directory_internal(&path, config, filter, callback)?;
                continue;
            }

            if !filter.should_descend(&path, config) {
                walk_allowed_descendants(&path, config, filter, callback)?;
                continue;
            }

            walk_directory_internal(&path, config, filter, callback)?;
        } else if path.is_file() {
            callback(&path)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn is_allowed_dir(path: &Path, config: &SimpleIndexerConfig) -> bool {
    path_starts_with_any(path, &config.allowed_dirs)
}

#[allow(dead_code)]
fn walk_allowed_descendants<F>(
    dir_path: &Path,
    config: &SimpleIndexerConfig,
    filter: &dyn TraversalFilter,
    callback: &mut F,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    let allowed_dirs = config.allowed_dirs.clone();
    for allowed in allowed_dirs {
        if allowed.starts_with(dir_path) && allowed.exists() {
            walk_directory_internal(&allowed, config, filter, callback)?;
        }
    }
    Ok(())
}
