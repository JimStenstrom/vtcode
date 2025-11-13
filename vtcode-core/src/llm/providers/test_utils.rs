// Test utilities and fixtures for LLM provider testing
#![cfg(test)]

use crate::config::constants::models;
use crate::llm::provider::{
    FinishReason, LLMError, LLMRequest, LLMResponse, Message, MessageRole, ToolCall,
    ToolChoice, ToolDefinition,
};
use serde_json::{Value, json};

/// Creates a simple request with user message
pub fn simple_request(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("Hello, world!".to_string())],
        system_prompt: None,
        tools: None,
        model: model.to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a request with system prompt
pub fn request_with_system_prompt(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("What's the weather?".to_string())],
        system_prompt: Some("You are a helpful weather assistant.".to_string()),
        tools: None,
        model: model.to_string(),
        max_tokens: Some(200),
        temperature: Some(0.5),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a request with multiple messages including assistant messages
pub fn multi_message_request(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![
            Message::user("Hello!".to_string()),
            Message::assistant("Hi! How can I help you today?".to_string()),
            Message::user("Tell me a joke.".to_string()),
        ],
        system_prompt: Some("You are a friendly assistant.".to_string()),
        tools: None,
        model: model.to_string(),
        max_tokens: Some(150),
        temperature: Some(0.8),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a tool definition for weather lookup
pub fn weather_tool() -> ToolDefinition {
    ToolDefinition::function(
        "get_weather".to_string(),
        "Get the current weather for a location".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city name"
                },
                "units": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature units"
                }
            },
            "required": ["location"]
        }),
    )
}

/// Creates a tool definition for calculations
pub fn calculator_tool() -> ToolDefinition {
    ToolDefinition::function(
        "calculate".to_string(),
        "Perform a mathematical calculation".to_string(),
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
    )
}

/// Creates a tool definition with complex parameters
pub fn complex_tool() -> ToolDefinition {
    ToolDefinition::function(
        "search_database".to_string(),
        "Search a database with filters".to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "filters": {
                    "type": "object",
                    "properties": {
                        "category": {"type": "string"},
                        "min_price": {"type": "number"},
                        "max_price": {"type": "number"},
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    }
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query"]
        }),
    )
}

/// Creates a request with tools
pub fn request_with_tools(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("What's the weather in Paris?".to_string())],
        system_prompt: Some("You are a helpful assistant with access to tools.".to_string()),
        tools: Some(vec![weather_tool(), calculator_tool()]),
        model: model.to_string(),
        max_tokens: Some(500),
        temperature: Some(0.3),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a request with tool choice
pub fn request_with_tool_choice(model: &str) -> LLMRequest {
    let mut request = request_with_tools(model);
    request.tool_choice = Some(ToolChoice::Required);
    request
}

/// Creates a request with auto tool choice
pub fn request_with_auto_tool_choice(model: &str) -> LLMRequest {
    let mut request = request_with_tools(model);
    request.tool_choice = Some(ToolChoice::Auto);
    request
}

/// Creates a request with specific tool choice
pub fn request_with_specific_tool_choice(model: &str, tool_name: &str) -> LLMRequest {
    let mut request = request_with_tools(model);
    request.tool_choice = Some(ToolChoice::Specific(tool_name.to_string()));
    request
}

/// Creates a request with special characters in messages
pub fn request_with_special_chars(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![
            Message::user(
                r#"Test special chars: "quotes", 'apostrophes', \backslash, <tags>, & ampersand, emoji 🎉"#
                    .to_string(),
            ),
        ],
        system_prompt: Some("Handle special characters correctly.".to_string()),
        tools: None,
        model: model.to_string(),
        max_tokens: Some(100),
        temperature: Some(0.5),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a request with empty user message (edge case)
pub fn request_with_empty_message(model: &str) -> LLMRequest {
    LLMRequest {
        messages: vec![Message::user("".to_string())],
        system_prompt: None,
        tools: None,
        model: model.to_string(),
        max_tokens: Some(10),
        temperature: Some(0.5),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a request with very long message
pub fn request_with_long_message(model: &str) -> LLMRequest {
    let long_message = "word ".repeat(1000);
    LLMRequest {
        messages: vec![Message::user(long_message)],
        system_prompt: None,
        tools: None,
        model: model.to_string(),
        max_tokens: Some(100),
        temperature: Some(0.5),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Creates a sample tool call for testing
pub fn sample_tool_call() -> ToolCall {
    ToolCall {
        id: "call_123".to_string(),
        name: "get_weather".to_string(),
        parameters: json!({
            "location": "Paris",
            "units": "celsius"
        }),
    }
}

/// Creates a sample tool call with complex parameters
pub fn complex_tool_call() -> ToolCall {
    ToolCall {
        id: "call_456".to_string(),
        name: "search_database".to_string(),
        parameters: json!({
            "query": "laptops",
            "filters": {
                "category": "electronics",
                "min_price": 500,
                "max_price": 2000,
                "tags": ["gaming", "portable"]
            },
            "limit": 10
        }),
    }
}

/// Creates a request with message containing tool call results
pub fn request_with_tool_result(model: &str) -> LLMRequest {
    let tool_call = sample_tool_call();
    LLMRequest {
        messages: vec![
            Message::user("What's the weather in Paris?".to_string()),
            Message::assistant_with_tool_calls(vec![tool_call]),
            Message::tool_result(
                "call_123".to_string(),
                json!({
                    "temperature": 22,
                    "condition": "sunny",
                    "humidity": 65
                })
                .to_string(),
            ),
        ],
        system_prompt: None,
        tools: Some(vec![weather_tool()]),
        model: model.to_string(),
        max_tokens: Some(200),
        temperature: Some(0.5),
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    }
}

/// Helper to assert that a JSON value contains a field
pub fn assert_json_has_field(value: &Value, field: &str) -> &Value {
    value.get(field).unwrap_or_else(|| panic!("Expected field '{}' not found in JSON: {}", field, value))
}

/// Helper to assert that a JSON array has expected length
pub fn assert_json_array_length(value: &Value, field: &str, expected_len: usize) {
    let array = value[field].as_array().unwrap_or_else(|| panic!("Field '{}' is not an array", field));
    assert_eq!(
        array.len(),
        expected_len,
        "Expected array '{}' to have {} elements, but got {}",
        field,
        expected_len,
        array.len()
    );
}

/// Helper to assert that a string contains a substring
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "Expected string to contain '{}', but got: {}",
        needle,
        haystack
    );
}

#[cfg(test)]
mod test_utils_tests {
    use super::*;

    #[test]
    fn simple_request_creates_valid_request() {
        let request = simple_request("test-model");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.model, "test-model");
        assert!(request.system_prompt.is_none());
        assert!(request.tools.is_none());
    }

    #[test]
    fn request_with_system_prompt_includes_system() {
        let request = request_with_system_prompt("test-model");
        assert!(request.system_prompt.is_some());
        assert!(request.system_prompt.unwrap().contains("weather"));
    }

    #[test]
    fn request_with_tools_includes_tools() {
        let request = request_with_tools("test-model");
        assert!(request.tools.is_some());
        let tools = request.tools.unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn weather_tool_has_required_fields() {
        let tool = weather_tool();
        assert_eq!(tool.name, "get_weather");
        assert!(!tool.description.is_empty());
    }

    #[test]
    fn sample_tool_call_has_valid_structure() {
        let call = sample_tool_call();
        assert_eq!(call.name, "get_weather");
        assert!(!call.id.is_empty());
        assert!(call.parameters.is_object());
    }
}
