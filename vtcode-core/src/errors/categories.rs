//! Error categorization and handling guidelines
//!
//! This module defines error severity levels and provides comprehensive
//! guidelines for when to use different error handling patterns.

/// Error severity levels
///
/// Use these to categorize errors and determine appropriate handling strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Critical - application cannot continue, must exit
    ///
    /// Examples:
    /// - Failed to initialize required system resources
    /// - Corrupted critical data structures
    /// - Unrecoverable system errors
    Critical,

    /// Error - operation failed, but app can continue
    ///
    /// Examples:
    /// - Tool execution failed
    /// - File not found
    /// - Network request failed
    Error,

    /// Warning - potential issue, operation succeeded
    ///
    /// Examples:
    /// - Cache load failed, using defaults
    /// - Deprecated API used
    /// - Performance degradation detected
    Warning,

    /// Info - informational, no issue
    ///
    /// Examples:
    /// - Fallback value used
    /// - Retry succeeded
    /// - Alternative method used
    Info,
}

impl Severity {
    /// Returns true if this severity level should cause the application to exit
    pub fn is_fatal(&self) -> bool {
        matches!(self, Severity::Critical)
    }

    /// Returns true if this severity level should be logged
    pub fn should_log(&self) -> bool {
        true // All severity levels should be logged
    }

    /// Returns the appropriate log level for this severity
    pub fn log_level(&self) -> &'static str {
        match self {
            Severity::Critical => "error",
            Severity::Error => "error",
            Severity::Warning => "warn",
            Severity::Info => "info",
        }
    }
}

/// Error Handling Guidelines
///
/// This module contains comprehensive guidelines for error handling patterns.
/// Review these guidelines before writing error handling code.
pub mod guidelines {
    //! # Error Handling Guidelines
    //!
    //! ## Overview
    //!
    //! vtcode-core uses `anyhow::Result` for error handling with structured error types
    //! from the `errors` module. These guidelines help ensure consistent error handling
    //! across the codebase.
    //!
    //! ## Quick Reference
    //!
    //! - ✅ **Always use `?` operator** - Propagate errors with context
    //! - ✅ **Always add context** - Use `.with_context()` or extension traits
    //! - ✅ **Log recoverable failures** - Use `match` or `try_with_fallback`
    //! - ❌ **Never use `.unwrap()`** - Except in tests/examples
    //! - ❌ **Never use `.expect()`** - Unless provably infallible with comment
    //! - ❌ **Never swallow errors silently** - Always log or return
    //!
    //! ## When to use `.unwrap()`
    //!
    //! **NEVER in production code!**
    //!
    //! Only acceptable in:
    //! - Tests
    //! - Examples
    //! - Prototypes marked with TODO
    //!
    //! ```rust,ignore
    //! // ❌ BAD - will panic in production
    //! let value = map.get(&key).unwrap();
    //!
    //! // ✅ GOOD - returns error to caller
    //! let value = map.get(&key)
    //!     .ok_or_else(|| anyhow!("Missing key: {}", key))?;
    //! ```
    //!
    //! ## When to use `.expect()`
    //!
    //! Only for operations that are **provably infallible** with a clear comment
    //! explaining why.
    //!
    //! Acceptable cases:
    //! - Compile-time constants (regex, static initialization)
    //! - Mathematically impossible failures
    //!
    //! ```rust,ignore
    //! use once_cell::sync::Lazy;
    //! use regex::Regex;
    //!
    //! // ✅ GOOD - regex is hardcoded and must compile
    //! static RE: Lazy<Regex> = Lazy::new(|| {
    //!     Regex::new(r"\d+").expect("Hardcoded regex must compile")
    //! });
    //!
    //! // ✅ GOOD - lock poisoning is unrecoverable
    //! let data = mutex.lock().expect("Mutex poisoned");
    //! ```
    //!
    //! ## When to use `?` operator (ALWAYS PREFER THIS!)
    //!
    //! The `?` operator should be your default choice for error handling.
    //! Always add context using `anyhow::Context` or our extension traits.
    //!
    //! ```rust,ignore
    //! use anyhow::Context;
    //! use crate::errors::context::FileErrorExt;
    //!
    //! // ✅ GOOD - with custom context
    //! let content = fs::read_to_string(&path)
    //!     .with_context(|| format!("Failed to read: {}", path.display()))?;
    //!
    //! // ✅ GOOD - with extension trait
    //! let content = fs::read_to_string(&path)
    //!     .with_file_read_context(&path)?;
    //!
    //! // ❌ BAD - no context
    //! let content = fs::read_to_string(&path)?;
    //! ```
    //!
    //! ## When to use `.unwrap_or()` / `.unwrap_or_default()`
    //!
    //! Use when:
    //! - A fallback value is acceptable
    //! - The failure is not an error condition
    //! - But **consider logging the None/Err case!**
    //!
    //! ```rust,ignore
    //! // ✅ GOOD - timeout has a reasonable default
    //! let timeout = config.timeout.unwrap_or(300);
    //!
    //! // ✅ GOOD - empty list is acceptable
    //! let items = optional_items.unwrap_or_default();
    //!
    //! // ⚠️  CONSIDER - should we log that config is missing?
    //! let config = try_with_fallback(
    //!     || load_config(),
    //!     Config::default(),
    //!     "Config not found, using defaults",
    //! );
    //! ```
    //!
    //! ## When to use `match` / `if let Ok()`
    //!
    //! Use when you need **custom error handling logic**:
    //!
    //! ```rust,ignore
    //! use tracing::warn;
    //!
    //! // ✅ GOOD - custom recovery logic
    //! match load_cache() {
    //!     Ok(cache) => use_cache(cache),
    //!     Err(e) => {
    //!         warn!("Cache unavailable: {}", e);
    //!         rebuild_cache()
    //!     }
    //! }
    //!
    //! // ✅ GOOD - different handling per error type
    //! match execute_command(cmd) {
    //!     Ok(output) => process_output(output),
    //!     Err(e) if e.is_permission_denied() => {
    //!         warn!("Permission denied, trying with sudo");
    //!         execute_with_sudo(cmd)?
    //!     }
    //!     Err(e) => return Err(e),
    //! }
    //! ```
    //!
    //! ## When to use custom error types
    //!
    //! Use structured error types from `crate::errors` for domain-specific errors:
    //!
    //! ```rust,ignore
    //! use crate::errors::{ToolError, VTCodeError};
    //!
    //! // ✅ GOOD - specific error type
    //! fn find_tool(name: &str) -> Result<Tool, VTCodeError> {
    //!     if !tool_exists(name) {
    //!         return Err(ToolError::NotFound(name.to_string()).into());
    //!     }
    //!     Ok(load_tool(name))
    //! }
    //! ```
    //!
    //! ## Error Context Best Practices
    //!
    //! Always provide enough context to debug the error:
    //!
    //! ```rust,ignore
    //! // ❌ BAD - what file? what failed?
    //! fs::read_to_string(path)?
    //!
    //! // ⚠️  OKAY - has file path
    //! fs::read_to_string(path)
    //!     .with_file_read_context(&path)?
    //!
    //! // ✅ GOOD - has context about why we're reading it
    //! fs::read_to_string(&config_path)
    //!     .with_context(|| {
    //!         format!("Failed to read config file for project: {}", project_name)
    //!     })?
    //! ```
    //!
    //! ## Recoverable vs Non-Recoverable Errors
    //!
    //! ### Non-Recoverable (propagate with `?`)
    //!
    //! - User requested file doesn't exist
    //! - Invalid tool arguments
    //! - Network request failed
    //! - Parse errors
    //!
    //! ### Recoverable (use fallback or alternative)
    //!
    //! - Cache load failed (use empty cache)
    //! - Theme load failed (use default theme)
    //! - Optional config missing (use defaults)
    //! - Telemetry failed (continue without telemetry)
    //!
    //! ## Testing Error Paths
    //!
    //! Always test error conditions:
    //!
    //! ```rust,ignore
    //! #[test]
    //! fn test_missing_file_error() {
    //!     let result = read_config("nonexistent.toml");
    //!     assert!(result.is_err());
    //!
    //!     let error = result.unwrap_err();
    //!     assert!(error.to_string().contains("Failed to read"));
    //!     assert!(error.to_string().contains("nonexistent.toml"));
    //! }
    //!
    //! #[test]
    //! fn test_invalid_arguments_error() {
    //!     let result = execute_tool("grep", serde_json::json!({}));
    //!     assert!(result.is_err());
    //!
    //!     // Verify error type
    //!     match result.unwrap_err().downcast_ref::<VTCodeError>() {
    //!         Some(VTCodeError::Tool(ToolError::InvalidArguments(_))) => (),
    //!         _ => panic!("Wrong error type"),
    //!     }
    //! }
    //! ```
    //!
    //! ## Common Anti-Patterns
    //!
    //! ### ❌ Swallowing Errors Silently
    //!
    //! ```rust,ignore
    //! // ❌ BAD - error is completely ignored
    //! if let Ok(data) = load_data() {
    //!     process(data);
    //! }
    //! // What if load_data() failed? User has no idea!
    //!
    //! // ✅ GOOD - error is logged
    //! match load_data() {
    //!     Ok(data) => process(data),
    //!     Err(e) => warn!("Failed to load data: {}", e),
    //! }
    //! ```
    //!
    //! ### ❌ Using unwrap() in production
    //!
    //! ```rust,ignore
    //! // ❌ BAD - will panic
    //! let config = config_map.get(&key).unwrap();
    //!
    //! // ✅ GOOD - returns error
    //! let config = config_map.get(&key)
    //!     .ok_or_else(|| anyhow!("Missing config: {}", key))?;
    //! ```
    //!
    //! ### ❌ Poor Error Messages
    //!
    //! ```rust,ignore
    //! // ❌ BAD - no context
    //! .context("Failed")?
    //!
    //! // ⚠️  OKAY - has context but not specific
    //! .context("Failed to read file")?
    //!
    //! // ✅ GOOD - specific and actionable
    //! .with_context(|| format!("Failed to read config file: {}", path.display()))?
    //! ```

    // This is a documentation module, no code needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_severity_is_fatal() {
        assert!(Severity::Critical.is_fatal());
        assert!(!Severity::Error.is_fatal());
        assert!(!Severity::Warning.is_fatal());
        assert!(!Severity::Info.is_fatal());
    }

    #[test]
    fn test_severity_should_log() {
        // All severities should be logged
        assert!(Severity::Critical.should_log());
        assert!(Severity::Error.should_log());
        assert!(Severity::Warning.should_log());
        assert!(Severity::Info.should_log());
    }

    #[test]
    fn test_severity_log_level() {
        assert_eq!(Severity::Critical.log_level(), "error");
        assert_eq!(Severity::Error.log_level(), "error");
        assert_eq!(Severity::Warning.log_level(), "warn");
        assert_eq!(Severity::Info.log_level(), "info");
    }
}
