//! Command execution backends
//!
//! This module provides the [`CommandExecutor`] trait and several implementations:
//!
//! - [`ProcessCommandExecutor`]: Delegates to system shell (sh/powershell)
//! - [`DryRunCommandExecutor`]: Logs commands without executing (testing)
//! - [`PureRustCommandExecutor`]: Pure Rust file operations (no subprocess)
//! - [`EventfulExecutor`]: Wrapper that emits execution events

pub mod types;
pub mod helpers;

#[cfg(feature = "std-process")]
pub mod process;

#[cfg(feature = "dry-run")]
pub mod dry_run;

#[cfg(feature = "pure-rust")]
pub mod pure_rust;

#[cfg(feature = "exec-events")]
pub mod eventful;

// Re-export core types
pub use types::{CommandCategory, CommandInvocation, CommandOutput, CommandStatus, ShellKind};

// Re-export executors
#[cfg(feature = "std-process")]
pub use process::ProcessCommandExecutor;

#[cfg(feature = "dry-run")]
pub use dry_run::DryRunCommandExecutor;

#[cfg(feature = "pure-rust")]
pub use pure_rust::PureRustCommandExecutor;

#[cfg(feature = "exec-events")]
pub use eventful::EventfulExecutor;

use anyhow::Result;

/// Trait implemented by concrete command execution strategies.
///
/// Implementors receive a [`CommandInvocation`] describing what to execute
/// and return a [`CommandOutput`] with the result.
pub trait CommandExecutor: Send + Sync {
    fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput>;
}
