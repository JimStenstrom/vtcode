//! OpenAI response parsing utilities

use serde_json::Value;
use vtcode_llm_types::{FinishReason, FunctionCall, LLMError, LLMResponse, ToolCall, Usage};

/// Parse OpenAI API response into LLMResponse
pub fn parse_response(response_json: Value) -> Result<LLMResponse, LLMError> {
    let choices = response_json
        .get("choices")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LLMError::Provider("Invalid response: missing choices".to_string()))?;

    if choices.is_empty() {
        return Err(LLMError::Provider("No choices in response".to_string()));
    }

    let choice = &choices[0];
    let message = choice.get("message").ok_or_else(|| {
        LLMError::Provider("Invalid response: missing message".to_string())
    })?;

    let content = message
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Parse tool calls if present
    let tool_calls = message.get("tool_calls").and_then(|tc| {
        tc.as_array().map(|arr| {
            arr.iter()
                .filter_map(|call| {
                    Some(ToolCall {
                        id: call.get("id")?.as_str()?.to_string(),
                        call_type: call.get("type")?.as_str()?.to_string(),
                        function: FunctionCall {
                            name: call.get("function")?.get("name")?.as_str()?.to_string(),
                            arguments: call.get("function")?.get("arguments")?.as_str()?.to_string(),
                        },
                    })
                })
                .collect()
        })
    });

    // Parse usage
    let usage = response_json.get("usage").and_then(|u| {
        Some(Usage {
            prompt_tokens: u.get("prompt_tokens")?.as_u64()? as u32,
            completion_tokens: u.get("completion_tokens")?.as_u64()? as u32,
            total_tokens: u.get("total_tokens")?.as_u64()? as u32,
            cached_prompt_tokens: u
                .get("prompt_tokens_details")
                .and_then(|d| d.get("cached_tokens"))
                .and_then(|t| t.as_u64())
                .map(|t| t as u32),
            cache_creation_tokens: None,
            cache_read_tokens: None,
        })
    });

    // Parse finish reason
    let finish_reason = choice
        .get("finish_reason")
        .and_then(|fr| fr.as_str())
        .map(|fr_str| match fr_str {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            "function_call" => FinishReason::ToolCalls,
            _ => FinishReason::Error(fr_str.to_string()),
        })
        .unwrap_or(FinishReason::Stop);

    Ok(LLMResponse {
        content: Some(content),
        usage,
        reasoning: None,
        tool_calls,
        finish_reason,
        reasoning_details: None,
    })
}
