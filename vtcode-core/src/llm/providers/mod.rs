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

pub(crate) use codex_prompt::gpt5_codex_developer_prompt;
pub(crate) use reasoning::{ReasoningBuffer, extract_reasoning_trace, split_reasoning_from_text};

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

// Re-export GeminiProvider from the modularized crate
pub use vtcode_llm_gemini::GeminiProvider;

// Re-export OpenRouter provider from the modularized crate
pub use vtcode_llm_openrouter::OpenRouterProvider;
