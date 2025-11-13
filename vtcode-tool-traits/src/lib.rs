//! Core traits and types for the VTCode tool system
//!
//! This crate provides the foundational types, traits, and interfaces needed by the VTCode
//! tool system. It was extracted from `vtcode-core` to break circular dependencies and
//! enable modular architecture.
//!
//! # Overview
//!
//! The tool system is built around several key concepts:
//!
//! - **Tools**: Reusable components that perform specific operations (file I/O, command execution, etc.)
//! - **Tool Executor**: Manages tool registration and execution
//! - **Tool Policy**: Controls which tools can be executed and under what conditions
//! - **Tool Requests/Responses**: Structured data for tool invocation
//!
//! # Example
//!
//! ```rust,ignore
//! use vtcode_tool_traits::{Tool, ToolRequest, ToolResponse, ToolPolicy};
//! use async_trait::async_trait;
//! use serde_json::json;
//!
//! struct MyCustomTool;
//!
//! #[async_trait]
//! impl Tool for MyCustomTool {
//!     async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
//!         Ok(json!({"result": "success"}))
//!     }
//!
//!     fn name(&self) -> &'static str {
//!         "my_custom_tool"
//!     }
//!
//!     fn description(&self) -> &'static str {
//!         "A custom tool that does something useful"
//!     }
//! }
//! ```

pub mod error;
pub mod mcp;
pub mod policy;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use error::{classify_error, ToolErrorType, ToolExecutionError, ToolResult};
pub use mcp::{McpClientStatus, McpToolExecutor, McpToolInfo};
pub use policy::ToolPolicy;
pub use traits::{Tool, ToolExecutor, ToolValidator};
pub use types::{ToolMetadata, ToolParameter, ToolRequest, ToolResponse};
