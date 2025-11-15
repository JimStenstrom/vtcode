//! Moonshot.ai provider implementation for VTCode
//!
//! This crate provides the Moonshot.ai (Kimi) LLM provider implementation, including:
//! - MoonshotProvider: Main provider interface
//! - Moonshot API client with message serialization
//! - Function calling integration
//! - Reasoning model support (Kimi-K2-Thinking models)
//!
//! # Features
//! - OpenAI-compatible API with tool calling
//! - Native reasoning support for Kimi-K2-Thinking models
//! - Reasoning effort configuration
//! - Heavy Mode support for enhanced reasoning
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use vtcode_llm_moonshot::MoonshotProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! let provider = MoonshotProvider::new("your-api-key".to_string());
//! // Use the provider with LLMProvider trait methods
//! ```

pub mod moonshot;

// Re-export main types
pub use moonshot::MoonshotProvider;
