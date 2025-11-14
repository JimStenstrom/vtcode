use super::provider::{LLMError, LLMProvider};
use super::factory::create_provider_for_model;
use crate::config::models::ModelId;

/// Type-erased LLM provider (Phase 3 architecture)
pub type AnyClient = Box<dyn LLMProvider>;

/// Create a provider based on the model ID
///
/// This function now returns the Phase 3 LLMProvider trait instead of the deprecated LLMClient.
pub fn make_client(api_key: String, model: ModelId) -> AnyClient {
    create_provider_for_model(model.as_str(), api_key, None)
        .unwrap_or_else(|e| panic!("Failed to create provider for model {}: {}", model.as_str(), e))
}
