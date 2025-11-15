//! DeepSeek provider implementation for VTCode
//!
//! This crate provides the DeepSeek LLM provider implementation, including:
//! - DeepSeekProvider: Main provider interface
//! - DeepSeek API client with message serialization
//! - Function calling integration
//! - Reasoning model support (DeepSeek-R1)
//!
//! # Features
//! - OpenAI-compatible API with tool calling
//! - Native reasoning support for DeepSeek-R1 models
//! - Prompt caching with metrics surface
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use vtcode_llm_deepseek::DeepSeekProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! let provider = DeepSeekProvider::new("your-api-key".to_string());
//! // Use the provider with LLMProvider trait methods
//! ```

pub mod deepseek;

// Re-export main types
pub use deepseek::DeepSeekProvider;
