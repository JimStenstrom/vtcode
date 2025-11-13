//! Core types for the tool system

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A request to execute a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    /// The name of the tool to execute
    pub tool_name: String,
    /// The input parameters for the tool
    pub parameters: Value,
    /// Optional metadata about the request
    #[serde(default)]
    pub metadata: ToolMetadata,
}

impl ToolRequest {
    /// Create a new tool request
    pub fn new(tool_name: impl Into<String>, parameters: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            parameters,
            metadata: ToolMetadata::default(),
        }
    }

    /// Create a new tool request with metadata
    pub fn with_metadata(
        tool_name: impl Into<String>,
        parameters: Value,
        metadata: ToolMetadata,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            parameters,
            metadata,
        }
    }
}

/// The response from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    /// The name of the tool that was executed
    pub tool_name: String,
    /// Whether the execution was successful
    pub success: bool,
    /// The output data from the tool
    pub output: Value,
    /// Optional error information if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Optional metadata about the response
    #[serde(default)]
    pub metadata: ToolMetadata,
}

impl ToolResponse {
    /// Create a successful response
    pub fn success(tool_name: impl Into<String>, output: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: true,
            output,
            error: None,
            metadata: ToolMetadata::default(),
        }
    }

    /// Create an error response
    pub fn error(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: false,
            output: Value::Null,
            error: Some(error.into()),
            metadata: ToolMetadata::default(),
        }
    }

    /// Create a response with metadata
    pub fn with_metadata(
        tool_name: impl Into<String>,
        success: bool,
        output: Value,
        error: Option<String>,
        metadata: ToolMetadata,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            success,
            output,
            error,
            metadata,
        }
    }
}

/// Metadata associated with tool requests and responses
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Arbitrary key-value pairs for extensibility
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ToolMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self {
            extra: HashMap::new(),
        }
    }

    /// Create metadata from a hashmap
    pub fn from_map(extra: HashMap<String, Value>) -> Self {
        Self { extra }
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.extra.insert(key.into(), value);
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
}

/// Parameter definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type (e.g., "string", "number", "boolean", "object", "array")
    #[serde(rename = "type")]
    pub param_type: String,
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// JSON schema for validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
}

impl ToolParameter {
    /// Create a new required parameter
    pub fn required(
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: param_type.into(),
            required: true,
            default: None,
            schema: None,
        }
    }

    /// Create a new optional parameter
    pub fn optional(
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: param_type.into(),
            required: false,
            default: None,
            schema: None,
        }
    }

    /// Set a default value for this parameter
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Set a JSON schema for this parameter
    pub fn with_schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }
}
