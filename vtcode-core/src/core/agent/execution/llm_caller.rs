//! LLM request handling

use super::{AgentState, ExecutionContext, Transition};
use crate::config::types::ReasoningEffortLevel;
use crate::llm::provider::{LLMProvider, LLMRequest, LLMResponse, Message, ToolDefinition};
use anyhow::Result;

/// Handles LLM API calls (both streaming and non-streaming)
pub struct LLMCaller;

impl LLMCaller {
    pub fn new() -> Self {
        Self
    }

    /// Call the LLM provider with prepared messages
    pub async fn call_llm(
        &self,
        provider: &dyn LLMProvider,
        messages: Vec<Message>,
        system_instruction: String,
        tools: &[ToolDefinition],
        model: &str,
        reasoning_effort: Option<ReasoningEffortLevel>,
        parallel_tool_config: Option<crate::llm::provider::ParallelToolConfig>,
        supports_streaming: bool,
        ctx: &ExecutionContext,
    ) -> Result<Transition> {
        // Build LLM request
        let request = self.build_request(
            messages,
            system_instruction,
            tools,
            model,
            reasoning_effort,
            parallel_tool_config,
            provider.supports_reasoning_effort(model),
            supports_streaming,
        );

        // For now, we'll use non-streaming only to simplify the refactoring
        // Streaming will be added back in a follow-up enhancement
        let response = provider.generate(request).await?;

        // Transition to processing response
        Ok(Transition::Continue(AgentState::ProcessingResponse {
            response,
        }))
    }

    /// Build LLM request with all necessary parameters
    fn build_request(
        &self,
        messages: Vec<Message>,
        system_instruction: String,
        tools: &[ToolDefinition],
        model: &str,
        reasoning_effort: Option<ReasoningEffortLevel>,
        parallel_tool_config: Option<crate::llm::provider::ParallelToolConfig>,
        supports_reasoning_effort: bool,
        _supports_streaming: bool,
    ) -> LLMRequest {
        LLMRequest {
            messages,
            system_prompt: Some(system_instruction),
            tools: Some(tools.to_vec()),
            model: model.to_string(),
            max_tokens: Some(2000),
            temperature: Some(0.7),
            stream: false, // Disabled for simplicity in initial refactoring
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config,
            reasoning_effort: if supports_reasoning_effort {
                reasoning_effort
            } else {
                None
            },
        }
    }
}

impl Default for LLMCaller {
    fn default() -> Self {
        Self::new()
    }
}
