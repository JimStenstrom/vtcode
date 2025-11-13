//! Standalone Anthropic LLM provider for VTCode
//!
//! This crate provides a modular Anthropic provider implementation that can be used
//! independently or as part of the VTCode LLM ecosystem.
//!
//! ## Features
//!
//! - **Streaming**: Full support for streaming completions (coming soon)
//! - **Tool Calling**: Support for Anthropic tool calling
//! - **Prompt Caching**: Automatic prompt caching support with 5m and 1h TTL
//! - **Reasoning Models**: Support for Claude 3.7 Sonnet and 4.5 Sonnet reasoning models
//!
//! ## Example
//!
//! ```no_run
//! use vtcode_llm_anthropic::{AnthropicProvider, LLMProvider, LLMRequest, Message, MessageRole, MessageContent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = AnthropicProvider::new("your-api-key".to_string());
//!
//!     let request = LLMRequest {
//!         messages: vec![Message::user("Hello, Claude!".to_string())],
//!         system_prompt: Some("You are a helpful assistant.".to_string()),
//!         tools: None,
//!         model: "claude-sonnet-4-5-20250929".to_string(),
//!         max_tokens: Some(1000),
//!         temperature: Some(0.7),
//!         stream: false,
//!         tool_choice: None,
//!         parallel_tool_calls: None,
//!         parallel_tool_config: None,
//!         reasoning_effort: None,
//!     };
//!
//!     let response = provider.generate(request).await?;
//!     println!("Response: {}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod provider;
pub mod anthropic;

// Re-export main types and traits
pub use types::*;
pub use provider::{LLMProvider, LLMStream, LLMStreamEvent};
pub use anthropic::AnthropicProvider;
