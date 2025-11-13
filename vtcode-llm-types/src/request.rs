use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::message::Message;

/// Reasoning effort level for models that support configurable reasoning intensity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffortLevel {
    Low,
    Medium,
    High,
}

impl ReasoningEffortLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Universal LLM request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,

    /// Tool choice configuration based on official API docs
    /// Supports: "auto" (default), "none", "any", or specific tool selection
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel tool calls (OpenAI specific)
    pub parallel_tool_calls: Option<bool>,

    /// Parallel tool use configuration following Anthropic best practices
    pub parallel_tool_config: Option<ParallelToolConfig>,

    /// Reasoning effort level for models that support it (low, medium, high)
    /// Applies to: Claude, GPT-5, Gemini, Qwen3, DeepSeek with reasoning capability
    pub reasoning_effort: Option<ReasoningEffortLevel>,
}

/// Tool choice configuration that works across different providers
/// Based on OpenAI, Anthropic, and Gemini API specifications
/// Follows Anthropic's tool use best practices for optimal performance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Let the model decide whether to call tools ("auto")
    /// Default behavior - allows model to use tools when appropriate
    Auto,

    /// Force the model to not call any tools ("none")
    /// Useful for pure conversational responses without tool usage
    None,

    /// Force the model to call at least one tool ("any")
    /// Ensures tool usage even when model might prefer direct response
    Any,

    /// Force the model to call a specific tool
    /// Useful for directing model to use particular functionality
    Specific(SpecificToolChoice),
}

/// Specific tool choice for forcing a particular function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String, // "function"

    pub function: SpecificFunctionChoice,
}

/// Specific function choice details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificFunctionChoice {
    pub name: String,
}

impl ToolChoice {
    /// Create auto tool choice (default behavior)
    pub fn auto() -> Self {
        Self::Auto
    }

    /// Create none tool choice (disable tool calling)
    pub fn none() -> Self {
        Self::None
    }

    /// Create any tool choice (force at least one tool call)
    pub fn any() -> Self {
        Self::Any
    }

    /// Create specific function tool choice
    pub fn function(name: String) -> Self {
        Self::Specific(SpecificToolChoice {
            tool_type: "function".to_string(),
            function: SpecificFunctionChoice { name },
        })
    }

    /// Check if this tool choice allows parallel tool use
    /// Based on Anthropic's parallel tool use guidelines
    pub fn allows_parallel_tools(&self) -> bool {
        match self {
            // Auto allows parallel tools by default
            Self::Auto => true,
            // Any forces at least one tool, may allow parallel
            Self::Any => true,
            // Specific forces one particular tool, typically no parallel
            Self::Specific(_) => false,
            // None disables tools entirely
            Self::None => false,
        }
    }

    /// Get human-readable description of tool choice behavior
    pub fn description(&self) -> &'static str {
        match self {
            Self::Auto => "Model decides when to use tools (allows parallel)",
            Self::None => "No tools will be used",
            Self::Any => "At least one tool must be used (allows parallel)",
            Self::Specific(_) => "Specific tool must be used (no parallel)",
        }
    }

    /// Convert to provider-specific format
    pub fn to_provider_format(&self, provider: &str) -> Value {
        match (self, provider) {
            (Self::Auto, "openai") | (Self::Auto, "deepseek") => json!("auto"),
            (Self::None, "openai") | (Self::None, "deepseek") => json!("none"),
            (Self::Any, "openai") | (Self::Any, "deepseek") => json!("required"),
            (Self::Specific(choice), "openai") | (Self::Specific(choice), "deepseek") => {
                json!(choice)
            }

            (Self::Auto, "anthropic") => json!({"type": "auto"}),
            (Self::None, "anthropic") => json!({"type": "none"}),
            (Self::Any, "anthropic") => json!({"type": "any"}),
            (Self::Specific(choice), "anthropic") => {
                json!({"type": "tool", "name": choice.function.name})
            }

            (Self::Auto, "gemini") => json!({"mode": "auto"}),
            (Self::None, "gemini") => json!({"mode": "none"}),
            (Self::Any, "gemini") => json!({"mode": "any"}),
            (Self::Specific(choice), "gemini") => {
                json!({"mode": "any", "allowed_function_names": [choice.function.name]})
            }

            // Generic follows OpenAI format
            _ => match self {
                Self::Auto => json!("auto"),
                Self::None => json!("none"),
                Self::Any => json!("required"),
                Self::Specific(choice) => json!(choice),
            },
        }
    }
}

/// Configuration for parallel tool use behavior
/// Based on Anthropic's parallel tool use guidelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelToolConfig {
    /// Whether to disable parallel tool use
    /// When true, forces sequential tool execution
    pub disable_parallel_tool_use: bool,

    /// Maximum number of tools to execute in parallel
    /// None means no limit (provider default)
    pub max_parallel_tools: Option<usize>,

    /// Whether to encourage parallel tool use in prompts
    pub encourage_parallel: bool,
}

impl Default for ParallelToolConfig {
    fn default() -> Self {
        Self {
            disable_parallel_tool_use: false,
            max_parallel_tools: Some(5), // Reasonable default
            encourage_parallel: true,
        }
    }
}

impl ParallelToolConfig {
    /// Create configuration optimized for Anthropic models
    pub fn anthropic_optimized() -> Self {
        Self {
            disable_parallel_tool_use: false,
            max_parallel_tools: None, // Let Anthropic decide
            encourage_parallel: true,
        }
    }

    /// Create configuration for sequential tool use
    pub fn sequential_only() -> Self {
        Self {
            disable_parallel_tool_use: true,
            max_parallel_tools: Some(1),
            encourage_parallel: false,
        }
    }
}

/// Universal tool definition that matches OpenAI/Anthropic/Gemini specifications
/// Based on official API documentation from Context7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The type of tool (always "function" for function calling)
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function definition containing name, description, and parameters
    pub function: FunctionDefinition,
}

/// Function definition within a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function to be called
    pub name: String,

    /// A description of what the function does
    pub description: String,

    /// The parameters the function accepts, described as a JSON Schema object
    pub parameters: Value,
}

fn sanitize_tool_description(description: &str) -> String {
    let normalized = description
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    normalized.trim().to_string()
}

impl ToolDefinition {
    /// Create a new tool definition with function type
    pub fn function(name: String, description: String, parameters: Value) -> Self {
        let sanitized_description = sanitize_tool_description(&description);
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name,
                description: sanitized_description,
                parameters,
            },
        }
    }

    /// Get the function name for easy access
    pub fn function_name(&self) -> &str {
        &self.function.name
    }

    /// Validate that this tool definition is properly formed
    pub fn validate(&self) -> Result<(), String> {
        if self.tool_type != "function" {
            return Err(format!(
                "Only 'function' type is supported, got: {}",
                self.tool_type
            ));
        }

        if self.function.name.is_empty() {
            return Err("Function name cannot be empty".to_string());
        }

        if self.function.description.is_empty() {
            return Err("Function description cannot be empty".to_string());
        }

        // Validate that parameters is a proper JSON Schema object
        if !self.function.parameters.is_object() {
            return Err("Function parameters must be a JSON object".to_string());
        }

        Ok(())
    }
}

/// Universal tool call that matches the exact structure from OpenAI API
/// Based on OpenAI Cookbook examples and official documentation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call (e.g., "call_123")
    pub id: String,

    /// The type of tool call (always "function" for function calling)
    #[serde(rename = "type")]
    pub call_type: String,

    /// Function call details
    pub function: FunctionCall,
}

/// Function call within a tool call
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call
    pub name: String,

    /// The arguments to pass to the function, as a JSON string
    pub arguments: String,
}

impl ToolCall {
    /// Create a new function tool call
    pub fn function(id: String, name: String, arguments: String) -> Self {
        Self {
            id,
            call_type: "function".to_string(),
            function: FunctionCall { name, arguments },
        }
    }

    /// Parse the arguments as JSON Value
    pub fn parsed_arguments(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }

    /// Validate that this tool call is properly formed
    pub fn validate(&self) -> Result<(), String> {
        if self.call_type != "function" {
            return Err(format!(
                "Only 'function' type is supported, got: {}",
                self.call_type
            ));
        }

        if self.id.is_empty() {
            return Err("Tool call ID cannot be empty".to_string());
        }

        if self.function.name.is_empty() {
            return Err("Function name cannot be empty".to_string());
        }

        // Validate that arguments is valid JSON
        if let Err(e) = self.parsed_arguments() {
            return Err(format!("Invalid JSON in function arguments: {}", e));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sanitize_tool_description_trims_padding() {
        let input = "\n\nLine 1\nLine 2 \n";
        assert_eq!(sanitize_tool_description(input), "Line 1\nLine 2");
    }

    #[test]
    fn sanitize_tool_description_preserves_internal_blank_lines() {
        let input = "Line 1\n\nLine 3";
        assert_eq!(sanitize_tool_description(input), input);
    }

    #[test]
    fn tool_definition_function_uses_sanitized_description() {
        let tool = ToolDefinition::function(
            "demo".to_string(),
            "  Line 1  \n".to_string(),
            json!({"type": "object", "properties": {}}),
        );
        assert_eq!(tool.function.description, "Line 1");
    }
}
