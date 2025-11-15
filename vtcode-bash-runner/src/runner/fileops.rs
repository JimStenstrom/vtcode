//! File operation commands

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use crate::executor::{CommandCategory, CommandExecutor, CommandInvocation, CommandOutput, ShellKind};
use crate::policy::CommandPolicy;

use super::formatting;
use super::paths;

/// List directory contents
pub fn ls<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    path: Option<&str>,
    show_hidden: bool,
) -> Result<String>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let target = path
        .map(|p| paths::resolve_existing_path(workspace_root, working_dir, p))
        .transpose()?
        .unwrap_or_else(|| working_dir.to_path_buf());

    let command = match shell_kind {
        ShellKind::Unix => {
            let flag = if show_hidden { "-la" } else { "-l" };
            format!("ls {} {}", flag, formatting::format_path(shell_kind, &target))
        }
        ShellKind::Windows => {
            let mut parts = vec!["Get-ChildItem".to_string()];
            if show_hidden {
                parts.push("-Force".to_string());
            }
            parts.push(format!("-Path {}", formatting::format_path(shell_kind, &target)));
            formatting::join_command(parts)
        }
    };

    let invocation = CommandInvocation::new(
        shell_kind,
        command,
        CommandCategory::ListDirectory,
        working_dir.to_path_buf(),
    )
    .with_paths(vec![target]);

    let output = execute_and_expect_success(executor, policy, invocation)?;
    Ok(output.stdout)
}

/// Print working directory
pub fn pwd<P>(
    policy: &P,
    working_dir: &Path,
    shell_kind: ShellKind,
) -> Result<String>
where
    P: CommandPolicy,
{
    let invocation = CommandInvocation::new(
        shell_kind,
        match shell_kind {
            ShellKind::Unix => "pwd".to_string(),
            ShellKind::Windows => "Get-Location".to_string(),
        },
        CommandCategory::PrintDirectory,
        working_dir.to_path_buf(),
    );
    policy.check(&invocation)?;
    Ok(working_dir.to_string_lossy().to_string())
}

/// Create directory
pub fn mkdir<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    path: &str,
    parents: bool,
) -> Result<()>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let target = paths::resolve_path(working_dir, path);
    paths::ensure_mutation_target_within_workspace(workspace_root, working_dir, &target)?;

    let command = match shell_kind {
        ShellKind::Unix => {
            let mut parts = vec!["mkdir".to_string()];
            if parents {
                parts.push("-p".to_string());
            }
            parts.push(formatting::format_path(shell_kind, &target));
            formatting::join_command(parts)
        }
        ShellKind::Windows => {
            let mut parts = vec!["New-Item".to_string(), "-ItemType Directory".to_string()];
            if parents {
                parts.push("-Force".to_string());
            }
            parts.push(format!("-Path {}", formatting::format_path(shell_kind, &target)));
            formatting::join_command(parts)
        }
    };

    execute_fs_command(
        executor,
        policy,
        shell_kind,
        working_dir,
        command,
        CommandCategory::CreateDirectory,
        vec![target],
    )
}

/// Remove file or directory
pub fn rm<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    path: &str,
    recursive: bool,
    force: bool,
) -> Result<()>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let target = paths::resolve_path(working_dir, path);
    paths::ensure_mutation_target_within_workspace(workspace_root, working_dir, &target)?;

    let command = match shell_kind {
        ShellKind::Unix => {
            let mut parts = vec!["rm".to_string()];
            if recursive {
                parts.push("-r".to_string());
            }
            if force {
                parts.push("-f".to_string());
            }
            parts.push(formatting::format_path(shell_kind, &target));
            formatting::join_command(parts)
        }
        ShellKind::Windows => {
            let mut parts = vec!["Remove-Item".to_string()];
            if recursive {
                parts.push("-Recurse".to_string());
            }
            if force {
                parts.push("-Force".to_string());
            }
            parts.push(format!("-Path {}", formatting::format_path(shell_kind, &target)));
            formatting::join_command(parts)
        }
    };

    execute_fs_command(
        executor,
        policy,
        shell_kind,
        working_dir,
        command,
        CommandCategory::Remove,
        vec![target],
    )
}

/// Copy file or directory
pub fn cp<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    source: &str,
    dest: &str,
    recursive: bool,
) -> Result<()>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let source_path = paths::resolve_existing_path(workspace_root, working_dir, source)?;
    let dest_path = paths::resolve_path(working_dir, dest);
    paths::ensure_mutation_target_within_workspace(workspace_root, working_dir, &dest_path)?;

    let command = match shell_kind {
        ShellKind::Unix => {
            let mut parts = vec!["cp".to_string()];
            if recursive {
                parts.push("-r".to_string());
            }
            parts.push(formatting::format_path(shell_kind, &source_path));
            parts.push(formatting::format_path(shell_kind, &dest_path));
            formatting::join_command(parts)
        }
        ShellKind::Windows => {
            let mut parts = vec![
                "Copy-Item".to_string(),
                format!("-Path {}", formatting::format_path(shell_kind, &source_path)),
                format!("-Destination {}", formatting::format_path(shell_kind, &dest_path)),
            ];
            if recursive {
                parts.push("-Recurse".to_string());
            }
            formatting::join_command(parts)
        }
    };

    execute_fs_command(
        executor,
        policy,
        shell_kind,
        working_dir,
        command,
        CommandCategory::Copy,
        vec![source_path, dest_path],
    )
}

/// Move file or directory
pub fn mv<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    source: &str,
    dest: &str,
) -> Result<()>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let source_path = paths::resolve_existing_path(workspace_root, working_dir, source)?;
    let dest_path = paths::resolve_path(working_dir, dest);
    paths::ensure_mutation_target_within_workspace(workspace_root, working_dir, &dest_path)?;

    let command = match shell_kind {
        ShellKind::Unix => format!(
            "mv {} {}",
            formatting::format_path(shell_kind, &source_path),
            formatting::format_path(shell_kind, &dest_path)
        ),
        ShellKind::Windows => formatting::join_command(vec![
            "Move-Item".to_string(),
            format!("-Path {}", formatting::format_path(shell_kind, &source_path)),
            format!("-Destination {}", formatting::format_path(shell_kind, &dest_path)),
        ]),
    };

    execute_fs_command(
        executor,
        policy,
        shell_kind,
        working_dir,
        command,
        CommandCategory::Move,
        vec![source_path, dest_path],
    )
}

fn execute_fs_command<E, P>(
    executor: &E,
    policy: &P,
    shell_kind: ShellKind,
    working_dir: &Path,
    command: String,
    category: CommandCategory,
    paths: Vec<PathBuf>,
) -> Result<()>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    let invocation = CommandInvocation::new(
        shell_kind,
        command,
        category,
        working_dir.to_path_buf(),
    )
    .with_paths(paths);

    execute_and_expect_success(executor, policy, invocation).map(|_| ())
}

fn execute_and_expect_success<E, P>(
    executor: &E,
    policy: &P,
    invocation: CommandInvocation,
) -> Result<CommandOutput>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    policy.check(&invocation)?;
    let output = executor.execute(&invocation)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(anyhow!(
            "command `{}` failed: {}",
            invocation.command,
            select_error_output(&output)
        ))
    }
}

fn select_error_output(output: &CommandOutput) -> &str {
    if output.stderr.trim().is_empty() {
        &output.stdout
    } else {
        &output.stderr
    }
}
