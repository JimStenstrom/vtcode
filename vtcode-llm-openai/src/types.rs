//! Core types for OpenAI LLM provider

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// LLM request structure for OpenAI
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

/// Message role
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
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
    Auto,
    None,
    Required,
    Specific(SpecificToolChoice),
}

/// Specific tool choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: SpecificFunctionChoice,
}

/// Specific function choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificFunctionChoice {
    pub name: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
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
    FunctionCall,
}

/// LLM errors
#[derive(Debug, Clone, thiserror::Error)]
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
