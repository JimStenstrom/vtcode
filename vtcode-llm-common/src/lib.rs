//! Common utilities for VTCode LLM providers
//!
//! This crate provides shared utilities used across all VTCode LLM provider implementations:
//! - Error formatting
//! - Configuration helpers (resolve_model, override_base_url)
//! - Provider builder pattern
//! - Reasoning extraction utilities
//!
//! # Example
//! ```no_run
//! use vtcode_llm_common::{resolve_model, format_llm_error};
//!
//! let model = resolve_model(None, "default-model");
//! let error_msg = format_llm_error("MyProvider", "Connection failed");
//! ```

pub mod error;
pub mod config;
pub mod reasoning;

// Re-export commonly used functions
pub use error::format_llm_error;
pub use config::{resolve_model, override_base_url, ProviderBuilder};
pub use reasoning::{extract_reasoning_trace, split_reasoning_from_text, ReasoningBuffer};
