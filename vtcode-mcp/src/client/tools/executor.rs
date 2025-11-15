//! Tool execution logic.

use crate::client::providers::McpProvider;
use anyhow::{Result, anyhow};
use mcp_types::{CallToolResult, CallToolResultContentItem};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Duration;
use vtcode_config::mcp::McpAllowListConfig;

/// Format the result from a tool call into a structured JSON response.
///
/// # Arguments
/// * `provider_name` - Name of the provider that executed the tool
/// * `tool_name` - Name of the tool that was executed
/// * `result` - The raw result from the MCP tool call
///
/// # Returns
/// * `Ok(Value)` - Formatted JSON response with provider, tool, meta, and content
/// * `Err` - If the tool reported an error
pub fn format_tool_result(
    provider_name: &str,
    tool_name: &str,
    result: CallToolResult,
) -> Result<Value> {
    if result.is_error.unwrap_or(false) {
        let mut message = result
            .meta
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned);

        if message.is_none() {
            message = result.content.iter().find_map(|block| match block {
                CallToolResultContentItem::TextContent(text) => Some(text.text.clone()),
                _ => None,
            });
        }

        let message = message.unwrap_or_else(|| "Unknown MCP tool error".to_string());
        return Err(anyhow!(
            "MCP tool '{}' on provider '{}' reported an error: {}",
            tool_name,
            provider_name,
            message
        ));
    }

    let mut payload = Map::new();
    payload.insert("provider".into(), Value::String(provider_name.to_string()));
    payload.insert("tool".into(), Value::String(tool_name.to_string()));

    if !result.meta.is_empty() {
        payload.insert("meta".into(), Value::Object(result.meta.clone()));
    }

    if !result.content.is_empty() {
        payload.insert("content".into(), serde_json::to_value(result.content)?);
    }

    Ok(Value::Object(payload))
}

/// Execute a tool on the given provider.
///
/// # Arguments
/// * `provider` - The MCP provider to execute the tool on
/// * `tool_name` - Name of the tool to execute
/// * `args` - Arguments to pass to the tool
/// * `timeout` - Optional timeout for the tool execution
/// * `allowlist` - Allowlist configuration for filtering
///
/// # Returns
/// * `Ok(CallToolResult)` - The result from the tool execution
/// * `Err` - If the tool execution fails or is not allowed
pub async fn execute_tool_on_provider(
    provider: &Arc<McpProvider>,
    tool_name: &str,
    args: Value,
    timeout: Option<Duration>,
    allowlist: &McpAllowListConfig,
) -> Result<CallToolResult> {
    provider.call_tool(tool_name, args, timeout, allowlist).await
}
