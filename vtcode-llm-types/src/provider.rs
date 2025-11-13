//! Universal LLM provider abstraction with API-specific role handling
//!
//! This module provides a unified interface for different LLM providers (OpenAI, Anthropic, Gemini)
//! while properly handling their specific requirements for message roles and tool calling.
//!
//! ## Message Role Mapping
//!
//! Different LLM providers have varying support for message roles, especially for tool calling:
//!
//! ### OpenAI API
//! - **Full Support**: `system`, `user`, `assistant`, `tool`
//! - **Tool Messages**: Must include `tool_call_id` to reference the original tool call
//! - **Tool Calls**: Only `assistant` messages can contain `tool_calls`
//!
//! ### Anthropic API
//! - **Standard Roles**: `user`, `assistant`
//! - **System Messages**: Can be hoisted to system parameter or treated as user messages
//! - **Tool Responses**: Converted to `user` messages (no separate tool role)
//! - **Tool Choice**: Supports `auto`, `any`, `tool`, `none` modes
//!
//! ### Gemini API
//! - **Conversation Roles**: Only `user` and `model` (not `assistant`)
//! - **System Messages**: Handled separately as `systemInstruction` parameter
//! - **Tool Responses**: Converted to `user` messages with `functionResponse` format
//! - **Function Calls**: Uses `functionCall` in `model` messages
//!
//! ## Best Practices
//!
//! 1. Always use `MessageRole::tool_response()` constructor for tool responses
//! 2. Validate messages using `validate_for_provider()` before sending
//! 3. Use appropriate role mapping methods for each provider
//! 4. Handle provider-specific constraints (e.g., Gemini's system instruction requirement)

use async_trait::async_trait;
use std::pin::Pin;

use crate::error::LLMError;
use crate::request::LLMRequest;
use crate::response::LLMResponse;

#[derive(Debug, Clone)]
pub enum LLMStreamEvent {
    Token { delta: String },
    Reasoning { delta: String },
    Completed { response: LLMResponse },
}

pub type LLMStream = Pin<Box<dyn futures::Stream<Item = Result<LLMStreamEvent, LLMError>> + Send>>;

/// Universal LLM provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider name (e.g., "gemini", "openai", "anthropic")
    fn name(&self) -> &str;

    /// Whether the provider has native streaming support
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Whether the provider surfaces structured reasoning traces for the given model
    fn supports_reasoning(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider accepts configurable reasoning effort for the model
    fn supports_reasoning_effort(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports structured tool calling for the given model
    fn supports_tools(&self, _model: &str) -> bool {
        true
    }

    /// Whether the provider understands parallel tool configuration payloads
    fn supports_parallel_tool_config(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports structured output (JSON schema guarantees)
    fn supports_structured_output(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports prompt/context caching
    fn supports_context_caching(&self, _model: &str) -> bool {
        false
    }

    /// Get the effective context window size for a model
    fn effective_context_size(&self, _model: &str) -> usize {
        // Default to 128k context window (common baseline)
        128_000
    }

    /// Generate completion
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError>;

    /// Stream completion (optional)
    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError>;

    /// Get supported models
    fn supported_models(&self) -> Vec<String>;

    /// Validate request for this provider
    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError>;
}
