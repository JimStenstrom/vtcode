//! Test fixtures for validation testing
//!
//! This module provides common test data for requests, responses, and error scenarios

use serde_json::json;
use vtcode_core::llm::provider::{
    FinishReason, LLMError, LLMRequest, LLMResponse, Message, MessageContent, ToolCall,
    ToolChoice, ToolDefinition, Usage,
};

// ============================================================================
// Request Fixtures
// ============================================================================

/// Create a simple request with just a user message
pub fn simple_request() -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("Hello, how are you?".to_string())],
        system_prompt: None,
        tools: None,
        model: "test-model".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Create a request with system prompt
pub fn request_with_system_prompt() -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("What is 2+2?".to_string())],
        system_prompt: Some("You are a helpful math tutor.".to_string()),
        tools: None,
        model: "test-model".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Create a request with multiple messages
pub fn multi_message_request() -> LLMRequest {
    LLMRequest {
        messages: vec![
            Message::user("What is the capital of France?".to_string()),
            Message::assistant("The capital of France is Paris.".to_string()),
            Message::user("What about Germany?".to_string()),
        ],
        system_prompt: None,
        tools: None,
        model: "test-model".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Create a request with tool definitions
pub fn tool_request() -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("Search for information about Rust".to_string())],
        system_prompt: None,
        tools: Some(vec![
            ToolDefinition::function(
                "search".to_string(),
                "Search for information on the web".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        }
                    },
                    "required": ["query"]
                }),
            ),
            ToolDefinition::function(
                "calculator".to_string(),
                "Perform mathematical calculations".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "The mathematical expression to evaluate"
                        }
                    },
                    "required": ["expression"]
                }),
            ),
        ]),
        model: "test-model".to_string(),
        max_tokens: None,
        temperature: Some(0.7),
        stream: false,
        tool_choice: Some(ToolChoice::Auto),
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Create a streaming request
pub fn streaming_request() -> LLMRequest {
    let mut req = simple_request();
    req.stream = true;
    req
}

/// Create a request with custom temperature and max_tokens
pub fn request_with_params() -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("Write a short poem".to_string())],
        system_prompt: None,
        tools: None,
        model: "test-model".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.9),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

// ============================================================================
// Response Fixtures
// ============================================================================

/// Create a simple successful response
pub fn simple_response() -> LLMResponse {
    LLMResponse {
        content: "I'm doing well, thank you for asking! How can I help you today?".to_string(),
        model: "test-model".to_string(),
        stop_reason: FinishReason::EndTurn,
        tool_calls: None,
        usage: Usage {
            input_tokens: 15,
            output_tokens: 25,
            total_tokens: 40,
        },
        reasoning: None,
    }
}

/// Create a response with tool calls
pub fn tool_response() -> LLMResponse {
    LLMResponse {
        content: String::new(),
        model: "test-model".to_string(),
        stop_reason: FinishReason::ToolUse,
        tool_calls: Some(vec![ToolCall {
            id: "call_123".to_string(),
            name: "search".to_string(),
            input: json!({
                "query": "Rust programming language"
            }),
        }]),
        usage: Usage {
            input_tokens: 50,
            output_tokens: 30,
            total_tokens: 80,
        },
        reasoning: None,
    }
}

/// Create a response with reasoning
pub fn reasoning_response() -> LLMResponse {
    LLMResponse {
        content: "The answer is 4.".to_string(),
        model: "test-model".to_string(),
        stop_reason: FinishReason::EndTurn,
        tool_calls: None,
        usage: Usage {
            input_tokens: 20,
            output_tokens: 50,
            total_tokens: 70,
        },
        reasoning: Some("Let me think about this step by step: 2 + 2 = 4".to_string()),
    }
}

/// Create a response that was stopped due to length
pub fn max_tokens_response() -> LLMResponse {
    LLMResponse {
        content: "This is a response that was cut off because it reached the maximum".to_string(),
        model: "test-model".to_string(),
        stop_reason: FinishReason::MaxTokens,
        tool_calls: None,
        usage: Usage {
            input_tokens: 10,
            output_tokens: 100,
            total_tokens: 110,
        },
        reasoning: None,
    }
}

// ============================================================================
// Error Fixtures
// ============================================================================

/// Create a rate limit error
pub fn rate_limit_error() -> LLMError {
    LLMError::RateLimit("Rate limit exceeded. Please try again later.".to_string())
}

/// Create an invalid request error
pub fn invalid_request_error() -> LLMError {
    LLMError::InvalidRequest("Invalid model specified".to_string())
}

/// Create an authentication error
pub fn authentication_error() -> LLMError {
    LLMError::Authentication("Invalid API key".to_string())
}

/// Create a network error
pub fn network_error() -> LLMError {
    LLMError::Network("Connection timeout".to_string())
}

/// Create an internal error
pub fn internal_error() -> LLMError {
    LLMError::Internal("Internal server error".to_string())
}

// ============================================================================
// Provider-Specific Response JSON Fixtures
// ============================================================================

/// Anthropic error response JSON
pub fn anthropic_rate_limit_json() -> String {
    json!({
        "type": "error",
        "error": {
            "type": "rate_limit_error",
            "message": "Number of requests exceeds rate limit"
        }
    })
    .to_string()
}

/// Anthropic invalid request JSON
pub fn anthropic_invalid_request_json() -> String {
    json!({
        "type": "error",
        "error": {
            "type": "invalid_request_error",
            "message": "Invalid model: unknown-model"
        }
    })
    .to_string()
}

/// Gemini RESOURCE_EXHAUSTED error JSON
pub fn gemini_resource_exhausted_json() -> String {
    json!({
        "error": {
            "code": 429,
            "message": "RESOURCE_EXHAUSTED: Quota exceeded",
            "status": "RESOURCE_EXHAUSTED"
        }
    })
    .to_string()
}

/// Gemini rate limit error JSON
pub fn gemini_rate_limit_json() -> String {
    json!({
        "error": {
            "code": 429,
            "message": "rateLimitExceeded",
            "status": "RATE_LIMIT_EXCEEDED"
        }
    })
    .to_string()
}

/// OpenAI rate limit error JSON
pub fn openai_rate_limit_json() -> String {
    json!({
        "error": {
            "message": "Rate limit exceeded",
            "type": "rate_limit_error",
            "param": null,
            "code": "rate_limit_exceeded"
        }
    })
    .to_string()
}

/// OpenAI invalid model error JSON
pub fn openai_invalid_model_json() -> String {
    json!({
        "error": {
            "message": "The model `unknown-model` does not exist",
            "type": "invalid_request_error",
            "param": null,
            "code": "model_not_found"
        }
    })
    .to_string()
}

/// OpenAI successful response JSON
pub fn openai_success_json() -> String {
    json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    })
    .to_string()
}

/// Anthropic successful response JSON
pub fn anthropic_success_json() -> String {
    json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "Hello! How can I help you today?"
        }],
        "model": "claude-sonnet-4-5",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    })
    .to_string()
}

// ============================================================================
// Test Data Builders
// ============================================================================

/// Builder for creating custom LLMRequest instances
pub struct RequestBuilder {
    request: LLMRequest,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            request: simple_request(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.request.model = model.into();
        self
    }

    pub fn with_message(mut self, message: Message) -> Self {
        self.request.messages = vec![message];
        self
    }

    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.request.messages = messages;
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.request.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.request.tools = Some(tools);
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.request.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.request.max_tokens = Some(tokens);
        self
    }

    pub fn streaming(mut self) -> Self {
        self.request.stream = true;
        self
    }

    pub fn build(self) -> LLMRequest {
        self.request
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_request_fixture() {
        let req = simple_request();
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].content.as_text(), "Hello, how are you?");
        assert_eq!(req.stream, false);
    }

    #[test]
    fn test_tool_request_fixture() {
        let req = tool_request();
        assert!(req.tools.is_some());
        let tools = req.tools.unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].function_name(), "search");
        assert_eq!(tools[1].function_name(), "calculator");
    }

    #[test]
    fn test_request_builder() {
        let req = RequestBuilder::new()
            .with_model("gpt-5")
            .with_temperature(0.8)
            .with_max_tokens(500)
            .streaming()
            .build();

        assert_eq!(req.model, "gpt-5");
        assert_eq!(req.temperature, Some(0.8));
        assert_eq!(req.max_tokens, Some(500));
        assert_eq!(req.stream, true);
    }

    #[test]
    fn test_response_fixtures() {
        let resp = simple_response();
        assert!(!resp.content.is_empty());
        assert_eq!(resp.stop_reason, FinishReason::EndTurn);

        let tool_resp = tool_response();
        assert!(tool_resp.tool_calls.is_some());
        assert_eq!(tool_resp.stop_reason, FinishReason::ToolUse);
    }
}
