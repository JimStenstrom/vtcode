//! MCP (Model Context Protocol) tool executor traits and types
//!
//! This module provides the trait interface for executing MCP tools, allowing
//! the execution layer to interact with MCP providers without depending on
//! the full MCP implementation.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Information about an MCP tool exposed by a provider.
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub provider: String,
    pub input_schema: Value,
}

/// Snapshot describing the MCP client at runtime.
#[derive(Debug, Clone)]
pub struct McpClientStatus {
    pub enabled: bool,
    pub provider_count: usize,
    pub active_connections: usize,
    pub configured_providers: Vec<String>,
}

/// Trait abstraction used by the tool registry and execution layer to talk to the MCP client.
///
/// This trait provides a clean interface for executing MCP tools without requiring
/// direct dependencies on the MCP implementation details.
#[async_trait]
pub trait McpToolExecutor: Send + Sync {
    /// Execute an MCP tool by name with the given arguments.
    async fn execute_mcp_tool(&self, tool_name: &str, args: Value) -> Result<Value>;

    /// List all available MCP tools from all providers.
    async fn list_mcp_tools(&self) -> Result<Vec<McpToolInfo>>;

    /// Check if a specific MCP tool is available.
    async fn has_mcp_tool(&self, tool_name: &str) -> Result<bool>;

    /// Get the current status of the MCP client.
    fn get_status(&self) -> McpClientStatus;
}
