//! Tool execution dispatch orchestration

use super::core::ToolRegistry;
use super::error::{ToolErrorType, ToolExecutionError, classify_error};
use super::registration::ToolHandler;
use super::timeout::ToolTimeoutCategory;
use super::utils::normalize_tool_output;
use crate::tool_policy::ToolPolicy;
use crate::tools::names::canonical_tool_name;
use anyhow::Result;
use serde_json::Value;
use tracing::warn;
use vtcode_mcp::McpToolExecutor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPermissionDecision {
    Allow,
    Deny,
    Prompt,
}

impl ToolRegistry {
    pub async fn timeout_category_for(&mut self, name: &str) -> ToolTimeoutCategory {
        let canonical_name = canonical_tool_name(name);
        let tool_name = canonical_name.as_ref();

        if let Some(registration) = self.inventory.registration_for(tool_name) {
            return if registration.uses_pty() {
                ToolTimeoutCategory::Pty
            } else {
                ToolTimeoutCategory::Default
            };
        }

        if let Some(stripped) = name.strip_prefix("mcp_") {
            if self.has_mcp_tool(stripped).await {
                return ToolTimeoutCategory::Mcp;
            }
        } else if self.find_mcp_provider(tool_name).is_some() || self.has_mcp_tool(tool_name).await
        {
            return ToolTimeoutCategory::Mcp;
        }

        ToolTimeoutCategory::Default
    }

    pub async fn execute_tool(&mut self, name: &str, args: Value) -> Result<Value> {
        let canonical_name = canonical_tool_name(name);
        let tool_name = canonical_name.as_ref();
        let display_name = if tool_name == name {
            name.to_string()
        } else {
            format!("{} (alias for {})", name, tool_name)
        };

        if self.policy_gateway.has_full_auto_allowlist()
            && !self.policy_gateway.is_allowed_in_full_auto(tool_name)
        {
            let error = ToolExecutionError::new(
                tool_name.to_string(),
                ToolErrorType::PolicyViolation,
                format!(
                    "Tool '{}' is not permitted while full-auto mode is active",
                    display_name
                ),
            );
            return Ok(error.to_json_value());
        }

        let skip_policy_prompt = self.policy_gateway.take_preapproved(tool_name);

        if !skip_policy_prompt && !self.policy_gateway.should_execute_tool(tool_name).await? {
            let error = ToolExecutionError::new(
                tool_name.to_string(),
                ToolErrorType::PolicyViolation,
                format!("Tool '{}' execution denied by policy", display_name),
            );
            return Ok(error.to_json_value());
        }

        let args = match self
            .policy_gateway
            .apply_policy_constraints(tool_name, args)
        {
            Ok(args) => args,
            Err(err) => {
                let error = ToolExecutionError::with_original_error(
                    tool_name.to_string(),
                    ToolErrorType::InvalidParameters,
                    "Failed to apply policy constraints".to_string(),
                    err.to_string(),
                );
                return Ok(error.to_json_value());
            }
        };

        // First, check if we need a PTY session by checking if the tool exists and needs PTY
        let mut needs_pty = false;
        let mut tool_exists = false;
        let mut is_mcp_tool = false;
        let mut mcp_tool_name: Option<String> = None;
        let mut mcp_lookup_error: Option<anyhow::Error> = None;

        // Check if it's a standard tool first
        if let Some(registration) = self.inventory.registration_for(tool_name) {
            needs_pty = registration.uses_pty();
            tool_exists = true;
        }
        // If not a standard tool, check if it's an MCP tool
        else if let Some(mcp_client) = &self.mcp_client {
            let resolved_mcp_name = if let Some(stripped) = name.strip_prefix("mcp_") {
                stripped.to_string()
            } else {
                tool_name.to_string()
            };

            match mcp_client.has_mcp_tool(&resolved_mcp_name).await {
                Ok(true) => {
                    needs_pty = true;
                    tool_exists = true;
                    is_mcp_tool = true;
                    mcp_tool_name = Some(resolved_mcp_name);
                }
                Ok(false) => {
                    tool_exists = false;
                }
                Err(err) => {
                    warn!("Error checking MCP tool '{}': {}", resolved_mcp_name, err);
                    mcp_lookup_error = Some(err);
                }
            }
        }

        // If tool doesn't exist in either registry, return an error
        if !tool_exists {
            if let Some(err) = mcp_lookup_error {
                let error = ToolExecutionError::with_original_error(
                    tool_name.to_string(),
                    ToolErrorType::ExecutionError,
                    format!("Failed to resolve MCP tool '{}': {}", display_name, err),
                    err.to_string(),
                );
                return Ok(error.to_json_value());
            }

            let error = ToolExecutionError::new(
                tool_name.to_string(),
                ToolErrorType::ToolNotFound,
                format!("Unknown tool: {}", display_name),
            );
            return Ok(error.to_json_value());
        }

        // Start PTY session if needed
        if needs_pty {
            if let Err(err) = self.start_pty_session() {
                let error = ToolExecutionError::with_original_error(
                    tool_name.to_string(),
                    ToolErrorType::ExecutionError,
                    "Failed to start PTY session".to_string(),
                    err.to_string(),
                );
                return Ok(error.to_json_value());
            }
        }

        // Execute the appropriate tool based on its type
        let result = if is_mcp_tool {
            let mcp_name =
                mcp_tool_name.expect("mcp_tool_name should be set when is_mcp_tool is true");
            self.execute_mcp_tool(&mcp_name, args).await
        } else if let Some(registration) = self.inventory.registration_for(tool_name) {
            // Log deprecation warning if tool is deprecated
            if registration.is_deprecated() {
                if let Some(msg) = registration.deprecation_message() {
                    warn!("Tool '{}' is deprecated: {}", tool_name, msg);
                } else {
                    warn!(
                        "Tool '{}' is deprecated and may be removed in a future version",
                        tool_name
                    );
                }
            }

            let handler = registration.handler();
            match handler {
                ToolHandler::RegistryFn(executor) => executor(self, args).await,
                ToolHandler::TraitObject(tool) => tool.execute(args).await,
            }
        } else {
            // This should theoretically never happen since we checked tool_exists above
            return Ok(ToolExecutionError::new(
                tool_name.to_string(),
                ToolErrorType::ToolNotFound,
                "Tool not found in registry".to_string(),
            )
            .to_json_value());
        };

        // Clean up PTY session if we started one
        if needs_pty {
            self.end_pty_session();
        }

        // Handle the execution result
        match result {
            Ok(value) => Ok(normalize_tool_output(value)),
            Err(err) => {
                let error_type = classify_error(&err);
                let error = ToolExecutionError::with_original_error(
                    tool_name.to_string(),
                    error_type,
                    format!("Tool execution failed: {}", err),
                    err.to_string(),
                );
                Ok(error.to_json_value())
            }
        }
    }

    /// Execute an MCP tool
    pub async fn execute_mcp_tool(&self, tool_name: &str, args: Value) -> Result<Value> {
        if let Some(mcp_client) = &self.mcp_client {
            mcp_client.execute_mcp_tool(tool_name, args).await
        } else {
            Err(anyhow::anyhow!("MCP client not available"))
        }
    }

    /// Prompt for permission before starting long-running tool executions to avoid spinner conflicts
    pub async fn preflight_tool_permission(&mut self, name: &str) -> Result<bool> {
        match self.evaluate_tool_policy(name).await? {
            ToolPermissionDecision::Allow => Ok(true),
            ToolPermissionDecision::Deny => Ok(false),
            ToolPermissionDecision::Prompt => Ok(true),
        }
    }

    pub async fn evaluate_tool_policy(&mut self, name: &str) -> Result<ToolPermissionDecision> {
        if let Some(tool_name) = name.strip_prefix("mcp_") {
            return self.evaluate_mcp_tool_policy(name, tool_name).await;
        }

        self.policy_gateway.evaluate_tool_policy(name).await
    }

    async fn evaluate_mcp_tool_policy(
        &mut self,
        full_name: &str,
        tool_name: &str,
    ) -> Result<ToolPermissionDecision> {
        let provider = match self.find_mcp_provider(tool_name) {
            Some(provider) => provider,
            None => {
                // Unknown provider for this tool; default to prompt for safety
                return Ok(ToolPermissionDecision::Prompt);
            }
        };

        if self.policy_gateway.has_full_auto_allowlist()
            && !self.policy_gateway.is_allowed_in_full_auto(full_name)
        {
            return Ok(ToolPermissionDecision::Deny);
        }

        if let Ok(policy_manager) = self.policy_manager_mut() {
            match policy_manager.get_mcp_tool_policy(&provider, tool_name) {
                ToolPolicy::Allow => {
                    self.policy_gateway.preapprove(full_name);
                    Ok(ToolPermissionDecision::Allow)
                }
                ToolPolicy::Deny => Ok(ToolPermissionDecision::Deny),
                ToolPolicy::Prompt => {
                    // Always prompt for explicit "prompt" policy, even in full-auto mode
                    // This ensures human-in-the-loop approval for sensitive operations
                    Ok(ToolPermissionDecision::Prompt)
                }
            }
        } else {
            // Policy manager not available - default to prompt for safety
            // instead of auto-approving
            Ok(ToolPermissionDecision::Prompt)
        }
    }

    pub fn mark_tool_preapproved(&mut self, name: &str) {
        self.policy_gateway.preapprove(name);
    }

    pub async fn persist_mcp_tool_policy(&mut self, name: &str, policy: ToolPolicy) -> Result<()> {
        if !name.starts_with("mcp_") {
            return Ok(());
        }

        let Some(tool_name) = name.strip_prefix("mcp_") else {
            return Ok(());
        };

        let Some(provider) = self.find_mcp_provider(tool_name) else {
            return Ok(());
        };

        self.policy_gateway
            .persist_mcp_tool_policy(&provider, tool_name, policy)
            .await
    }
}
