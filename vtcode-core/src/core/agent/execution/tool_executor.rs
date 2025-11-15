//! Tool execution logic

use super::{AgentState, ExecutionContext, Transition, ToolResult};
use crate::config::loader::ConfigManager;
use crate::config::constants::tools;
use crate::core::agent::types::AgentType;
use crate::tools::ToolRegistry;
use anyhow::{Result, anyhow};
use serde_json::Value;
use tracing::info;

/// Handles tool execution with retry logic and policy enforcement
pub struct ToolExecutor;

impl ToolExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a tool and prepare the result for recording
    pub async fn execute_tool(
        &self,
        tool_registry: &ToolRegistry,
        agent_type: AgentType,
        tool_name: String,
        tool_args: Value,
        call_id: String,
        _ctx: &ExecutionContext,
    ) -> Result<Transition> {
        // Enforce shell command policies if needed
        if tool_name == tools::RUN_COMMAND {
            self.enforce_shell_policies(agent_type, &tool_args)?;
        }

        // Clone registry for execution
        let mut registry = tool_registry.clone();
        registry.initialize_async().await?;

        // Execute with retry logic
        let result = self.execute_with_retry(&mut registry, &tool_name, &tool_args).await;

        // Wrap result in ToolResult for state machine
        let tool_result = ToolResult::from_result(result);

        // Transition to recording result
        Ok(Transition::Continue(AgentState::RecordingToolResult {
            tool_name,
            call_id,
            result: tool_result,
        }))
    }

    /// Execute tool with adaptive retry (up to 3 attempts)
    async fn execute_with_retry(
        &self,
        registry: &mut ToolRegistry,
        tool_name: &str,
        tool_args: &Value,
    ) -> Result<Value> {
        let mut delay = std::time::Duration::from_millis(200);

        for attempt in 0..3 {
            match registry.execute_tool(tool_name, tool_args.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < 2 => {
                    tokio::time::sleep(delay).await;
                    delay = delay.saturating_mul(2);
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Tool '{}' not found or failed to execute: {}",
                        tool_name,
                        e
                    ));
                }
            }
        }

        unreachable!()
    }

    /// Enforce shell command policies based on agent type and configuration
    fn enforce_shell_policies(&self, agent_type: AgentType, args: &Value) -> Result<()> {
        let cfg = ConfigManager::load()
            .or_else(|_| ConfigManager::load_from_workspace("."))
            .or_else(|_| ConfigManager::load_from_file("vtcode.toml"))
            .map(|cm| cm.config().clone())
            .unwrap_or_default();

        let cmd_text = if let Some(cmd_val) = args.get("command") {
            if cmd_val.is_array() {
                cmd_val
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default()
            } else {
                cmd_val.as_str().unwrap_or("").to_string()
            }
        } else {
            String::new()
        };

        let agent_prefix = format!(
            "VTCODE_{}_COMMANDS_",
            agent_type.to_string().to_uppercase()
        );

        // Check deny regex patterns
        let mut deny_regex = cfg.commands.deny_regex.clone();
        if let Ok(extra) = std::env::var(format!("{}DENY_REGEX", agent_prefix)) {
            deny_regex.extend(extra.split(',').map(|s| s.trim().to_string()));
        }
        for pat in &deny_regex {
            if regex::Regex::new(pat)
                .ok()
                .map(|re| re.is_match(&cmd_text))
                .unwrap_or(false)
            {
                return Err(anyhow!("Shell command denied by regex: {}", pat));
            }
        }

        // Check deny glob patterns
        let mut deny_glob = cfg.commands.deny_glob.clone();
        if let Ok(extra) = std::env::var(format!("{}DENY_GLOB", agent_prefix)) {
            deny_glob.extend(extra.split(',').map(|s| s.trim().to_string()));
        }
        for pat in &deny_glob {
            let re = format!("^{}$", regex::escape(pat).replace(r"\*", ".*"));
            if regex::Regex::new(&re)
                .ok()
                .map(|re| re.is_match(&cmd_text))
                .unwrap_or(false)
            {
                return Err(anyhow!("Shell command denied by glob: {}", pat));
            }
        }

        info!(target = "policy", agent = ?agent_type, tool = tools::RUN_COMMAND, cmd = %cmd_text, "shell_policy_checked");

        Ok(())
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}
