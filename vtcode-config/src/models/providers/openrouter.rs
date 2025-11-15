//! OpenRouter model metadata and helpers

use crate::models::model_id;

/// Re-export OpenRouter generated entries for external access
pub use model_id::openrouter_generated::ENTRIES as OPENROUTER_ENTRIES;

/// Check if an OpenRouter model supports reasoning based on model name patterns
pub fn supports_reasoning_pattern(model: &str) -> bool {
    model.contains("-reasoning") || model.contains("-extended") || model.contains("-thinking")
}
