//! Tool name resolution and lookup

use super::core::ToolRegistry;
use super::registration::ToolRegistration;
use anyhow::Result;
use tracing::warn;
use vtcode_mcp::{McpToolExecutor, McpToolInfo};

impl ToolRegistry {
    /// Register a new tool with the registry
    ///
    /// # Arguments
    /// * `registration` - The tool registration to add
    ///
    /// # Returns
    /// `Result<()>` indicating success or an error if the tool is already registered
    pub fn register_tool(&mut self, registration: ToolRegistration) -> Result<()> {
        // Clone the name since we need it after moving registration
        let tool_name = registration.name().to_string();

        // Register the tool
        self.inventory.register_tool(registration)?;

        // Register any aliases for the tool
        for alias in crate::tools::names::tool_aliases(&tool_name) {
            self.inventory.add_alias(alias, &tool_name);
        }

        Ok(())
    }

    /// Get a list of all available tools, including MCP tools
    ///
    /// # Returns
    /// A `Vec<String>` containing the names of all available tools
    pub async fn available_tools(&self) -> Vec<String> {
        let mut tools = self.inventory.available_tools();

        // Add MCP tools if available
        if let Some(mcp_client) = &self.mcp_client {
            if let Ok(mcp_tools) = mcp_client.list_mcp_tools().await {
                for tool in mcp_tools {
                    tools.push(format!("mcp_{}", tool.name));
                }
            }
        }

        tools.sort();
        tools
    }

    /// Check if a tool with the given name is registered
    ///
    /// # Arguments
    /// * `name` - The name of the tool to check
    ///
    /// # Returns
    /// `bool` indicating whether the tool exists (including aliases)
    pub async fn has_tool(&self, name: &str) -> bool {
        // First check the main tool registry
        if self.inventory.has_tool(name) {
            return true;
        }

        // If not found, check if it's an MCP tool
        if name.starts_with("mcp_") {
            let tool_name = &name[4..];
            if self.find_mcp_provider(tool_name).is_some() {
                return true;
            }

            if let Some(mcp_client) = &self.mcp_client {
                if let Ok(true) = mcp_client.has_mcp_tool(tool_name).await {
                    return true;
                }
                // Check if it's an alias
                if let Some(resolved_name) = self.resolve_mcp_tool_alias(tool_name).await {
                    if resolved_name != tool_name {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if an MCP tool exists
    pub async fn has_mcp_tool(&mut self, tool_name: &str) -> bool {
        if self
            .mcp_tool_index
            .values()
            .any(|tools| tools.iter().any(|candidate| candidate == tool_name))
        {
            self.mcp_tool_presence.insert(tool_name.to_string(), true);
            return true;
        }

        if let Some(cached) = self.mcp_tool_presence.get(tool_name) {
            return *cached;
        }

        let Some(mcp_client) = &self.mcp_client else {
            self.mcp_tool_presence.insert(tool_name.to_string(), false);
            return false;
        };

        match mcp_client.has_mcp_tool(tool_name).await {
            Ok(result) => {
                self.mcp_tool_presence.insert(tool_name.to_string(), result);
                result
            }
            Err(err) => {
                warn!(
                    tool = tool_name,
                    error = %err,
                    "failed to query MCP tool presence"
                );
                self.mcp_tool_presence.insert(tool_name.to_string(), false);
                false
            }
        }
    }

    /// List all MCP tools
    pub async fn list_mcp_tools(&self) -> Result<Vec<McpToolInfo>> {
        if let Some(mcp_client) = &self.mcp_client {
            mcp_client.list_mcp_tools().await
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) async fn resolve_mcp_tool_alias(&self, tool_name: &str) -> Option<String> {
        let Some(mcp_client) = &self.mcp_client else {
            return None;
        };

        let normalized = normalize_mcp_tool_identifier(tool_name);
        if normalized.is_empty() {
            return None;
        }

        let tools = match mcp_client.list_mcp_tools().await {
            Ok(list) => list,
            Err(err) => {
                warn!(
                    "Failed to list MCP tools while resolving alias '{}': {}",
                    tool_name, err
                );
                return None;
            }
        };

        for tool in tools {
            if normalize_mcp_tool_identifier(&tool.name) == normalized {
                return Some(tool.name);
            }
        }

        None
    }
}

fn normalize_mcp_tool_identifier(value: &str) -> String {
    let mut normalized = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mcp_tool_identifiers() {
        assert_eq!(
            normalize_mcp_tool_identifier("sequential-thinking"),
            "sequentialthinking"
        );
        assert_eq!(
            normalize_mcp_tool_identifier("Context7.Lookup"),
            "context7lookup"
        );
        assert_eq!(normalize_mcp_tool_identifier("alpha_beta"), "alphabeta");
    }
}
