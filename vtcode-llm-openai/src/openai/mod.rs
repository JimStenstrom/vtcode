//! OpenAI LLM Provider
//!
//! Modular implementation of the OpenAI provider supporting GPT-4, GPT-4-turbo,
//! GPT-3.5-turbo, and o1/o3 models.
//!
//! ## Module Organization
//!
//! - `config`: Configuration constants and defaults
//! - `request`: Request building and serialization
//! - `response`: Response parsing
//! - `streaming`: SSE streaming implementation
//! - `provider`: Core provider struct and LLMProvider trait implementation
//!
//! ## Example
//!
//! ```no_run
//! use vtcode_llm_openai::OpenAIProvider;
//! use vtcode_llm_types::{LLMProvider, LLMRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = OpenAIProvider::new("your-api-key".to_string());
//!
//!     let request = LLMRequest {
//!         messages: vec![Message::user("Hello!".to_string())],
//!         system_prompt: None,
//!         tools: None,
//!         model: "gpt-4o-mini".to_string(),
//!         max_tokens: None,
//!         temperature: None,
//!         stream: false,
//!         tool_choice: None,
//!         parallel_tool_calls: None,
//!         parallel_tool_config: None,
//!         reasoning_effort: None,
//!     };
//!
//!     let response = provider.generate(request).await?;
//!     println!("{:?}", response.content);
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod provider;
pub mod request;
pub mod response;
pub mod streaming;

#[cfg(test)]
mod tests;

// Re-export the main provider
pub use provider::OpenAIProvider;

// Re-export constants for convenience
pub use config::{DEFAULT_MODEL, OPENAI_API_BASE};
