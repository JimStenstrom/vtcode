//! Standard process-based command executor

use anyhow::{Context, Result};
use std::process::Command;

use super::{CommandExecutor, CommandInvocation, CommandOutput, CommandStatus, ShellKind};

/// Executes commands by delegating to the system shell via [`std::process::Command`].
pub struct ProcessCommandExecutor;

impl ProcessCommandExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcessCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExecutor for ProcessCommandExecutor {
    fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput> {
        let mut cmd = match invocation.shell {
            ShellKind::Unix => {
                let mut command = Command::new("sh");
                command.arg("-c").arg(&invocation.command);
                command
            }
            ShellKind::Windows => {
                #[cfg(not(feature = "powershell-process"))]
                {
                    anyhow::bail!(
                        "powershell-process feature disabled; enable it to execute Windows commands"
                    );
                }
                #[cfg(feature = "powershell-process")]
                {
                    let mut command = Command::new("powershell");
                    command
                        .arg("-NoProfile")
                        .arg("-NonInteractive")
                        .arg("-Command")
                        .arg(&invocation.command);
                    command
                }
            }
        };

        cmd.current_dir(&invocation.working_dir);
        let output = cmd
            .output()
            .with_context(|| format!("failed to execute command: {}", invocation.command))?;

        Ok(CommandOutput {
            status: CommandStatus::from(output.status),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
