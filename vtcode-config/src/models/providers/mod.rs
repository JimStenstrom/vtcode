//! Provider-specific model helper functions and metadata
//!
//! This module contains provider-specific logic for different AI model providers.
//! Most model constants are defined in `crate::constants::models`.

pub mod openrouter;

// Re-export for convenience
pub use openrouter::OPENROUTER_ENTRIES;
