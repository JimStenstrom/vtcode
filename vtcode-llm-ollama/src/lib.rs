//! Ollama provider implementation for VTCode
//!
//! This crate provides the Ollama LLM provider implementation, including:
//! - OllamaProvider: Main provider interface
//! - Ollama API client with streaming support
//! - Tool calling integration
//! - Model fetching and management
//!
//! # Features
//! - Full streaming support with delta accumulation
//! - Tool calling with proper argument handling
//! - Local and cloud model support
//! - Reasoning model support (deepseek-r1, qwen2.5-r1)
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use vtcode_llm_ollama::OllamaProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! let provider = OllamaProvider::new("your-api-key".to_string());
//! // Use the provider with LLMProvider trait methods
//! ```

pub mod ollama;

// Re-export main types
pub use ollama::{OllamaProvider, fetch_ollama_models};
