//! Error context helpers
//!
//! This module provides extension traits for adding rich context to errors.
//! Using these helpers makes error messages more informative and easier to debug.
//!
//! # Examples
//!
//! ## File Operations
//!
//! ```rust,ignore
//! use std::fs;
//! use std::path::Path;
//! use crate::errors::context::FileErrorExt;
//!
//! let path = Path::new("config.toml");
//! let content = fs::read_to_string(&path)
//!     .with_file_read_context(&path)?;
//! ```
//!
//! ## Command Execution
//!
//! ```rust,ignore
//! use crate::errors::context::CommandErrorExt;
//!
//! let output = std::process::Command::new("git")
//!     .arg("status")
//!     .output()
//!     .with_command_context("git status")?;
//! ```
//!
//! ## Tool Execution
//!
//! ```rust,ignore
//! use crate::errors::context::ToolErrorExt;
//!
//! let result = execute_tool(name, args)
//!     .with_tool_context("grep_file")?;
//! ```

use anyhow::{Context, Result};
use std::path::Path;

/// Extension trait for adding file context to errors
pub trait FileErrorExt<T> {
    /// Add context for file reading operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let content = fs::read_to_string(&path)
    ///     .with_file_read_context(&path)?;
    /// ```
    fn with_file_read_context(self, path: &Path) -> Result<T>;

    /// Add context for file writing operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fs::write(&path, content)
    ///     .with_file_write_context(&path)?;
    /// ```
    fn with_file_write_context(self, path: &Path) -> Result<T>;

    /// Add context for file deletion operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fs::remove_file(&path)
    ///     .with_file_delete_context(&path)?;
    /// ```
    fn with_file_delete_context(self, path: &Path) -> Result<T>;

    /// Add context for directory creation operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fs::create_dir_all(&path)
    ///     .with_dir_create_context(&path)?;
    /// ```
    fn with_dir_create_context(self, path: &Path) -> Result<T>;
}

impl<T, E> FileErrorExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_file_read_context(self, path: &Path) -> Result<T> {
        self.with_context(|| format!("Failed to read file: {}", path.display()))
    }

    fn with_file_write_context(self, path: &Path) -> Result<T> {
        self.with_context(|| format!("Failed to write file: {}", path.display()))
    }

    fn with_file_delete_context(self, path: &Path) -> Result<T> {
        self.with_context(|| format!("Failed to delete file: {}", path.display()))
    }

    fn with_dir_create_context(self, path: &Path) -> Result<T> {
        self.with_context(|| format!("Failed to create directory: {}", path.display()))
    }
}

/// Extension trait for adding command context to errors
pub trait CommandErrorExt<T> {
    /// Add context for command execution
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let output = std::process::Command::new("git")
    ///     .arg("status")
    ///     .output()
    ///     .with_command_context("git status")?;
    /// ```
    fn with_command_context(self, command: &str) -> Result<T>;

    /// Add context for PTY operations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pty_session.execute(cmd)
    ///     .with_pty_context("session_123")?;
    /// ```
    fn with_pty_context(self, session_id: &str) -> Result<T>;
}

impl<T, E> CommandErrorExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_command_context(self, command: &str) -> Result<T> {
        self.with_context(|| format!("Failed to execute command: {}", command))
    }

    fn with_pty_context(self, session_id: &str) -> Result<T> {
        self.with_context(|| format!("PTY session error ({})", session_id))
    }
}

/// Extension trait for adding tool context to errors
pub trait ToolErrorExt<T> {
    /// Add context for tool execution
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// registry.execute_tool("grep_file", args)
    ///     .with_tool_context("grep_file")?;
    /// ```
    fn with_tool_context(self, tool_name: &str) -> Result<T>;

    /// Add context for tool arguments
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// parse_args(raw_args)
    ///     .with_tool_args_context("grep_file", &raw_args)?;
    /// ```
    fn with_tool_args_context(self, tool_name: &str, args: &str) -> Result<T>;
}

impl<T, E> ToolErrorExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_tool_context(self, tool_name: &str) -> Result<T> {
        self.with_context(|| format!("Tool execution failed: {}", tool_name))
    }

    fn with_tool_args_context(self, tool_name: &str, args: &str) -> Result<T> {
        self.with_context(|| format!("Tool {} failed with args: {}", tool_name, args))
    }
}

/// Extension trait for adding LLM provider context to errors
pub trait ProviderErrorExt<T> {
    /// Add context for provider API calls
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// client.chat_completion(request)
    ///     .with_provider_context("anthropic")?;
    /// ```
    fn with_provider_context(self, provider: &str) -> Result<T>;

    /// Add context for model selection
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// load_model(model_name)
    ///     .with_model_context("claude-3-5-sonnet")?;
    /// ```
    fn with_model_context(self, model: &str) -> Result<T>;
}

impl<T, E> ProviderErrorExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_provider_context(self, provider: &str) -> Result<T> {
        self.with_context(|| format!("Provider API error ({})", provider))
    }

    fn with_model_context(self, model: &str) -> Result<T> {
        self.with_context(|| format!("Model error ({})", model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::path::PathBuf;

    #[test]
    fn test_file_read_context() {
        let path = PathBuf::from("test.txt");
        let result: Result<()> = Err(anyhow!("test error")).with_file_read_context(&path);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read file"));
        assert!(error_msg.contains("test.txt"));
    }

    #[test]
    fn test_file_write_context() {
        let path = PathBuf::from("/tmp/output.txt");
        let result: Result<()> = Err(anyhow!("test error")).with_file_write_context(&path);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to write file"));
        assert!(error_msg.contains("output.txt"));
    }

    #[test]
    fn test_command_context() {
        let result: Result<()> = Err(anyhow!("test error")).with_command_context("git status");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to execute command"));
        assert!(error_msg.contains("git status"));
    }

    #[test]
    fn test_pty_context() {
        let result: Result<()> = Err(anyhow!("test error")).with_pty_context("session_123");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("PTY session error"));
        assert!(error_msg.contains("session_123"));
    }

    #[test]
    fn test_tool_context() {
        let result: Result<()> = Err(anyhow!("test error")).with_tool_context("grep_file");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Tool execution failed"));
        assert!(error_msg.contains("grep_file"));
    }

    #[test]
    fn test_tool_args_context() {
        let result: Result<()> =
            Err(anyhow!("test error")).with_tool_args_context("grep_file", "pattern: foo");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("grep_file"));
        assert!(error_msg.contains("pattern: foo"));
    }

    #[test]
    fn test_provider_context() {
        let result: Result<()> = Err(anyhow!("test error")).with_provider_context("anthropic");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Provider API error"));
        assert!(error_msg.contains("anthropic"));
    }

    #[test]
    fn test_model_context() {
        let result: Result<()> =
            Err(anyhow!("test error")).with_model_context("claude-3-5-sonnet");

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Model error"));
        assert!(error_msg.contains("claude-3-5-sonnet"));
    }

    #[test]
    fn test_success_passthrough() {
        // Test that successful results pass through unchanged
        let result: Result<i32> = Ok(42);
        let path = PathBuf::from("test.txt");
        assert_eq!(result.with_file_read_context(&path).unwrap(), 42);
    }
}
