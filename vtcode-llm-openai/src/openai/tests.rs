//! OpenAI provider tests

use super::provider::OpenAIProvider;
use vtcode_llm_types::LLMProvider;

#[test]
fn test_provider_creation() {
    let provider = OpenAIProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "openai");
    assert!(provider.supports_streaming());
}

#[test]
fn test_model_capabilities() {
    let provider = OpenAIProvider::new("test_key".to_string());

    assert!(provider.supports_reasoning("o1-preview"));
    assert!(!provider.supports_reasoning("gpt-4"));

    assert!(provider.supports_tools("gpt-4"));
    assert!(!provider.supports_tools("o1-preview"));
}

#[test]
fn test_context_sizes() {
    let provider = OpenAIProvider::new("test_key".to_string());

    assert_eq!(provider.effective_context_size("gpt-4o"), 128_000);
    assert_eq!(provider.effective_context_size("gpt-4"), 8_192);
    assert_eq!(provider.effective_context_size("gpt-3.5-turbo"), 4_096);
}
