//! Core command execution types

#[cfg(feature = "serde-errors")]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logical grouping for commands issued by the [`BashRunner`][crate::BashRunner].
#[cfg_attr(feature = "serde-errors", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    ChangeDirectory,
    ListDirectory,
    PrintDirectory,
    CreateDirectory,
    Remove,
    Copy,
    Move,
    Search,
}

/// Shell family used to execute commands.
#[cfg_attr(feature = "serde-errors", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellKind {
    Unix,
    Windows,
}

/// Describes a command that will be executed by a [`CommandExecutor`][super::CommandExecutor].
#[cfg_attr(feature = "serde-errors", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CommandInvocation {
    pub shell: ShellKind,
    pub command: String,
    pub category: CommandCategory,
    pub working_dir: PathBuf,
    pub touched_paths: Vec<PathBuf>,
}

impl CommandInvocation {
    pub fn new(
        shell: ShellKind,
        command: String,
        category: CommandCategory,
        working_dir: PathBuf,
    ) -> Self {
        Self {
            shell,
            command,
            category,
            working_dir,
            touched_paths: Vec::new(),
        }
    }

    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.touched_paths = paths;
        self
    }
}

/// Describes the exit status of a command execution.
#[cfg_attr(feature = "serde-errors", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandStatus {
    success: bool,
    code: Option<i32>,
}

impl CommandStatus {
    pub fn new(success: bool, code: Option<i32>) -> Self {
        Self { success, code }
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn code(&self) -> Option<i32> {
        self.code
    }

    pub fn failure(code: Option<i32>) -> Self {
        Self {
            success: false,
            code,
        }
    }
}

impl From<std::process::ExitStatus> for CommandStatus {
    fn from(status: std::process::ExitStatus) -> Self {
        let code = status.code();
        Self {
            success: status.success(),
            code,
        }
    }
}

/// Output produced by the executor for a command invocation.
#[cfg_attr(feature = "serde-errors", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: CommandStatus,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            status: CommandStatus::new(true, Some(0)),
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub fn failure(
        code: Option<i32>,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
    ) -> Self {
        Self {
            status: CommandStatus::failure(code),
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
}
