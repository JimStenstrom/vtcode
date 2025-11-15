//! OpenAI request building utilities

use serde_json::{Value, json};
use vtcode_llm_types::{Message, MessageRole, ToolDefinition};

/// Serialize messages for OpenAI API format
pub fn serialize_messages(messages: &[Message], system_prompt: Option<&String>) -> Vec<Value> {
    let mut result = Vec::new();

    // Add system prompt if provided
    if let Some(system) = system_prompt {
        result.push(json!({
            "role": "system",
            "content": system
        }));
    }

    // Add messages
    for msg in messages {
        let mut message_json = json!({
            "role": match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            },
            "content": msg.content.as_text(),
        });

        // Add tool_call_id if present
        if let Some(tool_call_id) = &msg.tool_call_id {
            message_json["tool_call_id"] = json!(tool_call_id);
        }

        // Add tool_calls if present
        if let Some(tool_calls) = &msg.tool_calls {
            let tool_calls_json: Vec<Value> = tool_calls
                .iter()
                .map(|tc| {
                    json!({
                        "id": tc.id,
                        "type": tc.call_type,
                        "function": {
                            "name": tc.function.name,
                            "arguments": tc.function.arguments,
                        }
                    })
                })
                .collect();
            message_json["tool_calls"] = json!(tool_calls_json);
        }

        result.push(message_json);
    }

    result
}

/// Serialize tools for OpenAI API format
pub fn serialize_tools(tools: &[ToolDefinition]) -> Option<Vec<Value>> {
    if tools.is_empty() {
        return None;
    }

    Some(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.function.name,
                        "description": tool.function.description,
                        "parameters": tool.function.parameters,
                    }
                })
            })
            .collect(),
    )
}
