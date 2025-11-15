//! Standalone OpenAI LLM provider for VTCode
//!
//! This crate provides a modular OpenAI provider implementation that can be used
//! independently or as part of the VTCode LLM ecosystem.
//!
//! ## Features
//!
//! - **Streaming**: Full support for streaming completions
//! - **Function Calling**: Support for OpenAI function calling / tools
//! - **Prompt Caching**: Automatic prompt caching support
//! - **Reasoning Models**: Support for o1/o3 reasoning models
//!
//! ## Example
//!
//! ```no_run
//! use vtcode_llm_openai::{OpenAIProvider, LLMProvider, LLMRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = OpenAIProvider::new("your-api-key".to_string());
//!
//!     let request = LLMRequest {
//!         messages: vec![Message::user("Hello, GPT!".to_string())],
//!         system_prompt: Some("You are a helpful assistant.".to_string()),
//!         tools: None,
//!         model: "gpt-4o-mini".to_string(),
//!         max_tokens: Some(1000),
//!         temperature: Some(0.7),
//!         stream: false,
//!         tool_choice: None,
//!         parallel_tool_calls: None,
//!         reasoning_effort: None,
//!     };
//!
//!     let response = provider.generate(request).await?;
//!     println!("Response: {:?}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod openai;
pub mod codex_prompt;

// Re-export universal LLM types from vtcode_llm_types
pub use vtcode_llm_types::{
    ContentPart, FinishReason, FunctionCall, FunctionDefinition, LLMError, LLMProvider,
    LLMRequest, LLMResponse, LLMResult, LLMStream, LLMStreamEvent, Message, MessageContent,
    MessageRole, ParallelToolConfig, ReasoningEffortLevel, SpecificFunctionChoice,
    SpecificToolChoice, ToolCall, ToolChoice, ToolDefinition, Usage,
};

// Re-export the provider implementation
pub use openai::OpenAIProvider;

// Re-export GPT-5 Codex utilities
pub use codex_prompt::gpt5_codex_developer_prompt;
