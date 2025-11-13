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
//! use vtcode_llm_openai::{OpenAIProvider, LLMProvider, LLMRequest, Message, MessageRole, MessageContent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = OpenAIProvider::new("your-api-key".to_string());
//!
//!     let request = LLMRequest {
//!         messages: vec![Message {
//!             role: MessageRole::User,
//!             content: MessageContent::Text("Hello, world!".to_string()),
//!             reasoning: None,
//!             tool_calls: None,
//!             tool_call_id: None,
//!         }],
//!         system_prompt: None,
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
//!     println!("Response: {}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod provider;
pub mod openai;

// Re-export main types and traits
pub use types::*;
pub use provider::{LLMProvider, LLMStream, LLMStreamEvent};
pub use openai::OpenAIProvider;
