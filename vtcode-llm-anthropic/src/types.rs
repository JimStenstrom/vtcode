//! Anthropic-specific types
//!
//! This module contains Anthropic-specific configuration types.
//! Universal LLM types are re-exported from vtcode_llm_types.

use serde::{Deserialize, Serialize};

// Re-export universal types from vtcode_llm_types
pub use vtcode_llm_types::{
    ContentPart, FinishReason, FunctionCall, FunctionDefinition, LLMError, LLMRequest,
    LLMResponse, LLMResult, Message, MessageContent, MessageRole, ParallelToolConfig,
    ReasoningEffortLevel, SpecificFunctionChoice, SpecificToolChoice, ToolCall, ToolChoice,
    ToolDefinition, Usage,
};

/// Prompt caching configuration for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCachingConfig {
    pub enabled: bool,
    pub max_breakpoints: u32,
    pub cache_user_messages: bool,
    pub cache_system_messages: bool,
    pub extended_ttl_seconds: Option<u64>,
}

impl Default for PromptCachingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_breakpoints: 3,
            cache_user_messages: false,
            cache_system_messages: true,
            extended_ttl_seconds: None,
        }
    }
}
