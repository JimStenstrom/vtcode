//! LLM provider trait for Anthropic

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::types::{LLMError, LLMRequest, LLMResponse};

/// Streaming event from LLM
#[derive(Debug, Clone)]
pub enum LLMStreamEvent {
    /// Content delta received
    Delta {
        content: String,
        reasoning: Option<String>,
    },
    /// Streaming completed with final response
    Completed { response: LLMResponse },
    /// Error occurred during streaming
    Error { error: LLMError },
}

/// Type alias for LLM stream
pub type LLMStream = Pin<Box<dyn Stream<Item = Result<LLMStreamEvent, LLMError>> + Send>>;

/// LLM provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Whether the provider supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Whether the provider supports reasoning for the model
    fn supports_reasoning(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports reasoning effort control
    fn supports_reasoning_effort(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports structured tool calling
    fn supports_tools(&self, _model: &str) -> bool {
        true
    }

    /// Whether the provider supports parallel tool configuration
    fn supports_parallel_tool_config(&self, _model: &str) -> bool {
        false
    }

    /// Whether the provider supports prompt caching
    fn supports_context_caching(&self, _model: &str) -> bool {
        false
    }

    /// Get effective context window size
    fn effective_context_size(&self, _model: &str) -> usize {
        200_000  // Anthropic's default context window
    }

    /// Generate completion
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError>;

    /// Stream completion
    async fn stream(&self, _request: LLMRequest) -> Result<LLMStream, LLMError> {
        Err(LLMError::Provider(
            "Streaming not supported".to_string(),
        ))
    }

    /// Get supported models
    fn supported_models(&self) -> Vec<String>;

    /// Validate request
    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        // Basic validation
        if request.messages.is_empty() {
            return Err(LLMError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}
