//! Z.AI provider implementation for VTCode
//!
//! This crate provides the Z.AI LLM provider implementation, including:
//! - ZAIProvider: Main provider interface
//! - Z.AI API client with non-streaming support
//! - Tool calling integration
//! - Reasoning model support (GLM-4.5, GLM-4.6)
//!
//! # Features
//! - Tool calling with proper argument handling
//! - Reasoning model support (GLM-4.5, GLM-4.6 variants)
//! - Rate limit error detection
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use vtcode_llm_zai::ZAIProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! let provider = ZAIProvider::new("your-api-key".to_string());
//! // Use the provider with LLMProvider trait methods
//! ```

pub mod zai;

// Re-export main types
pub use zai::ZAIProvider;
