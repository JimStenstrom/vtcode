//! Model Context Protocol (MCP) client module.
//!
//! **Note**: As of vtcode 0.43.6, the MCP implementation has been extracted into
//! the standalone [`vtcode-mcp`](https://docs.rs/vtcode-mcp) crate as part of
//! Phase 2 of the architecture transformation.
//!
//! This module re-exports all types from `vtcode-mcp` for backward compatibility,
//! so existing code using `vtcode_core::mcp` will continue to work without changes.
//!
//! # Migration
//!
//! For new code, you can choose to either:
//! 1. Continue using `vtcode_core::mcp` (recommended for vtcode integration)
//! 2. Use `vtcode_mcp` directly (for standalone MCP usage)
//!
//! ```rust,ignore
//! // Option 1: Through vtcode-core (unchanged)
//! use vtcode_core::mcp::{McpClient, McpToolExecutor};
//!
//! // Option 2: Direct vtcode-mcp usage
//! use vtcode_mcp::{McpClient, McpToolExecutor};
//! ```
//!
//! Both approaches work identically. See [`vtcode-mcp` documentation](https://docs.rs/vtcode-mcp)
//! and `vtcode-mcp/MIGRATION.md` for details.
//!
//! # Re-exported Types
//!
//! All public types from `vtcode-mcp` are re-exported here:
//! - [`McpClient`] - Main MCP client for managing providers
//! - [`McpToolExecutor`] - Trait for executing MCP tools
//! - [`ToolDiscovery`] - Progressive tool discovery service
//! - [`ValidatedMcpClientConfig`] - Enhanced configuration validation
//! - CLI commands and configuration types
//!
//! See `vtcode-mcp` crate documentation for detailed API reference.

pub use vtcode_mcp::*;
