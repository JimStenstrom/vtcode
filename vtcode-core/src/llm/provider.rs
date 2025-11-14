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
//!
//! ## Example Usage
//!
//! ```rust
//! use vtcode_core::llm::provider::{Message, MessageRole};
//!
//! // Create a proper tool response message
//! let tool_response = Message::tool_response(
//!     "call_123".to_string(),
//!     "Tool execution completed successfully".to_string()
//! );
//!
//! // Validate for specific provider
//! tool_response.validate_for_provider("openai").unwrap();
//! ```

use async_stream::try_stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

// Re-export Phase 3 provider types from vtcode-llm-types
pub use vtcode_llm_types::{
    // Provider trait and streaming
    LLMProvider, LLMStream, LLMStreamEvent,
    // Error types
    LLMError as LLMProviderError,
    // Request and response types
    LLMRequest, LLMResponse, Usage, FinishReason,
    // Message types
    Message, MessageContent, MessageRole, ContentPart,
    // Tool types
    ToolCall, ToolChoice, ToolDefinition, FunctionCall, FunctionDefinition,
    SpecificToolChoice, SpecificFunctionChoice,
    // Other types
    ParallelToolConfig, ReasoningEffortLevel,
};

// Type alias for backward compatibility
pub type LLMError = LLMProviderError;

// Implement conversion from provider::LLMError to llm::types::LLMError
impl From<LLMError> for crate::llm::types::LLMError {
    fn from(err: LLMError) -> crate::llm::types::LLMError {
        match err {
            LLMError::AuthenticationError(msg) => crate::llm::types::LLMError::ApiError(msg),
            LLMError::RateLimit => crate::llm::types::LLMError::RateLimit,
            LLMError::InvalidRequest(msg) => crate::llm::types::LLMError::InvalidRequest(msg),
            LLMError::NetworkError(msg) => crate::llm::types::LLMError::NetworkError(msg),
            LLMError::Provider(msg) => crate::llm::types::LLMError::ApiError(msg),
            LLMError::ApiError(msg) => crate::llm::types::LLMError::ApiError(msg),
            LLMError::SerializationError(msg) => crate::llm::types::LLMError::ApiError(format!("Serialization error: {}", msg)),
            LLMError::ModelNotFound(msg) => crate::llm::types::LLMError::ApiError(format!("Model not found: {}", msg)),
            LLMError::Timeout(msg) => crate::llm::types::LLMError::NetworkError(format!("Timeout: {}", msg)),
            LLMError::Other(msg) => crate::llm::types::LLMError::ApiError(msg),
        }
    }
}
