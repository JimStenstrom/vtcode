//! Request building for Gemini provider
//!
//! This module provides utilities for converting LLMRequest to Gemini API format,
//! including message conversion, tool definitions, and generation config.

use crate::gemini::function_calling::{
    FunctionCall as GeminiFunctionCall, FunctionCallingConfig, FunctionResponse,
};
use crate::gemini::models::SystemInstruction;
use crate::gemini::{
    Content, FunctionDeclaration, GenerateContentRequest, Part, Tool, ToolConfig,
};
use crate::tools::sanitize_function_parameters;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use vtcode_config::core::{GeminiPromptCacheMode, GeminiPromptCacheSettings};
use vtcode_llm_types::{LLMError, LLMRequest, MessageRole, ToolChoice};

/// Convert LLMRequest to Gemini GenerateContentRequest
///
/// This function handles:
/// - Message history conversion (including system messages as system_instruction)
/// - Tool definitions and function declarations
/// - Tool call and tool response mapping
/// - Generation configuration
/// - Prompt caching settings (if enabled)
pub fn convert_to_gemini_request(
    request: &LLMRequest,
    prompt_cache_enabled: bool,
    prompt_cache_settings: &GeminiPromptCacheSettings,
) -> Result<GenerateContentRequest, LLMError> {
    if prompt_cache_enabled
        && matches!(
            prompt_cache_settings.mode,
            GeminiPromptCacheMode::Explicit
        )
    {
        // Explicit cache handling requires separate cache lifecycle APIs which are
        // coordinated outside of the request payload. Placeholder ensures we surface
        // configuration usage even when implicit mode is active.
    }

    // Build mapping of tool call IDs to function names for tool responses
    let mut call_map: HashMap<String, String> = HashMap::new();
    for message in &request.messages {
        if message.role == MessageRole::Assistant
            && let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    call_map.insert(tool_call.id.clone(), tool_call.function.name.clone());
                }
            }
    }

    // Convert messages to Gemini Contents
    let mut contents: Vec<Content> = Vec::new();
    for message in &request.messages {
        // System messages are handled separately via system_instruction
        if message.role == MessageRole::System {
            continue;
        }

        let content_text = message.content.as_text();
        let mut parts: Vec<Part> = Vec::new();

        // Add text content for non-tool messages
        if message.role != MessageRole::Tool && !message.content.is_empty() {
            parts.push(Part::Text {
                text: content_text.clone(),
            });
        }

        // Add function calls for assistant messages
        if message.role == MessageRole::Assistant
            && let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    let parsed_args = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or_else(|_| json!({}));
                    parts.push(Part::FunctionCall {
                        function_call: GeminiFunctionCall {
                            name: tool_call.function.name.clone(),
                            args: parsed_args,
                            id: Some(tool_call.id.clone()),
                        },
                    });
                }
            }

        // Add function responses for tool messages
        if message.role == MessageRole::Tool {
            if let Some(tool_call_id) = &message.tool_call_id {
                let func_name = call_map
                    .get(tool_call_id)
                    .cloned()
                    .unwrap_or_else(|| tool_call_id.clone());

                // Try to parse and pretty-print JSON responses
                let response_text = serde_json::from_str::<Value>(&content_text)
                    .map(|value| {
                        serde_json::to_string_pretty(&value)
                            .unwrap_or_else(|_| content_text.clone())
                    })
                    .unwrap_or_else(|_| content_text.clone());

                let response_payload = json!({
                    "name": func_name.clone(),
                    "content": [{
                        "text": response_text
                    }]
                });

                parts.push(Part::FunctionResponse {
                    function_response: FunctionResponse {
                        name: func_name,
                        response: response_payload,
                    },
                });
            } else if !message.content.is_empty() {
                // Tool message without call_id - just add as text
                parts.push(Part::Text {
                    text: content_text.clone(),
                });
            }
        }

        if !parts.is_empty() {
            contents.push(Content {
                role: message.role.as_gemini_str().to_string(),
                parts,
            });
        }
    }

    // Convert tool definitions to Gemini function declarations
    let tools: Option<Vec<Tool>> = request.tools.as_ref().map(|definitions| {
        definitions
            .iter()
            .map(|tool| Tool {
                function_declarations: vec![FunctionDeclaration {
                    name: tool.function.name.clone(),
                    description: tool.function.description.clone(),
                    parameters: sanitize_function_parameters(tool.function.parameters.clone()),
                }],
            })
            .collect()
    });

    // Build generation config
    let mut generation_config = Map::new();
    if let Some(max_tokens) = request.max_tokens {
        generation_config.insert("maxOutputTokens".to_string(), json!(max_tokens));
    }
    if let Some(temp) = request.temperature {
        generation_config.insert("temperature".to_string(), json!(temp));
    }

    // Build tool config based on tool_choice
    let has_tools = request
        .tools
        .as_ref()
        .map(|defs| !defs.is_empty())
        .unwrap_or(false);
    let tool_config = if has_tools || request.tool_choice.is_some() {
        Some(match request.tool_choice.as_ref() {
            Some(ToolChoice::None) => ToolConfig {
                function_calling_config: FunctionCallingConfig::none(),
            },
            Some(ToolChoice::Any) => ToolConfig {
                function_calling_config: FunctionCallingConfig::any(),
            },
            Some(ToolChoice::Specific(spec)) => {
                let mut config = FunctionCallingConfig::any();
                if spec.tool_type == "function" {
                    config.allowed_function_names = Some(vec![spec.function.name.clone()]);
                }
                ToolConfig {
                    function_calling_config: config,
                }
            }
            _ => ToolConfig::auto(),
        })
    } else {
        None
    };

    Ok(GenerateContentRequest {
        contents,
        tools,
        tool_config,
        system_instruction: request
            .system_prompt
            .as_ref()
            .map(|text| SystemInstruction::new(text.clone())),
        generation_config: if generation_config.is_empty() {
            None
        } else {
            Some(Value::Object(generation_config))
        },
        reasoning_config: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtcode_config::core::GeminiPromptCacheSettings;
    use vtcode_llm_types::{
        Message, SpecificFunctionChoice, SpecificToolChoice, ToolCall, ToolDefinition,
    };

    #[test]
    fn convert_to_gemini_request_maps_history_and_system_prompt() {
        let mut assistant_message = Message::assistant("Sure thing".to_string());
        assistant_message.tool_calls = Some(vec![ToolCall::function(
            "call_1".to_string(),
            "list_files".to_string(),
            json!({ "path": "." }).to_string(),
        )]);

        let tool_response =
            Message::tool_response("call_1".to_string(), json!({ "result": "ok" }).to_string());

        let tool_def = ToolDefinition::function(
            "list_files".to_string(),
            "List files".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                }
            }),
        );

        let request = LLMRequest {
            messages: vec![
                Message::user("hello".to_string()),
                assistant_message,
                tool_response,
            ],
            system_prompt: Some("System prompt".to_string()),
            tools: Some(vec![tool_def]),
            model: "gemini-2.5-flash-preview".to_string(),
            max_tokens: Some(256),
            temperature: Some(0.4),
            stream: false,
            tool_choice: Some(ToolChoice::Specific(SpecificToolChoice {
                tool_type: "function".to_string(),
                function: SpecificFunctionChoice {
                    name: "list_files".to_string(),
                },
            })),
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let gemini_request = convert_to_gemini_request(
            &request,
            false,
            &GeminiPromptCacheSettings::default(),
        )
        .expect("conversion should succeed");

        let system_instruction = gemini_request
            .system_instruction
            .expect("system instruction should be present");
        assert!(matches!(
            system_instruction.parts.as_slice(),
            [Part::Text { text }] if text == "System prompt"
        ));

        assert_eq!(gemini_request.contents.len(), 3);
        assert_eq!(gemini_request.contents[0].role, "user");
        assert!(gemini_request.contents[1]
            .parts
            .iter()
            .any(|part| matches!(part, Part::FunctionCall { .. })));

        let tool_part = gemini_request.contents[2]
            .parts
            .iter()
            .find_map(|part| match part {
                Part::FunctionResponse { function_response } => Some(function_response),
                _ => None,
            })
            .expect("tool response part should exist");
        assert_eq!(tool_part.name, "list_files");
    }

    #[test]
    fn convert_includes_generation_config() {
        let request = LLMRequest {
            messages: vec![Message::user("test".to_string())],
            system_prompt: None,
            tools: None,
            model: "gemini-2.5-flash".to_string(),
            max_tokens: Some(500),
            temperature: Some(0.8),
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let gemini_request = convert_to_gemini_request(
            &request,
            false,
            &GeminiPromptCacheSettings::default(),
        )
        .expect("conversion should succeed");

        assert!(gemini_request.generation_config.is_some());
        let config = gemini_request.generation_config.unwrap();
        assert_eq!(config.get("maxOutputTokens").unwrap(), &json!(500));
        // Use approximate comparison for floating point
        let temp_value = config.get("temperature").unwrap().as_f64().unwrap();
        assert!((temp_value - 0.8).abs() < 0.001);
    }
}
