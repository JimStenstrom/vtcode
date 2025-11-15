// Local provider modules (not yet extracted to standalone crates)
pub mod anthropic;
pub mod deepseek;
pub mod lmstudio;
pub mod microsoft;
pub mod minimax;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod xai;
pub mod zai;

mod codex_prompt;
mod common;
mod reasoning;
mod shared;

#[cfg(test)]
pub(crate) mod test_utils;

pub(crate) use codex_prompt::gpt5_codex_developer_prompt;
pub(crate) use reasoning::{ReasoningBuffer, extract_reasoning_trace, split_reasoning_from_text};

// Re-export local providers
pub use anthropic::AnthropicProvider;
pub use deepseek::DeepSeekProvider;
pub use lmstudio::LmStudioProvider;
pub use microsoft::MicrosoftProvider;
pub use minimax::MinimaxProvider;
pub use moonshot::MoonshotProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use xai::XAIProvider;
pub use zai::ZAIProvider;

// Re-export providers from standalone crates (Phase 3 complete)
pub use vtcode_llm_gemini::GeminiProvider;
pub use vtcode_llm_openrouter::OpenRouterProvider;

// Gemini-specific utilities module
pub mod gemini {
    pub use vtcode_llm_gemini::sanitize_function_parameters;
}
