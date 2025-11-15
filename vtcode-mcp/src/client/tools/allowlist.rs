//! Tool allowlist enforcement and validation.

use anyhow::{Result, anyhow};
use serde_json::Value;

/// Validate tool arguments based on security configuration.
///
/// This function performs multiple security checks on tool arguments:
/// 1. **Size validation**: Ensures arguments don't exceed the configured maximum size
/// 2. **Path traversal protection**: Detects and prevents directory traversal attempts
///
/// # Arguments
/// * `_tool_name` - The name of the tool (reserved for future use)
/// * `args` - The JSON arguments to validate
/// * `max_argument_size` - Maximum allowed size in bytes (0 = unlimited)
/// * `path_traversal_protection` - Whether to check for path traversal
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err` if validation fails with a descriptive error message
///
/// # Security Notes
/// - Maximum argument size prevents memory exhaustion attacks
/// - Path traversal detection prevents accessing files outside allowed directories
pub fn validate_tool_arguments(
    _tool_name: &str,
    args: &Value,
    max_argument_size: u32,
    path_traversal_protection: bool,
) -> Result<()> {
    // Check argument size to prevent memory exhaustion
    if max_argument_size > 0 {
        let args_size = serde_json::to_string(args).map(|s| s.len()).unwrap_or(0) as u32;

        if args_size > max_argument_size {
            return Err(anyhow!(
                "Tool arguments exceed maximum size of {} bytes",
                max_argument_size
            ));
        }
    }

    // Check for path traversal attacks in file-related arguments
    // This prevents accessing files outside the allowed workspace
    if path_traversal_protection {
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            if path.contains("../")
                || path.starts_with("../")
                || path.contains("..\\")
                || path.starts_with("..\\")
            {
                return Err(anyhow!("Path traversal detected in arguments"));
            }
        }
    }

    Ok(())
}
