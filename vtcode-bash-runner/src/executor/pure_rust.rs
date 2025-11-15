//! Pure Rust command executor (no subprocess)

use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::path::{Path, PathBuf};

use super::{CommandCategory, CommandExecutor, CommandInvocation, CommandOutput};

/// Executes file system operations using pure Rust (no subprocess).
///
/// This executor implements common file operations like ls, mkdir, rm, cp, and mv
/// using Rust's standard library instead of spawning shell processes.
#[derive(Debug, Default, Clone, Copy)]
pub struct PureRustCommandExecutor;

impl PureRustCommandExecutor {
    fn resolve_primary_path(invocation: &CommandInvocation) -> Result<&PathBuf> {
        invocation
            .touched_paths
            .first()
            .ok_or_else(|| anyhow!("invocation missing target path"))
    }

    fn extract_source_dest_paths<'a>(
        invocation: &'a CommandInvocation,
        operation: &str,
    ) -> Result<(&'a Path, &'a Path)> {
        let source = invocation
            .touched_paths
            .first()
            .ok_or_else(|| anyhow!("{} missing source path", operation))?;
        let dest = invocation
            .touched_paths
            .get(1)
            .ok_or_else(|| anyhow!("{} missing destination path", operation))?;
        Ok((source.as_path(), dest.as_path()))
    }

    fn should_include_hidden(command: &str) -> bool {
        command.contains("-a") || command.contains("-Force")
    }

    fn is_recursive(command: &str) -> bool {
        command.contains("-r") || command.contains("-Recurse")
    }

    fn has_parents_flag(command: &str) -> bool {
        command.contains("-p") || command.contains("-Force")
    }

    fn mkdir(path: &Path, command: &str) -> Result<()> {
        if Self::has_parents_flag(command) {
            fs::create_dir_all(path)
                .with_context(|| format!("failed to create directory `{}`", path.display()))?
        } else {
            fs::create_dir(path)
                .with_context(|| format!("failed to create directory `{}`", path.display()))?
        }
        Ok(())
    }

    fn rm(path: &Path, command: &str) -> Result<()> {
        if path.is_dir() {
            if Self::is_recursive(command) {
                fs::remove_dir_all(path)
                    .with_context(|| format!("failed to remove directory `{}`", path.display()))?
            } else {
                fs::remove_dir(path)
                    .with_context(|| format!("failed to remove directory `{}`", path.display()))?
            }
        } else if path.exists() {
            fs::remove_file(path)
                .with_context(|| format!("failed to remove file `{}`", path.display()))?
        }
        Ok(())
    }

    fn copy_recursive(source: &Path, dest: &Path, recursive: bool) -> Result<()> {
        if source.is_dir() {
            if !recursive {
                bail!(
                    "copying directory `{}` requires recursive flag",
                    source.display()
                );
            }
            fs::create_dir_all(dest)
                .with_context(|| format!("failed to create directory `{}`", dest.display()))?;
            for entry in fs::read_dir(source)
                .with_context(|| format!("failed to read directory `{}`", source.display()))?
            {
                let entry = entry?;
                let entry_path = entry.path();
                let dest_path = dest.join(entry.file_name());
                if entry_path.is_dir() {
                    Self::copy_recursive(&entry_path, &dest_path, true)?;
                } else {
                    Self::copy_file(&entry_path, &dest_path)?;
                }
            }
        } else {
            Self::copy_file(source, dest)?;
        }
        Ok(())
    }

    fn ensure_parent_dir(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to prepare destination directory `{}`",
                    parent.display()
                )
            })?;
        }
        Ok(())
    }

    fn copy_file(source: &Path, dest: &Path) -> Result<()> {
        Self::ensure_parent_dir(dest)?;
        fs::copy(source, dest).with_context(|| {
            format!(
                "failed to copy `{}` to `{}`",
                source.display(),
                dest.display()
            )
        })?;
        Ok(())
    }

    fn move_path(source: &Path, dest: &Path) -> Result<()> {
        Self::ensure_parent_dir(dest)?;

        if let Err(rename_err) = fs::rename(source, dest) {
            Self::copy_recursive(source, dest, true)
                .and_then(|_| Self::rm(source, "-r -f"))
                .with_context(|| {
                    format!(
                        "failed to move `{}` to `{}` via rename: {rename_err}",
                        source.display(),
                        dest.display()
                    )
                })?;
        }
        Ok(())
    }
}

impl CommandExecutor for PureRustCommandExecutor {
    fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput> {
        match invocation.category {
            CommandCategory::ListDirectory => {
                let path = Self::resolve_primary_path(invocation)?;
                let mut entries = Vec::new();
                for entry in fs::read_dir(path)
                    .with_context(|| format!("failed to read directory `{}`", path.display()))?
                {
                    let entry = entry?;
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if !Self::should_include_hidden(&invocation.command) && name.starts_with('.') {
                        continue;
                    }
                    entries.push(name.to_string());
                }
                entries.sort();
                Ok(CommandOutput::success(entries.join("\n")))
            }
            CommandCategory::CreateDirectory => {
                let path = Self::resolve_primary_path(invocation)?;
                Self::mkdir(path, &invocation.command)?;
                Ok(CommandOutput::success(String::new()))
            }
            CommandCategory::Remove => {
                let path = Self::resolve_primary_path(invocation)?;
                Self::rm(path, &invocation.command)?;
                Ok(CommandOutput::success(String::new()))
            }
            CommandCategory::Copy => {
                let (source, dest) = Self::extract_source_dest_paths(invocation, "copy")?;
                let recursive = Self::is_recursive(&invocation.command);
                Self::copy_recursive(source, dest, recursive)?;
                Ok(CommandOutput::success(String::new()))
            }
            CommandCategory::Move => {
                let (source, dest) = Self::extract_source_dest_paths(invocation, "move")?;
                Self::move_path(source, dest)?;
                Ok(CommandOutput::success(String::new()))
            }
            CommandCategory::Search => bail!(
                "pure-rust executor does not implement search; enable std-process or provide a custom executor"
            ),
            CommandCategory::ChangeDirectory | CommandCategory::PrintDirectory => {
                Ok(CommandOutput::success(String::new()))
            }
        }
    }
}
