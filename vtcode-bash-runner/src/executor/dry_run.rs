//! Dry-run command executor for testing

use anyhow::Result;
use std::sync::{Arc, Mutex};

use super::{CommandCategory, CommandExecutor, CommandInvocation, CommandOutput};

/// Executor that logs commands without executing them.
///
/// Useful for testing and debugging command generation.
#[derive(Clone, Default)]
pub struct DryRunCommandExecutor {
    log: Arc<Mutex<Vec<CommandInvocation>>>,
}

impl DryRunCommandExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a clone of all logged invocations.
    pub fn logged_invocations(&self) -> Vec<CommandInvocation> {
        match self.log.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl CommandExecutor for DryRunCommandExecutor {
    fn execute(&self, invocation: &CommandInvocation) -> Result<CommandOutput> {
        let mut guard = match self.log.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.push(invocation.clone());
        Ok(match invocation.category {
            CommandCategory::ListDirectory => CommandOutput::success("(dry-run listing)"),
            _ => CommandOutput::success(String::new()),
        })
    }
}
