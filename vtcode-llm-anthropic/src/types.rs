//! Core types for Anthropic LLM provider

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// LLM request structure for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
    pub tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub parallel_tool_config: Option<ParallelToolConfig>,
    pub reasoning_effort: Option<String>,
}

/// LLM response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: Option<FinishReason>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub cached_prompt_tokens: Option<usize>,
    pub cache_creation_tokens: Option<usize>,
    pub cache_read_tokens: Option<usize>,
}

/// Message in the conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content),
            reasoning: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            reasoning: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant_with_tools(content: String, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            reasoning: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    pub fn tool_response(tool_call_id: String, content: String) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::Text(content),
            reasoning: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn as_anthropic_str(&self) -> &'static str {
        match self {
            MessageRole::System => "user",  // System messages get converted to user in Anthropic
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "user",  // Tool messages become user messages in Anthropic
        }
    }
}

/// Message content (text or multipart)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn as_text(&self) -> String {
        match self {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|part| part.as_text())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MessageContent::Text(text) => text.is_empty(),
            MessageContent::Parts(parts) => parts.is_empty(),
        }
    }
}

impl Default for MessageContent {
    fn default() -> Self {
        MessageContent::Text(String::new())
    }
}

impl From<String> for MessageContent {
    fn from(value: String) -> Self {
        MessageContent::Text(value)
    }
}

/// Content part (text or image)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text { text: String },
    Image { data: String, mime_type: String },
}

impl ContentPart {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentPart::Text { text } => Some(text),
            _ => None,
        }
    }
}

/// Tool choice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    String(String),  // "auto", "none", "any"
    Specific(SpecificToolChoice),
}

impl ToolChoice {
    pub fn auto() -> Self {
        ToolChoice::String("auto".to_string())
    }

    pub fn none() -> Self {
        ToolChoice::String("none".to_string())
    }

    pub fn any() -> Self {
        ToolChoice::String("any".to_string())
    }

    pub fn function(name: String) -> Self {
        ToolChoice::Specific(SpecificToolChoice {
            tool_type: "tool".to_string(),
            name,
        })
    }

    pub fn to_anthropic_format(&self) -> Value {
        match self {
            ToolChoice::String(s) => serde_json::json!({ "type": s }),
            ToolChoice::Specific(spec) => serde_json::json!({
                "type": spec.tool_type,
                "name": spec.name
            }),
        }
    }
}

/// Specific tool choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
}

/// Parallel tool configuration for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelToolConfig {
    pub max_parallel_calls: Option<usize>,
    pub allow_parallel: Option<bool>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

impl ToolDefinition {
    pub fn function(name: String, description: String, parameters: Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name,
                description: Some(description),
                parameters: Some(parameters),
            },
        }
    }
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

/// Tool call in response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    #[serde(rename = "type")]
    pub call_type: String,
}

impl ToolCall {
    pub fn function(id: String, name: String, arguments: String) -> Self {
        Self {
            id,
            function: FunctionCall { name, arguments },
            call_type: "function".to_string(),
        }
    }
}

/// Function call details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Finish reason for completion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error(String),
}

/// Prompt caching configuration for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCachingConfig {
    pub enabled: bool,
    pub max_breakpoints: u32,
    pub cache_user_messages: bool,
    pub cache_system_messages: bool,
    pub extended_ttl_seconds: Option<u64>,
}

impl Default for PromptCachingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_breakpoints: 3,
            cache_user_messages: false,
            cache_system_messages: true,
            extended_ttl_seconds: None,
        }
    }
}

/// LLM errors
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Stream error: {0}")]
    StreamError(String),
}

impl From<reqwest::Error> for LLMError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LLMError::Network(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            LLMError::Network(format!("Connection failed: {}", err))
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            LLMError::Authentication("Invalid API key".to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            LLMError::RateLimit
        } else {
            LLMError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for LLMError {
    fn from(err: serde_json::Error) -> Self {
        LLMError::Provider(format!("JSON parsing error: {}", err))
    }
}
