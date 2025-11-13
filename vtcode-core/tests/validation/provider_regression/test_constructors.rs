//! Tests for provider constructors
//!
//! These tests ensure that all provider constructors work correctly and
//! handle various configuration scenarios properly.

use vtcode_core::config::constants::models;
use vtcode_core::llm::client::LLMClient;  // For model_id() method
use vtcode_core::llm::provider::LLMProvider;  // For name() method
use vtcode_core::llm::providers::{
    AnthropicProvider, DeepSeekProvider, GeminiProvider, LmStudioProvider, MinimaxProvider,
    MoonshotProvider, OllamaProvider, OpenAIProvider, OpenRouterProvider, XAIProvider,
    ZAIProvider,
};

#[test]
fn test_anthropic_constructor_default() {
    let provider = AnthropicProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "anthropic");
}

#[test]
fn test_anthropic_constructor_with_model() {
    let provider =
        AnthropicProvider::with_model("test_key".to_string(), models::CLAUDE_SONNET_4_5.to_string());
    assert_eq!(provider.name(), "anthropic");
    assert_eq!(provider.model_id(), models::CLAUDE_SONNET_4_5);
}

#[test]
fn test_openai_constructor_default() {
    let provider = OpenAIProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "openai");
}

#[test]
fn test_openai_constructor_with_model() {
    let provider = OpenAIProvider::with_model("test_key".to_string(), "gpt-5".to_string());
    assert_eq!(provider.name(), "openai");
    assert_eq!(provider.model_id(), "gpt-5");
}

#[test]
fn test_gemini_constructor_default() {
    let provider = GeminiProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "gemini");
}

#[test]
fn test_gemini_constructor_with_model() {
    let provider =
        GeminiProvider::with_model("test_key".to_string(), "gemini-2.5-flash".to_string());
    assert_eq!(provider.name(), "gemini");
}

#[test]
fn test_openrouter_constructor_default() {
    let provider = OpenRouterProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "openrouter");
}

#[test]
fn test_openrouter_constructor_with_model() {
    let provider = OpenRouterProvider::with_model(
        "test_key".to_string(),
        models::OPENROUTER_X_AI_GROK_CODE_FAST_1.to_string(),
    );
    assert_eq!(provider.name(), "openrouter");
}

#[test]
fn test_xai_constructor_default() {
    let provider = XAIProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "xai");
}

#[test]
fn test_xai_constructor_with_model() {
    let provider =
        XAIProvider::with_model("test_key".to_string(), models::xai::GROK_4.to_string());
    assert_eq!(provider.name(), "xai");
}

#[test]
fn test_moonshot_constructor_default() {
    let provider = MoonshotProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "moonshot");
}

#[test]
fn test_moonshot_constructor_with_model() {
    let provider = MoonshotProvider::with_model(
        "test_key".to_string(),
        models::MOONSHOT_KIMI_K2_TURBO_PREVIEW.to_string(),
    );
    assert_eq!(provider.name(), "moonshot");
}

#[test]
fn test_deepseek_constructor_default() {
    let provider = DeepSeekProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "deepseek");
}

#[test]
fn test_deepseek_constructor_with_model() {
    let provider =
        DeepSeekProvider::with_model("test_key".to_string(), models::DEEPSEEK_CHAT.to_string());
    assert_eq!(provider.name(), "deepseek");
}

#[test]
fn test_zai_constructor_default() {
    let provider = ZAIProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "zai");
}

#[test]
fn test_zai_constructor_with_model() {
    let provider = ZAIProvider::with_model(
        "test_key".to_string(),
        models::zai::ZAI_CHAT_L.to_string(),
    );
    assert_eq!(provider.name(), "zai");
}

#[test]
fn test_ollama_constructor_default() {
    let provider = OllamaProvider::new(String::new());
    assert_eq!(provider.name(), "ollama");
}

#[test]
fn test_ollama_constructor_with_model() {
    let provider = OllamaProvider::with_model(String::new(), "llama2".to_string());
    assert_eq!(provider.name(), "ollama");
}

#[test]
fn test_lmstudio_constructor_default() {
    let provider = LmStudioProvider::new(String::new());
    assert_eq!(provider.name(), "lmstudio");
}

#[test]
fn test_lmstudio_constructor_with_model() {
    let provider = LmStudioProvider::with_model(
        String::new(),
        models::lmstudio::META_LLAMA_31_8B_INSTRUCT.to_string(),
    );
    assert_eq!(provider.name(), "lmstudio");
}

#[test]
fn test_minimax_constructor_default() {
    let provider = MinimaxProvider::new("test_key".to_string());
    assert_eq!(provider.name(), "minimax");
}

#[test]
fn test_minimax_constructor_with_model() {
    let provider = MinimaxProvider::with_model(
        "test_key".to_string(),
        models::minimax::MINIMAX_M2.to_string(),
    );
    assert_eq!(provider.name(), "minimax");
}

// Test provider constructor with empty API key
#[test]
fn test_constructor_with_empty_key() {
    // Local providers (Ollama, LMStudio) should handle empty keys
    let ollama = OllamaProvider::new(String::new());
    assert_eq!(ollama.name(), "ollama");

    let lmstudio = LmStudioProvider::new(String::new());
    assert_eq!(lmstudio.name(), "lmstudio");

    // Remote providers should still construct (validation happens on request)
    let anthropic = AnthropicProvider::new(String::new());
    assert_eq!(anthropic.name(), "anthropic");
}

// Test that supported models are reported correctly
#[test]
fn test_supported_models() {
    let anthropic = AnthropicProvider::new("test_key".to_string());
    let models = anthropic.supported_models();
    assert!(!models.is_empty());
    assert!(models.contains(&models::CLAUDE_SONNET_4_5.to_string()));

    let openai = OpenAIProvider::new("test_key".to_string());
    let models = openai.supported_models();
    assert!(!models.is_empty());
    assert!(models.contains(&"gpt-5".to_string()));

    let gemini = GeminiProvider::new("test_key".to_string());
    let models = gemini.supported_models();
    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.starts_with("gemini-")));
}

// Test provider names are consistent
#[test]
fn test_provider_names_consistent() {
    assert_eq!(AnthropicProvider::new("test_key".to_string()).name(), "anthropic");
    assert_eq!(OpenAIProvider::new("test_key".to_string()).name(), "openai");
    assert_eq!(GeminiProvider::new("test_key".to_string()).name(), "gemini");
    assert_eq!(OpenRouterProvider::new("test_key".to_string()).name(), "openrouter");
    assert_eq!(XAIProvider::new("test_key".to_string()).name(), "xai");
    assert_eq!(MoonshotProvider::new("test_key".to_string()).name(), "moonshot");
    assert_eq!(DeepSeekProvider::new("test_key".to_string()).name(), "deepseek");
    assert_eq!(ZAIProvider::new("test_key".to_string()).name(), "zai");
    assert_eq!(OllamaProvider::new(String::new()).name(), "ollama");
    assert_eq!(LmStudioProvider::new(String::new()).name(), "lmstudio");
    assert_eq!(MinimaxProvider::new("test_key".to_string()).name(), "minimax");
}
