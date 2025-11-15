// Local provider modules (DirectLine not fully extracted due to config dependencies)
pub mod microsoft;

// Internal utility modules
mod codex_prompt;
mod common;
mod reasoning;
mod shared;

#[cfg(test)]
pub(crate) mod test_utils;

pub(crate) use codex_prompt::gpt5_codex_developer_prompt;
pub(crate) use reasoning::{ReasoningBuffer, extract_reasoning_trace, split_reasoning_from_text};

// Re-export local provider (DirectLine)
pub use microsoft::MicrosoftProvider;

// Re-export all providers from standalone crates (Phase 8 complete)
pub use vtcode_llm_anthropic::AnthropicProvider;
pub use vtcode_llm_deepseek::DeepSeekProvider;
pub use vtcode_llm_gemini::GeminiProvider;
pub use vtcode_llm_lmstudio::{LmStudioProvider, fetch_lmstudio_models};
pub use vtcode_llm_minimax::MinimaxProvider;
pub use vtcode_llm_moonshot::MoonshotProvider;
pub use vtcode_llm_ollama::{OllamaProvider, fetch_ollama_models};
pub use vtcode_llm_openai::OpenAIProvider;
pub use vtcode_llm_openrouter::OpenRouterProvider;
pub use vtcode_llm_xai::XAIProvider;
pub use vtcode_llm_zai::ZAIProvider;

// Gemini-specific utilities module
pub mod gemini {
    pub use vtcode_llm_gemini::sanitize_function_parameters;
}
