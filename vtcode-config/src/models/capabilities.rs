//! Model capability detection

use super::Provider;
use crate::constants::models;

/// Check if model supports reasoning effort parameter
pub fn supports_reasoning_effort(provider: &Provider, model: &str) -> bool {
    match provider {
        Provider::Gemini => model == models::google::GEMINI_2_5_PRO,
        Provider::OpenAI => models::openai::REASONING_MODELS.contains(&model),
        Provider::Anthropic => models::anthropic::REASONING_MODELS.contains(&model),
        Provider::DeepSeek => model == models::deepseek::DEEPSEEK_REASONER,
        Provider::OpenRouter => {
            // OpenRouter reasoning support checked via metadata in ModelId
            models::openrouter::REASONING_MODELS.contains(&model)
                || model.contains("-reasoning")
                || model.contains("-extended")
        }
        Provider::Ollama => false,
        Provider::LmStudio => false,
        Provider::Moonshot => model == models::moonshot::KIMI_K2_THINKING,
        Provider::XAI => {
            model == models::xai::GROK_4 || model == models::xai::GROK_4_CODE
        }
        Provider::ZAI => model == models::zai::GLM_4_6,
        Provider::Microsoft => false,
    }
}

/// Check if model supports context caching
pub fn supports_context_caching(provider: &Provider, model: &str) -> bool {
    match provider {
        Provider::Gemini => models::google::CACHING_MODELS.contains(&model),
        Provider::Anthropic => {
            // Anthropic caching support based on model constants
            matches!(
                model,
                "claude-opus-4-1-20250805"
                    | "claude-sonnet-4-5"
                    | "claude-haiku-4-5"
                    | "claude-sonnet-4-20250514"
            )
        }
        _ => false,
    }
}

/// Check if model supports tool calling
pub fn supports_tools(provider: &Provider, model: &str) -> bool {
    match provider {
        Provider::OpenAI => !models::openai::TOOL_UNAVAILABLE_MODELS.contains(&model),
        Provider::Anthropic => true,
        Provider::Gemini => true,
        Provider::OpenRouter => true,
        Provider::DeepSeek => true,
        Provider::Moonshot => true,
        Provider::XAI => true,
        Provider::ZAI => true,
        Provider::Microsoft => true,
        Provider::Ollama => false, // Most Ollama models don't support tools
        Provider::LmStudio => false,
    }
}

/// Check if model supports code execution
pub fn supports_code_execution(provider: &Provider, model: &str) -> bool {
    match provider {
        Provider::Gemini => models::google::CODE_EXECUTION_MODELS.contains(&model),
        _ => false,
    }
}
