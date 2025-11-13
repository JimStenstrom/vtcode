//! # vtcode-mcp
//!
//! Model Context Protocol (MCP) client implementation for VTCode.
//!
//! This crate provides a complete MCP client built on top of the rmcp library,
//! with VTCode-specific features like:
//! - Multi-provider management
//! - Tool allowlisting and security policies
//! - Configuration management
//! - CLI commands for provider management
//! - Tool discovery and search functionality
//!
//! ## Features
//!
//! - **Multi-transport support**: stdio and HTTP transports
//! - **Provider management**: Add, remove, and configure multiple MCP providers
//! - **Security**: Tool and resource allowlisting, argument validation
//! - **Tool discovery**: Progressive disclosure of tools with search functionality
//! - **Elicitation support**: Handle user input requests from MCP providers
//!
//! ## Example
//!
//! ```ignore
//! use vtcode_mcp::{McpClient, McpClientConfig};
//!
//! let config = McpClientConfig::default();
//! let mut client = McpClient::new(config);
//! client.initialize().await?;
//!
//! // List available tools
//! let tools = client.list_tools().await?;
//!
//! // Execute a tool
//! let result = client.execute_tool("read_file", args).await?;
//! ```

pub mod client;
pub mod cli;
pub mod enhanced_config;
pub mod tool_discovery;

// Re-export main types for convenience
pub use client::{
    McpClient, McpClientStatus, McpElicitationHandler, McpElicitationRequest,
    McpElicitationResponse, McpPromptDetail, McpPromptInfo, McpResourceData, McpResourceInfo,
    McpToolExecutor, McpToolInfo, validate_mcp_config,
};
pub use cli::{
    AddArgs, GetArgs, ListArgs, LoginArgs, LogoutArgs, McpCommands, RemoveArgs,
    handle_mcp_command,
};
pub use enhanced_config::{
    EnhancedMcpSecurityConfig, EnhancedMcpToolConfig, McpRateLimitConfig, McpValidationConfig,
    ValidatedMcpClientConfig, ValidationError,
};
pub use tool_discovery::{DetailLevel, ToolDiscovery, ToolDiscoveryResult};

// Re-export from rmcp for convenience
pub use rmcp::model::ElicitationAction;
