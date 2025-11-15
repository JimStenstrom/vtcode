//! MCP client management and integration

use super::core::ToolRegistry;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
use vtcode_mcp::{McpClient, McpToolExecutor};

impl ToolRegistry {
    /// Set the MCP client for this registry
    pub fn with_mcp_client(mut self, mcp_client: Arc<McpClient>) -> Self {
        self.mcp_client = Some(mcp_client);
        self.mcp_tool_index.clear();
        self.mcp_tool_presence.clear();
        self
    }

    /// Attach an MCP client without consuming the registry
    pub fn set_mcp_client(&mut self, mcp_client: Arc<McpClient>) {
        self.mcp_client = Some(mcp_client);
        self.mcp_tool_index.clear();
        self.mcp_tool_presence.clear();
    }

    /// Get the MCP client if available
    pub fn mcp_client(&self) -> Option<&Arc<McpClient>> {
        self.mcp_client.as_ref()
    }

    /// Refresh MCP tools (reconnect to providers and update tool lists)
    pub async fn refresh_mcp_tools(&mut self) -> Result<()> {
        if let Some(mcp_client) = &self.mcp_client {
            debug!(
                "Refreshing MCP tools for {} providers",
                mcp_client.get_status().provider_count
            );

            let tools = mcp_client.list_mcp_tools().await?;
            let mut provider_map: HashMap<String, Vec<String>> = HashMap::new();

            for tool in tools {
                provider_map
                    .entry(tool.provider.clone())
                    .or_default()
                    .push(tool.name.clone());
            }

            for tools in provider_map.values_mut() {
                tools.sort();
                tools.dedup();
            }

            self.mcp_tool_index = provider_map;
            self.mcp_tool_presence.clear();
            for tools in self.mcp_tool_index.values() {
                for tool in tools {
                    self.mcp_tool_presence.insert(tool.clone(), true);
                }
            }

            if let Some(allowlist) = self
                .policy_gateway
                .update_mcp_tools(&self.mcp_tool_index)
                .await?
            {
                mcp_client.update_allowlist(allowlist);
            }

            self.sync_policy_catalog().await;
            Ok(())
        } else {
            debug!("No MCP client configured, nothing to refresh");
            Ok(())
        }
    }
}
