//! Core validation framework
//!
//! This module provides the main command validation dispatcher that routes
//! commands to specialized validators based on the command type.

use std::path::Path;

use anyhow::{Result, anyhow};

/// Validate whether a command is allowed to run under the execution policy.
///
/// The policy is inspired by the Codex execution policy and limits commands to
/// a curated allow-list with argument validation to prevent workspace
/// breakout or destructive actions.
///
/// # Arguments
///
/// * `command` - The command and arguments to validate (first element is program)
/// * `workspace_root` - The root workspace directory
/// * `working_dir` - The current working directory
///
/// # Returns
///
/// * `Ok(())` if the command is allowed
/// * `Err(_)` if the command is not allowed or arguments are invalid
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_execution::policy::validate_command;
/// use std::path::Path;
///
/// let workspace = Path::new("/workspace");
/// let working_dir = Path::new("/workspace/project");
/// let command = vec!["git".to_string(), "status".to_string()];
///
/// validate_command(&command, workspace, working_dir).await?;
/// ```
pub async fn validate_command(
    command: &[String],
    workspace_root: &Path,
    working_dir: &Path,
) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("command cannot be empty"));
    }

    let program = command[0].as_str();
    let args = &command[1..];

    match program {
        // File operations
        "cat" => super::validators::fileops::validate_cat(args, workspace_root, working_dir).await,
        "cp" => super::validators::fileops::validate_cp(args, workspace_root, working_dir).await,
        "head" => super::validators::fileops::validate_head(args, workspace_root, working_dir).await,
        "tail" => super::validators::fileops::validate_tail(args, workspace_root, working_dir).await,
        "ls" => super::validators::fileops::validate_ls(args, workspace_root, working_dir).await,
        "wc" => super::validators::fileops::validate_wc(args, workspace_root, working_dir).await,

        // Search operations
        "grep" => super::validators::search::validate_grep(args, workspace_root, working_dir).await,
        "rg" => super::validators::search::validate_rg(args, workspace_root, working_dir).await,
        "sed" => super::validators::search::validate_sed(args, workspace_root, working_dir).await,

        // Development tools
        "git" => super::validators::git::validate_git(args, workspace_root, working_dir).await,
        "cargo" => super::validators::cargo::validate_cargo(args, workspace_root, working_dir).await,
        "npm" => super::validators::npm::validate_npm(args, workspace_root, working_dir).await,
        "python" | "python3" => super::validators::python::validate_python(args, workspace_root, working_dir).await,
        "node" => super::validators::python::validate_node(args, workspace_root, working_dir).await,

        // System info
        "echo" => super::validators::system::validate_echo(args),
        "pwd" => super::validators::system::validate_pwd(args),
        "printenv" => super::validators::system::validate_printenv(args),
        "which" => super::validators::system::validate_which(args),
        "date" => super::validators::system::validate_date(args),
        "whoami" => super::validators::system::validate_whoami(args),
        "hostname" => super::validators::system::validate_hostname(args),
        "uname" => super::validators::system::validate_uname(args),

        other => Err(anyhow!(
            "command '{}' is not permitted by the execution policy",
            other
        )),
    }
}
