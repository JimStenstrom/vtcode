//! Resource fetching logic.

use crate::client::providers::McpProvider;
use crate::client::McpResourceData;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use vtcode_config::mcp::McpAllowListConfig;

/// Fetch a resource from the given provider.
///
/// # Arguments
/// * `provider` - The MCP provider to fetch the resource from
/// * `uri` - The URI of the resource to fetch
/// * `timeout` - Optional timeout for the fetch operation
/// * `allowlist` - Allowlist configuration for filtering
///
/// # Returns
/// * `Ok(McpResourceData)` - The fetched resource data
/// * `Err` - If the fetch fails or the resource is not allowed
pub async fn fetch_resource_from_provider(
    provider: &Arc<McpProvider>,
    uri: &str,
    timeout: Option<Duration>,
    allowlist: &McpAllowListConfig,
) -> Result<McpResourceData> {
    provider.read_resource(uri, timeout, allowlist).await
}
