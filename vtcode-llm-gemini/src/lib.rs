//! Gemini (Google AI) provider implementation for VTCode
//!
//! This crate provides the Google Gemini LLM provider implementation, including:
//! - GeminiProvider: Main provider interface
//! - Gemini API client with streaming support
//! - Function calling integration
//! - Model definitions and request/response types
//!
//! # Features
//! - Full streaming support with delta accumulation
//! - Function/tool calling with proper argument handling
//! - Prompt caching (context caching) support
//! - Code execution capabilities
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use vtcode_llm_gemini::GeminiProvider;
//! use vtcode_llm_types::LLMProvider;
//!
//! let provider = GeminiProvider::new("your-api-key".to_string());
//! // Use the provider with LLMProvider trait methods
//! ```

pub mod common;
pub mod gemini;
pub mod provider;

// Re-export main types
pub use provider::GeminiProvider;
pub use provider::sanitize_function_parameters;

// Re-export gemini API types for consumers who need them
pub use gemini::{
    Candidate, Client, ClientConfig, Content, FunctionCall, FunctionCallingConfig,
    FunctionDeclaration, FunctionResponse, GenerateContentRequest, GenerateContentResponse, Part,
    RetryConfig, StreamingCandidate, StreamingConfig, StreamingError, StreamingMetrics,
    StreamingProcessor, StreamingResponse, SystemInstruction, Tool, ToolConfig,
};

// Re-export common utilities
pub use common::{
    extract_prompt_cache_settings, forward_prompt_cache_with_state, override_base_url,
    resolve_model,
};
