//! Core traits for the composable tool system

use async_trait::async_trait;
use serde_json::Value;

use crate::error::ToolResult;

/// Core trait for all agent tools
///
/// This trait defines the interface that all tools must implement.
/// Tools are the primary way agents interact with the system and external resources.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with given arguments
    ///
    /// # Arguments
    /// * `args` - JSON value containing the tool's input parameters
    ///
    /// # Returns
    /// A JSON value representing the tool's output on success, or an error
    async fn execute(&self, args: Value) -> ToolResult<Value>;

    /// Get the tool's unique name
    ///
    /// This name is used to identify and invoke the tool from the registry.
    fn name(&self) -> &'static str;

    /// Get the tool's human-readable description
    ///
    /// This description helps agents understand what the tool does and when to use it.
    fn description(&self) -> &'static str;

    /// Validate arguments before execution
    ///
    /// Override this method to perform tool-specific validation of input parameters.
    /// The default implementation accepts all arguments.
    ///
    /// # Arguments
    /// * `args` - JSON value containing the tool's input parameters
    ///
    /// # Returns
    /// Ok(()) if validation succeeds, or an error describing what's wrong
    fn validate_args(&self, _args: &Value) -> ToolResult<()> {
        // Default implementation - tools can override for specific validation
        Ok(())
    }
}

/// Main tool executor that coordinates all tools
///
/// This trait defines the interface for tool registries and executors
/// that manage collections of tools and route execution requests.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    ///
    /// # Arguments
    /// * `name` - The name of the tool to execute
    /// * `args` - JSON value containing the tool's input parameters
    ///
    /// # Returns
    /// The tool's output as a JSON value, or an error if execution fails
    async fn execute_tool(&self, name: &str, args: Value) -> ToolResult<Value>;

    /// List all available tools
    ///
    /// # Returns
    /// A vector of tool names that can be executed
    fn available_tools(&self) -> Vec<String>;

    /// Check if a tool exists
    ///
    /// # Arguments
    /// * `name` - The name of the tool to check
    ///
    /// # Returns
    /// true if the tool is available, false otherwise
    fn has_tool(&self, name: &str) -> bool;
}

/// Trait for tools that support validation of their inputs
///
/// This trait provides a hook for tools to perform complex validation
/// that goes beyond basic type checking.
pub trait ToolValidator: Send + Sync {
    /// Validate tool input parameters
    ///
    /// # Arguments
    /// * `args` - The input parameters to validate
    ///
    /// # Returns
    /// Ok(()) if validation succeeds, or an error describing validation failures
    fn validate(&self, args: &Value) -> ToolResult<()>;

    /// Get the JSON schema for this tool's parameters
    ///
    /// # Returns
    /// An optional JSON schema that describes the expected input format
    fn schema(&self) -> Option<Value> {
        None
    }
}
