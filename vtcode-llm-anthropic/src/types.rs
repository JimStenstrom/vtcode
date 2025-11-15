//! Anthropic-specific types
//!
//! This module re-exports universal LLM types from vtcode_llm_types.
//! Anthropic does not require provider-specific configuration types.

// Re-export universal types from vtcode_llm_types
pub use vtcode_llm_types::{
    ContentPart, FinishReason, FunctionCall, FunctionDefinition, LLMError, LLMRequest,
    LLMResponse, LLMResult, Message, MessageContent, MessageRole, ParallelToolConfig,
    ReasoningEffortLevel, SpecificFunctionChoice, SpecificToolChoice, ToolCall, ToolChoice,
    ToolDefinition, Usage,
};
