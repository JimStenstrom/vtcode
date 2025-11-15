//! LLM response processing and tool call extraction

use super::{AgentState, ExecutionContext, Transition};
use crate::config::constants::tools;
use crate::config::models::{ModelId, Provider as ModelProvider};
use crate::llm::provider::{LLMResponse, ToolCall};
use anyhow::Result;
use serde_json::Value;
use vtcode_llm_gemini::{Content, Part};

/// Handles processing of LLM responses and extracting tool calls
pub struct ResponseProcessor;

impl ResponseProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process LLM response and determine next action
    pub fn process_response(
        &self,
        response: LLMResponse,
        provider_kind: ModelProvider,
        turn: usize,
        _ctx: &mut ExecutionContext,
    ) -> Result<ProcessedResponse> {
        let response_text = response.content.clone().unwrap_or_default();
        let reasoning = response.reasoning.clone();

        // Extract tool calls based on provider type
        let mut tool_calls = response.tool_calls.clone().unwrap_or_default();

        // For providers that support tool calls natively (OpenAI, Anthropic, DeepSeek)
        if matches!(
            provider_kind,
            ModelProvider::OpenAI | ModelProvider::Anthropic | ModelProvider::DeepSeek
        ) {
            // Check for textual run_terminal_cmd if no tool calls present
            if tool_calls.is_empty() {
                if let Some(args_value) = detect_textual_run_terminal_cmd(&response_text) {
                    let call_id = format!("textual_call_{}_{}", turn, turn);
                    let args_json = serde_json::to_string(&args_value)?;
                    tool_calls = vec![ToolCall::function(
                        call_id,
                        tools::RUN_COMMAND.to_string(),
                        args_json,
                    )];
                }
            }

            // Check for loop detection message
            const LOOP_DETECTED_MESSAGE: &str = "A potential loop was detected";
            if response_text.contains(LOOP_DETECTED_MESSAGE) {
                return Ok(ProcessedResponse {
                    response_text,
                    reasoning,
                    tool_calls: vec![],
                    had_tool_call: false,
                    loop_detected: true,
                });
            }
        }

        let had_tool_call = !tool_calls.is_empty();

        Ok(ProcessedResponse {
            response_text,
            reasoning,
            tool_calls,
            had_tool_call,
            loop_detected: false,
        })
    }

    /// Determine next state after processing response
    pub fn next_state(&self, processed: &ProcessedResponse) -> Transition {
        if processed.loop_detected {
            // Loop detected - should terminate
            return Transition::Terminal(crate::core::agent::task::TaskOutcome::ToolLoopLimitReached);
        }

        if let Some(first_call) = processed.tool_calls.first() {
            // Execute first tool call
            let args = first_call
                .parsed_arguments()
                .unwrap_or_else(|_| serde_json::json!({}));

            Transition::Continue(AgentState::ExecutingTool {
                tool_name: first_call.function.name.clone(),
                tool_args: args,
                call_id: first_call.id.clone(),
            })
        } else {
            // No tool calls - check continuation
            Transition::Continue(AgentState::CheckingContinuation)
        }
    }
}

impl Default for ResponseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Processed response data
#[derive(Debug, Clone)]
pub struct ProcessedResponse {
    pub response_text: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub had_tool_call: bool,
    pub loop_detected: bool,
}

/// Detect textual run_terminal_cmd in fenced code blocks
fn detect_textual_run_terminal_cmd(text: &str) -> Option<Value> {
    const FENCE_PREFIXES: [&str; 2] = ["```tool:run_terminal_cmd", "```run_terminal_cmd"];

    let (start_idx, prefix) = FENCE_PREFIXES
        .iter()
        .filter_map(|candidate| text.find(candidate).map(|idx| (idx, *candidate)))
        .min_by_key(|(idx, _)| *idx)?;

    let mut remainder = &text[start_idx + prefix.len()..];
    if remainder.starts_with('\r') {
        remainder = &remainder[1..];
    }
    remainder = remainder.strip_prefix('\n')?;

    let fence_close = remainder.find("```")?;
    let block = remainder[..fence_close].trim();
    if block.is_empty() {
        return None;
    }

    let parsed = serde_json::from_str::<Value>(block)
        .or_else(|_| json5::from_str::<Value>(block))
        .ok()?;
    parsed.as_object()?;
    Some(parsed)
}
