//! OpenRouter LLM Provider for VTCode
//!
//! This crate provides an implementation of the LLM provider interface for OpenRouter,
//! allowing VTCode to interact with OpenRouter's API which provides access to multiple
//! LLM models through a unified interface.
//!
//! ## Features
//!
//! - Support for OpenAI-compatible chat completions API
//! - Streaming and non-streaming responses
//! - Tool/function calling support with automatic fallback for unsupported models
//! - Reasoning support for compatible models
//! - Prompt caching configuration
//! - Both `/chat/completions` and `/responses` API endpoints
//!
//! ## Usage
//!
//! ```rust,no_run
//! use vtcode_llm_openrouter::OpenRouterProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! #[tokio::main]
//! async fn main() {
//!     let provider = OpenRouterProvider::new("your-api-key".to_string());
//!     // Use the provider...
//! }
//! ```

mod reasoning;
mod common;
mod codex_prompt;
mod shared;
mod provider;

pub use provider::OpenRouterProvider;

// Re-export commonly used types from vtcode_llm_types for convenience
pub use vtcode_llm_types::{
    LLMProvider, LLMRequest, LLMResponse, LLMStream, LLMStreamEvent,
    Message, MessageRole, ToolCall, ToolChoice, ToolDefinition,
};
