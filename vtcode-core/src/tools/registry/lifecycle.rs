//! Tool lifecycle management (initialization, cleanup, configuration)

use super::core::ToolRegistry;
use super::timeout::ToolTimeoutPolicy;
use crate::config::{CommandsConfig, TimeoutsConfig, ToolsConfig};
use crate::tool_policy::{ToolPolicy, ToolPolicyManager};
use anyhow::Result;
use tracing::warn;

impl ToolRegistry {
    pub async fn initialize_async(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn apply_config_policies(&mut self, tools_config: &ToolsConfig) -> Result<()> {
        if let Ok(policy_manager) = self.policy_manager_mut() {
            policy_manager.apply_tools_config(tools_config).await?;
        }

        Ok(())
    }

    pub fn apply_commands_config(&mut self, commands_config: &CommandsConfig) {
        self.inventory
            .command_tool_mut()
            .update_commands_config(commands_config);
        self.pty_sessions
            .manager()
            .apply_commands_config(commands_config);
    }

    pub fn apply_timeout_policy(&mut self, timeouts: &TimeoutsConfig) {
        let policy = ToolTimeoutPolicy::from_config(timeouts);

        // Validate the policy before applying
        if let Err(e) = policy.validate() {
            warn!(
                error = %e,
                "Invalid timeout configuration detected, using defaults"
            );
            self.timeout_policy = ToolTimeoutPolicy::default();
        } else {
            self.timeout_policy = policy;
        }
    }

    pub fn policy_manager_mut(&mut self) -> Result<&mut ToolPolicyManager> {
        self.policy_gateway.policy_manager_mut()
    }

    pub fn policy_manager(&self) -> Result<&ToolPolicyManager> {
        self.policy_gateway.policy_manager()
    }

    pub async fn set_policy_manager(&mut self, manager: ToolPolicyManager) {
        self.policy_gateway.set_policy_manager(manager);
        self.sync_policy_catalog().await;
    }

    pub async fn set_tool_policy(&mut self, tool_name: &str, policy: ToolPolicy) -> Result<()> {
        self.policy_gateway.set_tool_policy(tool_name, policy).await
    }

    pub fn get_tool_policy(&self, tool_name: &str) -> ToolPolicy {
        self.policy_gateway.get_tool_policy(tool_name)
    }

    pub async fn reset_tool_policies(&mut self) -> Result<()> {
        self.policy_gateway.reset_tool_policies().await
    }

    pub async fn allow_all_tools(&mut self) -> Result<()> {
        self.policy_gateway.allow_all_tools().await
    }

    pub async fn deny_all_tools(&mut self) -> Result<()> {
        self.policy_gateway.deny_all_tools().await
    }

    pub fn print_policy_status(&self) {
        self.policy_gateway.print_policy_status();
    }

    pub async fn enable_full_auto_mode(&mut self, allowed_tools: &[String]) {
        let available = self.available_tools().await;
        self.policy_gateway
            .enable_full_auto_mode(allowed_tools, &available);
    }

    pub fn disable_full_auto_mode(&mut self) {
        self.policy_gateway.disable_full_auto_mode();
    }

    pub fn current_full_auto_allowlist(&self) -> Option<Vec<String>> {
        self.policy_gateway.current_full_auto_allowlist()
    }
}
