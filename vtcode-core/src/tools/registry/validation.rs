//! Common validation utilities for tool executors
//!
//! This module provides reusable validation functions to ensure consistent
//! security and safety checks across all tool executors.

use anyhow::{Result, anyhow};

/// Maximum allowed results for search operations to prevent excessive resource usage
pub const MAX_ALLOWED_RESULTS: usize = 1000;

/// Maximum allowed file size (100MB in bytes)
pub const MAX_ALLOWED_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Maximum allowed context lines for grep operations
pub const MAX_ALLOWED_CONTEXT: usize = 20;

/// Validates that a path is relative and doesn't contain path traversal attempts
///
/// # Security
/// This validation prevents directory traversal attacks by ensuring:
/// - Path doesn't contain ".." segments
/// - Path doesn't start with "/" (absolute path)
///
/// # Arguments
/// * `path` - The path string to validate
/// * `field_name` - Name of the field for error messages (e.g., "path", "glob_pattern")
///
/// # Returns
/// * `Ok(())` if the path is valid
/// * `Err` if the path contains security issues
///
/// # Examples
/// ```
/// use vtcode_core::tools::registry::validation::validate_relative_path;
///
/// // Valid paths
/// assert!(validate_relative_path("src/main.rs", "path").is_ok());
/// assert!(validate_relative_path("./config.toml", "path").is_ok());
///
/// // Invalid paths
/// assert!(validate_relative_path("../etc/passwd", "path").is_err());
/// assert!(validate_relative_path("/etc/passwd", "path").is_err());
/// ```
pub fn validate_relative_path(path: &str, field_name: &str) -> Result<()> {
    if path.contains("..") || path.starts_with('/') {
        return Err(anyhow!(
            "{} must be a relative path and cannot contain '..' or start with '/'",
            field_name
        ));
    }
    Ok(())
}

/// Validates a numeric limit against a maximum allowed value
///
/// # Arguments
/// * `value` - The optional value to validate
/// * `max_allowed` - The maximum allowed value
/// * `field_name` - Name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the value is within bounds or None
/// * `Err` if the value exceeds the maximum or is zero
pub fn validate_numeric_limit(
    value: Option<usize>,
    max_allowed: usize,
    field_name: &str,
) -> Result<()> {
    if let Some(val) = value {
        if val > max_allowed {
            return Err(anyhow!(
                "{} ({}) exceeds the maximum allowed value of {}",
                field_name,
                val,
                max_allowed
            ));
        }
        if val == 0 {
            return Err(anyhow!("{} must be greater than 0", field_name));
        }
    }
    Ok(())
}

/// Validates max_results parameter against the allowed maximum
///
/// # Arguments
/// * `max_results` - Optional maximum results value
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err` if exceeds MAX_ALLOWED_RESULTS or is zero
pub fn validate_max_results(max_results: Option<usize>) -> Result<()> {
    validate_numeric_limit(max_results, MAX_ALLOWED_RESULTS, "max_results")
}

/// Validates max_file_size parameter against the allowed maximum (100MB)
///
/// # Arguments
/// * `max_file_size` - Optional maximum file size in bytes
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err` if exceeds MAX_ALLOWED_FILE_SIZE or is zero
pub fn validate_max_file_size(max_file_size: Option<usize>) -> Result<()> {
    validate_numeric_limit(max_file_size, MAX_ALLOWED_FILE_SIZE, "max_file_size")
}

/// Validates context_lines parameter for grep operations
///
/// # Arguments
/// * `context_lines` - Optional number of context lines
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err` if exceeds MAX_ALLOWED_CONTEXT or is negative
pub fn validate_context_lines(context_lines: Option<usize>) -> Result<()> {
    validate_numeric_limit(context_lines, MAX_ALLOWED_CONTEXT, "context_lines")
}

/// Validates that a string contains only alphanumeric characters, hyphens, and underscores
///
/// # Arguments
/// * `value` - The optional string to validate
/// * `field_name` - Name of the field for error messages
///
/// # Returns
/// * `Ok(())` if the string is valid or None
/// * `Err` if the string contains invalid characters
///
/// # Examples
/// ```
/// use vtcode_core::tools::registry::validation::validate_alphanumeric_pattern;
///
/// // Valid patterns
/// assert!(validate_alphanumeric_pattern(Some("rust"), "type_pattern").is_ok());
/// assert!(validate_alphanumeric_pattern(Some("my-type_123"), "type_pattern").is_ok());
///
/// // Invalid patterns
/// assert!(validate_alphanumeric_pattern(Some("type/../etc"), "type_pattern").is_err());
/// assert!(validate_alphanumeric_pattern(Some("type!@#"), "type_pattern").is_err());
/// ```
pub fn validate_alphanumeric_pattern(value: Option<&str>, field_name: &str) -> Result<()> {
    if let Some(pattern) = value {
        if !pattern
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow!(
                "{} can only contain alphanumeric characters, hyphens, and underscores",
                field_name
            ));
        }
    }
    Ok(())
}

/// Validates a glob pattern for security (relative paths only, no traversal)
///
/// # Arguments
/// * `glob_pattern` - The optional glob pattern to validate
///
/// # Returns
/// * `Ok(())` if the pattern is valid or None
/// * `Err` if the pattern contains security issues
pub fn validate_glob_pattern(glob_pattern: Option<&str>) -> Result<()> {
    if let Some(pattern) = glob_pattern {
        validate_relative_path(pattern, "glob_pattern")?;
    }
    Ok(())
}

/// Combined validation for grep-like operations
///
/// Validates all common parameters used in grep operations in one call
///
/// # Arguments
/// * `path` - The path to search
/// * `glob_pattern` - Optional glob pattern
/// * `type_pattern` - Optional type pattern
/// * `max_results` - Optional maximum results
/// * `max_file_size` - Optional maximum file size
/// * `context_lines` - Optional context lines
///
/// # Returns
/// * `Ok(())` if all parameters are valid
/// * `Err` on first validation failure
pub fn validate_grep_params(
    path: &str,
    glob_pattern: Option<&str>,
    type_pattern: Option<&str>,
    max_results: Option<usize>,
    max_file_size: Option<usize>,
    context_lines: Option<usize>,
) -> Result<()> {
    validate_relative_path(path, "path")?;
    validate_glob_pattern(glob_pattern)?;
    validate_alphanumeric_pattern(type_pattern, "type_pattern")?;
    validate_max_results(max_results)?;
    validate_max_file_size(max_file_size)?;
    validate_context_lines(context_lines)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_relative_path() {
        // Valid paths
        assert!(validate_relative_path("src/main.rs", "path").is_ok());
        assert!(validate_relative_path("config.toml", "path").is_ok());
        assert!(validate_relative_path("./src/lib.rs", "path").is_ok());

        // Invalid paths
        assert!(validate_relative_path("../etc/passwd", "path").is_err());
        assert!(validate_relative_path("/etc/passwd", "path").is_err());
        assert!(validate_relative_path("src/../etc/passwd", "path").is_err());
    }

    #[test]
    fn test_validate_numeric_limit() {
        // Valid values
        assert!(validate_numeric_limit(Some(500), 1000, "test").is_ok());
        assert!(validate_numeric_limit(Some(1), 1000, "test").is_ok());
        assert!(validate_numeric_limit(None, 1000, "test").is_ok());

        // Invalid values
        assert!(validate_numeric_limit(Some(1001), 1000, "test").is_err());
        assert!(validate_numeric_limit(Some(0), 1000, "test").is_err());
    }

    #[test]
    fn test_validate_alphanumeric_pattern() {
        // Valid patterns
        assert!(validate_alphanumeric_pattern(Some("rust"), "type").is_ok());
        assert!(validate_alphanumeric_pattern(Some("my-type_123"), "type").is_ok());
        assert!(validate_alphanumeric_pattern(None, "type").is_ok());

        // Invalid patterns
        assert!(validate_alphanumeric_pattern(Some("type/name"), "type").is_err());
        assert!(validate_alphanumeric_pattern(Some("type!@#"), "type").is_err());
    }

    #[test]
    fn test_validate_max_results() {
        assert!(validate_max_results(Some(100)).is_ok());
        assert!(validate_max_results(Some(1000)).is_ok());
        assert!(validate_max_results(None).is_ok());
        assert!(validate_max_results(Some(1001)).is_err());
        assert!(validate_max_results(Some(0)).is_err());
    }

    #[test]
    fn test_validate_context_lines() {
        assert!(validate_context_lines(Some(5)).is_ok());
        assert!(validate_context_lines(Some(20)).is_ok());
        assert!(validate_context_lines(None).is_ok());
        assert!(validate_context_lines(Some(21)).is_err());
        assert!(validate_context_lines(Some(0)).is_err());
    }

    #[test]
    fn test_validate_grep_params() {
        // Valid combination
        assert!(validate_grep_params(
            "src/main.rs",
            Some("*.rs"),
            Some("rust"),
            Some(100),
            Some(1024),
            Some(5)
        )
        .is_ok());

        // Invalid path
        assert!(validate_grep_params(
            "../etc/passwd",
            None,
            None,
            None,
            None,
            None
        )
        .is_err());

        // Invalid glob
        assert!(validate_grep_params(
            "src",
            Some("../../*"),
            None,
            None,
            None,
            None
        )
        .is_err());
    }
}
