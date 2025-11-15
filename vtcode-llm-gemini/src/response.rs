//! Response parsing for Gemini provider
//!
//! This module provides utilities for parsing and converting Gemini API responses
//! to the standard LLMResponse format.

use crate::gemini::streaming::StreamingResponse;
use crate::gemini::{Candidate, GenerateContentResponse, Part};
use vtcode_llm_types::{FinishReason, FunctionCall, LLMError, LLMResponse, ToolCall};

/// Apply streaming delta to accumulator
///
/// Gemini's streaming API can send:
/// - Incremental chunks (new text appended)
/// - Full replays (entire content sent again)
/// - Rewrites (shorter content replacing longer)
///
/// This function detects the pattern and returns only the new delta text.
pub fn apply_stream_delta(accumulator: &mut String, chunk: &str) -> Option<String> {
    if chunk.is_empty() {
        return None;
    }

    // Case 1: Chunk extends accumulator (e.g., "Hello" -> "Hello world")
    if chunk.starts_with(accumulator.as_str()) {
        let delta = &chunk[accumulator.len()..];
        if delta.is_empty() {
            return None;
        }
        accumulator.clear();
        accumulator.push_str(chunk);
        return Some(delta.to_string());
    }

    // Case 2: Accumulator extends chunk (rewrite/shorter version)
    if accumulator.starts_with(chunk) {
        accumulator.clear();
        accumulator.push_str(chunk);
        return None;
    }

    // Case 3: New unrelated chunk (incremental)
    accumulator.push_str(chunk);
    Some(chunk.to_string())
}

/// Convert Gemini response to LLMResponse
pub fn convert_from_gemini_response(
    response: GenerateContentResponse,
) -> Result<LLMResponse, LLMError> {
    let mut candidates = response.candidates.into_iter();
    let candidate = candidates.next().ok_or_else(|| {
        let formatted_error = "Gemini: No candidate in response".to_string();
        LLMError::Provider(formatted_error)
    })?;

    if candidate.content.parts.is_empty() {
        return Ok(LLMResponse {
            content: Some(String::new()),
            tool_calls: None,
            usage: None,
            finish_reason: FinishReason::Stop,
            reasoning: None,
            reasoning_details: None,
        });
    }

    let mut text_content = String::new();
    let mut tool_calls = Vec::new();

    for part in candidate.content.parts {
        match part {
            Part::Text { text } => {
                text_content.push_str(&text);
            }
            Part::FunctionCall { function_call } => {
                let call_id = function_call.id.clone().unwrap_or_else(|| {
                    format!(
                        "call_{}_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos(),
                        tool_calls.len()
                    )
                });
                tool_calls.push(ToolCall {
                    id: call_id,
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: function_call.name,
                        arguments: serde_json::to_string(&function_call.args)
                            .unwrap_or_else(|_| "{}".to_string()),
                    },
                });
            }
            Part::FunctionResponse { .. } => {
                // Ignore echoed tool responses to avoid duplicating tool output
            }
        }
    }

    let finish_reason = match candidate.finish_reason.as_deref() {
        Some("STOP") => FinishReason::Stop,
        Some("MAX_TOKENS") => FinishReason::Length,
        Some("SAFETY") => FinishReason::ContentFilter,
        Some("FUNCTION_CALL") => FinishReason::ToolCalls,
        Some(other) => FinishReason::Error(other.to_string()),
        None => FinishReason::Stop,
    };

    Ok(LLMResponse {
        content: if text_content.is_empty() {
            None
        } else {
            Some(text_content)
        },
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
        usage: None,
        finish_reason,
        reasoning: None,
        reasoning_details: None,
    })
}

/// Convert streaming response to LLMResponse
pub fn convert_from_streaming_response(
    response: StreamingResponse,
) -> Result<LLMResponse, LLMError> {
    let converted_candidates: Vec<Candidate> = response
        .candidates
        .into_iter()
        .map(|candidate| Candidate {
            content: candidate.content,
            finish_reason: candidate.finish_reason,
        })
        .collect();

    let converted = GenerateContentResponse {
        candidates: converted_candidates,
        prompt_feedback: None,
        usage_metadata: response.usage_metadata,
    };

    convert_from_gemini_response(converted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gemini::function_calling::FunctionCall as GeminiFunctionCall;
    use crate::gemini::Content;
    use serde_json::json;

    #[test]
    fn apply_stream_delta_handles_replayed_chunks() {
        let mut acc = String::new();
        assert_eq!(
            apply_stream_delta(&mut acc, "Hello"),
            Some("Hello".to_string())
        );
        assert_eq!(
            apply_stream_delta(&mut acc, "Hello world"),
            Some(" world".to_string())
        );
        assert_eq!(apply_stream_delta(&mut acc, "Hello world"), None);
        assert_eq!(acc, "Hello world");
    }

    #[test]
    fn apply_stream_delta_handles_incremental_chunks() {
        let mut acc = String::new();
        assert_eq!(
            apply_stream_delta(&mut acc, "Hello"),
            Some("Hello".to_string())
        );
        assert_eq!(
            apply_stream_delta(&mut acc, " there"),
            Some(" there".to_string())
        );
        assert_eq!(acc, "Hello there");
    }

    #[test]
    fn apply_stream_delta_handles_rewrites() {
        let mut acc = String::new();
        assert_eq!(
            apply_stream_delta(&mut acc, "Hello world"),
            Some("Hello world".to_string())
        );
        assert_eq!(apply_stream_delta(&mut acc, "Hello"), None);
        assert_eq!(acc, "Hello");
    }

    #[test]
    fn convert_from_gemini_response_extracts_tool_calls() {
        let response = GenerateContentResponse {
            candidates: vec![Candidate {
                content: Content {
                    role: "model".to_string(),
                    parts: vec![
                        Part::Text {
                            text: "Here you go".to_string(),
                        },
                        Part::FunctionCall {
                            function_call: GeminiFunctionCall {
                                name: "list_files".to_string(),
                                args: json!({ "path": "." }),
                                id: Some("call_1".to_string()),
                            },
                        },
                    ],
                },
                finish_reason: Some("FUNCTION_CALL".to_string()),
            }],
            prompt_feedback: None,
            usage_metadata: None,
        };

        let llm_response =
            convert_from_gemini_response(response).expect("conversion should succeed");

        assert_eq!(llm_response.content.as_deref(), Some("Here you go"));
        let calls = llm_response
            .tool_calls
            .expect("tool call should be present");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "list_files");
        assert!(calls[0].function.arguments.contains("path"));
        assert_eq!(llm_response.finish_reason, FinishReason::ToolCalls);
    }

    #[test]
    fn convert_from_gemini_response_handles_empty_parts() {
        let response = GenerateContentResponse {
            candidates: vec![Candidate {
                content: Content {
                    role: "model".to_string(),
                    parts: vec![],
                },
                finish_reason: Some("STOP".to_string()),
            }],
            prompt_feedback: None,
            usage_metadata: None,
        };

        let llm_response =
            convert_from_gemini_response(response).expect("conversion should succeed");
        assert_eq!(llm_response.content, Some(String::new()));
        assert_eq!(llm_response.finish_reason, FinishReason::Stop);
    }
}
