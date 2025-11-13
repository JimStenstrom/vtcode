//! Error types for tool execution

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Result type for tool operations
pub type ToolResult<T> = anyhow::Result<T>;

/// Classification of tool execution errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolErrorType {
    /// Invalid or malformed parameters
    InvalidParameters,
    /// Tool not found in registry
    ToolNotFound,
    /// Permission denied for operation
    PermissionDenied,
    /// Resource (file, network, etc.) not found
    ResourceNotFound,
    /// Network-related error
    NetworkError,
    /// Operation timed out
    Timeout,
    /// General execution error
    ExecutionError,
    /// Policy violation (security, workspace boundaries, etc.)
    PolicyViolation,
}

/// Structured error information for tool execution failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionError {
    /// Name of the tool that failed
    pub tool_name: String,
    /// Classification of the error
    pub error_type: ToolErrorType,
    /// Human-readable error message
    pub message: String,
    /// Whether the error can be recovered from
    pub is_recoverable: bool,
    /// Suggestions for recovering from the error
    pub recovery_suggestions: Vec<String>,
    /// Original error message if available
    pub original_error: Option<String>,
}

impl ToolExecutionError {
    /// Create a new tool execution error
    pub fn new(tool_name: String, error_type: ToolErrorType, message: String) -> Self {
        let (is_recoverable, recovery_suggestions) = generate_recovery_info(&error_type);

        Self {
            tool_name,
            error_type,
            message,
            is_recoverable,
            recovery_suggestions,
            original_error: None,
        }
    }

    /// Create a new tool execution error with original error information
    pub fn with_original_error(
        tool_name: String,
        error_type: ToolErrorType,
        message: String,
        original_error: String,
    ) -> Self {
        let mut error = Self::new(tool_name, error_type, message);
        error.original_error = Some(original_error);
        error
    }

    /// Convert error to JSON value for serialization
    pub fn to_json_value(&self) -> Value {
        json!({
            "error": {
                "tool_name": self.tool_name,
                "error_type": format!("{:?}", self.error_type),
                "message": self.message,
                "is_recoverable": self.is_recoverable,
                "recovery_suggestions": self.recovery_suggestions,
                "original_error": self.original_error,
            }
        })
    }
}

/// Classify an anyhow error into a tool error type
pub fn classify_error(error: &anyhow::Error) -> ToolErrorType {
    let error_msg = error.to_string().to_lowercase();

    if error_msg.contains("permission") || error_msg.contains("access denied") {
        ToolErrorType::PermissionDenied
    } else if error_msg.contains("not found") || error_msg.contains("no such file") {
        ToolErrorType::ResourceNotFound
    } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
        ToolErrorType::Timeout
    } else if error_msg.contains("network") || error_msg.contains("connection") {
        ToolErrorType::NetworkError
    } else if error_msg.contains("invalid") || error_msg.contains("malformed") {
        ToolErrorType::InvalidParameters
    } else if error_msg.contains("policy") || error_msg.contains("denied") {
        ToolErrorType::PolicyViolation
    } else {
        ToolErrorType::ExecutionError
    }
}

fn generate_recovery_info(error_type: &ToolErrorType) -> (bool, Vec<String>) {
    match error_type {
        ToolErrorType::InvalidParameters => (
            true,
            vec![
                "Check parameter names and types against the tool schema".to_string(),
                "Ensure required parameters are provided".to_string(),
                "Verify parameter values are within acceptable ranges".to_string(),
            ],
        ),
        ToolErrorType::ToolNotFound => (
            false,
            vec![
                "Verify the tool name is spelled correctly".to_string(),
                "Check if the tool is available in the current context".to_string(),
                "Contact administrator if tool should be available".to_string(),
            ],
        ),
        ToolErrorType::PermissionDenied => (
            true,
            vec![
                "Check file permissions and access rights".to_string(),
                "Ensure workspace boundaries are respected".to_string(),
                "Try running with appropriate permissions".to_string(),
            ],
        ),
        ToolErrorType::ResourceNotFound => (
            true,
            vec![
                "Verify file paths and resource locations".to_string(),
                "Check if files exist and are accessible".to_string(),
                "Use list_dir to explore available resources".to_string(),
            ],
        ),
        ToolErrorType::NetworkError => (
            true,
            vec![
                "Check network connectivity".to_string(),
                "Retry the operation after a brief delay".to_string(),
                "Verify external service availability".to_string(),
            ],
        ),
        ToolErrorType::Timeout => (
            true,
            vec![
                "Increase timeout values if appropriate".to_string(),
                "Break large operations into smaller chunks".to_string(),
                "Check system resources and performance".to_string(),
            ],
        ),
        ToolErrorType::ExecutionError => (
            false,
            vec![
                "Review error details for specific issues".to_string(),
                "Check tool documentation for known limitations".to_string(),
                "Report the issue if it appears to be a bug".to_string(),
            ],
        ),
        ToolErrorType::PolicyViolation => (
            false,
            vec![
                "Review workspace policies and restrictions".to_string(),
                "Contact administrator for policy changes".to_string(),
                "Use alternative tools that comply with policies".to_string(),
            ],
        ),
    }
}
