use serde::{Deserialize, Serialize};

use crate::request::ToolCall;

/// Content type for messages that can include both text and images
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text {
        text: String,
    },
    Image {
        data: String,      // Base64 encoded image data
        mime_type: String, // MIME type (e.g., "image/png")
        #[serde(rename = "type")]
        content_type: String, // "image"
    },
}

impl ContentPart {
    pub fn text(text: String) -> Self {
        ContentPart::Text { text }
    }

    pub fn image(data: String, mime_type: String) -> Self {
        ContentPart::Image {
            data,
            mime_type,
            content_type: "image".to_string(),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentPart::Text { text } => Some(text),
            _ => None,
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self, ContentPart::Image { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Legacy single text string
    Text(String),
    /// Multiple content parts (text and images)
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn text(text: String) -> Self {
        MessageContent::Text(text)
    }

    pub fn parts(parts: Vec<ContentPart>) -> Self {
        MessageContent::Parts(parts)
    }

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

    pub fn trim(&self) -> String {
        self.as_text().trim().to_string()
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MessageContent::Text(text) => text.is_empty(),
            MessageContent::Parts(parts) => {
                parts.is_empty()
                    || parts.iter().all(|part| match part {
                        ContentPart::Text { text } => text.is_empty(),
                        _ => false,
                    })
            }
        }
    }

    pub fn has_images(&self) -> bool {
        match self {
            MessageContent::Text(_) => false,
            MessageContent::Parts(parts) => parts.iter().any(|part| part.is_image()),
        }
    }

    pub fn get_images(&self) -> Vec<&ContentPart> {
        match self {
            MessageContent::Text(_) => vec![],
            MessageContent::Parts(parts) => parts.iter().filter(|part| part.is_image()).collect(),
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

impl From<&str> for MessageContent {
    fn from(value: &str) -> Self {
        MessageContent::Text(value.to_string())
    }
}

/// Universal message structure supporting both text and image content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    /// Content can be a string (for backward compatibility) or an array of content parts
    #[serde(default)]
    pub content: MessageContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<serde_json::Value>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    /// Optional origin tool name for tracking which tool generated this message
    /// Used in tool-aware context retention to preserve results from recently-active tools
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_tool: Option<String>,
}

impl Message {
    /// Create a user message with text content
    pub fn user(content: String) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create a user message with multiple content parts (text and images)
    pub fn user_with_parts(content_parts: Vec<ContentPart>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Parts(content_parts),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(content: String) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create an assistant message with multiple content parts
    pub fn assistant_with_parts(content_parts: Vec<ContentPart>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Parts(content_parts),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create an assistant message with tool calls
    /// Based on OpenAI Cookbook patterns for function calling
    pub fn assistant_with_tools(content: String, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create an assistant message with tool calls and multiple content parts
    pub fn assistant_with_tools_and_parts(
        content_parts: Vec<ContentPart>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Parts(content_parts),
            reasoning: None,
            reasoning_details: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create an assistant message with tool calls and reasoning details
    /// Used for preserving reasoning state in multi-turn conversations
    pub fn assistant_with_tools_and_reasoning(
        content: String,
        tool_calls: Vec<ToolCall>,
        reasoning_details: Option<Vec<serde_json::Value>>,
    ) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: None,
            origin_tool: None,
        }
    }

    /// Create a tool response message
    /// This follows the exact pattern from OpenAI Cookbook:
    /// ```json
    /// {
    ///   "role": "tool",
    ///   "tool_call_id": "call_123",
    ///   "content": "Function result"
    /// }
    /// ```
    pub fn tool_response(tool_call_id: String, content: String) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            origin_tool: None,
        }
    }

    /// Create a tool response message with function name (for compatibility)
    /// Some providers might need the function name in addition to tool_call_id
    pub fn tool_response_with_name(
        tool_call_id: String,
        _function_name: String,
        content: String,
    ) -> Self {
        // We can store the function name in the content metadata or handle it provider-specifically
        Self::tool_response(tool_call_id, content)
    }

    /// Create a tool response message with origin tool tracking
    /// The origin_tool field helps with tool-aware context retention
    pub fn tool_response_with_origin(
        tool_call_id: String,
        content: String,
        origin_tool: String,
    ) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::Text(content),
            reasoning: None,
            reasoning_details: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            origin_tool: Some(origin_tool),
        }
    }

    /// Attach provider-visible reasoning trace for archival without affecting payloads.
    pub fn with_reasoning(mut self, reasoning: Option<String>) -> Self {
        self.reasoning = reasoning;
        self
    }

    /// Attach reasoning details for providers that support structured reasoning
    pub fn with_reasoning_details(
        mut self,
        reasoning_details: Option<Vec<serde_json::Value>>,
    ) -> Self {
        self.reasoning_details = reasoning_details;
        self
    }

    /// Validate this message for a specific provider
    /// Based on official API documentation constraints
    pub fn validate_for_provider(&self, provider: &str) -> Result<(), String> {
        // Check role-specific constraints
        self.role
            .validate_for_provider(provider, self.tool_call_id.is_some())?;

        // Check tool call constraints
        if let Some(tool_calls) = &self.tool_calls {
            if !self.role.can_make_tool_calls() {
                return Err(format!("Role {:?} cannot make tool calls", self.role));
            }

            if tool_calls.is_empty() {
                return Err("Tool calls array should not be empty".to_string());
            }

            // Validate each tool call
            for tool_call in tool_calls {
                tool_call.validate()?;
            }
        }

        // Provider-specific validations based on official docs
        match provider {
            "openai" | "openrouter" | "zai" => {
                if self.role == MessageRole::Tool && self.tool_call_id.is_none() {
                    return Err(format!(
                        "{} requires tool_call_id for tool messages",
                        provider
                    ));
                }
            }
            "gemini" => {
                if self.role == MessageRole::Tool && self.tool_call_id.is_none() {
                    return Err(
                        "Gemini tool responses need tool_call_id for function name mapping"
                            .to_string(),
                    );
                }
                // Gemini has additional constraints on content structure
                if self.role == MessageRole::System && !self.content.as_text().is_empty() {
                    // System messages should be handled as systemInstruction, not in contents
                }
            }
            "anthropic" => {
                // Anthropic is more flexible with tool message format
                // Tool messages are converted to user messages anyway
            }
            _ => {} // Generic validation already done above
        }

        Ok(())
    }

    /// Check if this message has tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .map_or(false, |calls| !calls.is_empty())
    }

    /// Get the tool calls if present
    pub fn get_tool_calls(&self) -> Option<&[ToolCall]> {
        self.tool_calls.as_deref()
    }

    /// Check if this is a tool response message
    pub fn is_tool_response(&self) -> bool {
        self.role == MessageRole::Tool
    }

    /// Get the text content of the message (for backward compatibility)
    pub fn get_text_content(&self) -> String {
        self.content.as_text()
    }

    /// Check if this message contains images
    pub fn has_images(&self) -> bool {
        self.content.has_images()
    }

    /// Get all images in this message
    pub fn get_images(&self) -> Vec<&ContentPart> {
        self.content.get_images()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    /// Get the role string for Gemini API
    /// Note: Gemini API has specific constraints on message roles
    /// - Only accepts "user" and "model" roles in conversations
    /// - System messages are handled separately as system instructions
    /// - Tool responses are sent as "user" role with function response format
    pub fn as_gemini_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system", // Handled as systemInstruction, not in contents
            MessageRole::User => "user",
            MessageRole::Assistant => "model", // Gemini uses "model" instead of "assistant"
            MessageRole::Tool => "user", // Tool responses are sent as user messages with functionResponse
        }
    }

    /// Get the role string for OpenAI API
    /// OpenAI supports all standard role types including:
    /// - system, user, assistant, tool
    /// - function (legacy, now replaced by tool)
    pub fn as_openai_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool", // Full support for tool role with tool_call_id
        }
    }

    /// Get the role string for Anthropic API
    /// Anthropic has specific handling for tool messages:
    /// - Supports user, assistant roles normally
    /// - Tool responses are treated as user messages
    /// - System messages can be handled as system parameter or hoisted
    pub fn as_anthropic_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system", // Can be hoisted to system parameter
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "user", // Anthropic treats tool responses as user messages
        }
    }

    /// Get the role string for generic OpenAI-compatible providers
    /// Most providers follow OpenAI's role conventions
    pub fn as_generic_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        }
    }

    /// Check if this role supports tool calls
    /// Only Assistant role can initiate tool calls in most APIs
    pub fn can_make_tool_calls(&self) -> bool {
        matches!(self, MessageRole::Assistant)
    }

    /// Check if this role represents a tool response
    pub fn is_tool_response(&self) -> bool {
        matches!(self, MessageRole::Tool)
    }

    /// Validate message role constraints for a given provider
    /// Based on official API documentation requirements
    pub fn validate_for_provider(
        &self,
        provider: &str,
        has_tool_call_id: bool,
    ) -> Result<(), String> {
        match (self, provider) {
            (MessageRole::Tool, provider)
                if matches!(
                    provider,
                    "openai" | "openrouter" | "xai" | "deepseek" | "zai"
                ) && !has_tool_call_id =>
            {
                Err(format!("{} tool messages must have tool_call_id", provider))
            }
            (MessageRole::Tool, "gemini") if !has_tool_call_id => {
                Err("Gemini tool messages need tool_call_id for function mapping".to_string())
            }
            _ => Ok(()),
        }
    }
}
