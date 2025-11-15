//! Search operation commands

use anyhow::{Result, anyhow};
use std::path::Path;

use crate::executor::{CommandCategory, CommandExecutor, CommandInvocation, CommandOutput, ShellKind};
use crate::policy::CommandPolicy;

use super::formatting;
use super::paths;

/// Search for pattern in files
pub fn grep<E, P>(
    executor: &E,
    policy: &P,
    workspace_root: &Path,
    working_dir: &Path,
    shell_kind: ShellKind,
    pattern: &str,
    path: Option<&str>,
    recursive: bool,
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
            let mut parts = vec!["grep".to_string(), "-n".to_string()];
            if recursive {
                parts.push("-r".to_string());
            }
            parts.push(formatting::format_pattern(shell_kind, pattern));
            parts.push(formatting::format_path(shell_kind, &target));
            formatting::join_command(parts)
        }
        ShellKind::Windows => {
            let mut parts = vec![
                "Select-String".to_string(),
                format!("-Pattern {}", formatting::format_pattern(shell_kind, pattern)),
                format!("-Path {}", formatting::format_path(shell_kind, &target)),
                "-SimpleMatch".to_string(),
            ];
            if recursive {
                parts.push("-Recurse".to_string());
            }
            formatting::join_command(parts)
        }
    };

    let invocation = CommandInvocation::new(
        shell_kind,
        command,
        CommandCategory::Search,
        working_dir.to_path_buf(),
    )
    .with_paths(vec![target]);

    policy.check(&invocation)?;
    let output = executor.execute(&invocation)?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    if output.stdout.trim().is_empty() && output.stderr.trim().is_empty() {
        Ok(String::new())
    } else {
        Err(anyhow!(
            "search command failed: {}",
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
