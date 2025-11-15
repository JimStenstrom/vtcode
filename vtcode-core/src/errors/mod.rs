//! Error handling types and utilities for vtcode-core
//!
//! This module provides a structured error hierarchy and helper functions
//! for consistent error handling across the codebase.
//!
//! # Error Type Hierarchy
//!
//! - **VTCodeError** - Top-level error type
//!   - **UserError** - Invalid input, wrong arguments
//!   - **SystemError** - I/O, network, OS errors
//!   - **ConfigError** - Configuration issues
//!   - **ToolError** - Tool execution failures
//!   - **ProviderError** - LLM provider errors
//!   - **Internal** - Should never happen
//!
//! # Usage
//!
//! ```rust,ignore
//! use vtcode_core::errors::{VTCodeError, ToolError};
//! use anyhow::Result;
//!
//! fn execute_tool(name: &str) -> Result<()> {
//!     if !tool_exists(name) {
//!         return Err(ToolError::NotFound(name.to_string()).into());
//!     }
//!     Ok(())
//! }
//! ```

pub mod categories;
pub mod context;
pub mod recovery;

use thiserror::Error;

/// Top-level error type for vtcode-core
#[derive(Debug, Error)]
pub enum VTCodeError {
    /// User-facing errors (invalid input, wrong arguments)
    #[error("User error: {0}")]
    User(#[from] UserError),

    /// System errors (file I/O, network, OS)
    #[error("System error: {0}")]
    System(#[from] SystemError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Tool execution errors
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    /// LLM provider errors
    #[error("LLM provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Internal errors (should never happen in production)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// User-facing errors
#[derive(Debug, Error)]
pub enum UserError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// System errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing configuration: {0}")]
    Missing(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Tool execution errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid tool arguments: {0}")]
    InvalidArguments(String),
}

/// LLM provider errors
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider unavailable: {0}")]
    Unavailable(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_error_display() {
        let error = UserError::InvalidArgument("test".to_string());
        assert_eq!(error.to_string(), "Invalid argument: test");
    }

    #[test]
    fn test_tool_error_conversion() {
        let tool_error = ToolError::NotFound("grep".to_string());
        let vtcode_error: VTCodeError = tool_error.into();
        assert!(vtcode_error.to_string().contains("Tool not found: grep"));
    }

    #[test]
    fn test_provider_error_display() {
        let error = ProviderError::RateLimit;
        assert_eq!(error.to_string(), "Rate limit exceeded");
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::Missing("api_key".to_string());
        assert_eq!(error.to_string(), "Missing configuration: api_key");
    }

    #[test]
    fn test_system_error_display() {
        let error = SystemError::Network("Connection timeout".to_string());
        assert_eq!(error.to_string(), "Network error: Connection timeout");
    }

    #[test]
    fn test_error_hierarchy() {
        // Test that errors can be converted to VTCodeError
        let user_err: VTCodeError = UserError::InvalidInput("bad".to_string()).into();
        assert!(user_err.to_string().contains("Invalid input"));

        let config_err: VTCodeError = ConfigError::Invalid("bad config".to_string()).into();
        assert!(config_err.to_string().contains("Invalid configuration"));

        let tool_err: VTCodeError = ToolError::ExecutionFailed("failed".to_string()).into();
        assert!(tool_err.to_string().contains("Tool execution failed"));
    }
}
