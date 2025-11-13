//! Core LLM types and traits for vtcode
//!
//! This crate provides the foundational types and traits needed for LLM provider
//! implementations, breaking circular dependencies between vtcode-core and vtcode-llm.

pub mod error;
pub mod message;
pub mod provider;
pub mod request;
pub mod response;

// Re-export commonly used types
pub use error::{LLMError, LLMResult};
pub use message::{ContentPart, Message, MessageContent, MessageRole};
pub use provider::{LLMProvider, LLMStream, LLMStreamEvent};
pub use request::{
    FunctionCall, FunctionDefinition, LLMRequest, ParallelToolConfig, ReasoningEffortLevel,
    SpecificFunctionChoice, SpecificToolChoice, ToolCall, ToolChoice, ToolDefinition,
};
pub use response::{FinishReason, LLMResponse, Usage};

use serde::{Deserialize, Serialize};

/// Backend kind for LLM providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendKind {
    Gemini,
    OpenAI,
    Anthropic,
    DeepSeek,
    OpenRouter,
    Ollama,
    XAI,
    ZAI,
    Moonshot,
}
