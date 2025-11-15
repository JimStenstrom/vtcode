//! Model registry and lookup functions

use super::model_id::ModelId;

/// Get vendor groups for OpenRouter models
pub fn openrouter_vendor_groups() -> Vec<(&'static str, &'static [ModelId])> {
    ModelId::openrouter_vendor_groups()
}

/// Get all OpenRouter models
pub fn openrouter_models() -> Vec<ModelId> {
    ModelId::openrouter_models()
}
