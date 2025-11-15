//! Prompt rendering logic.

use crate::client::providers::McpProvider;
use crate::client::McpPromptDetail;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use vtcode_config::mcp::McpAllowListConfig;

/// Get a rendered prompt from the given provider.
///
/// # Arguments
/// * `provider` - The MCP provider to get the prompt from
/// * `prompt_name` - Name of the prompt to render
/// * `arguments` - Arguments to pass to the prompt
/// * `timeout` - Optional timeout for the operation
/// * `allowlist` - Allowlist configuration for filtering
///
/// # Returns
/// * `Ok(McpPromptDetail)` - The rendered prompt details
/// * `Err` - If the prompt fetch fails or is not allowed
pub async fn get_prompt_from_provider(
    provider: &Arc<McpProvider>,
    prompt_name: &str,
    arguments: HashMap<String, String>,
    timeout: Option<Duration>,
    allowlist: &McpAllowListConfig,
) -> Result<McpPromptDetail> {
    provider
        .get_prompt(prompt_name, arguments, timeout, allowlist)
        .await
}
