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

// Adapter implementations to make Phase 3 providers compatible with LLMClient
use crate::llm::client::LLMClient;
use crate::llm::provider::{LLMProvider, LLMRequest, Message};
use crate::llm::types::{BackendKind, LLMResponse as LegacyLLMResponse};
use async_trait::async_trait;

#[async_trait]
impl LLMClient for GeminiProvider {
    async fn generate(&mut self, prompt: &str) -> Result<LegacyLLMResponse, crate::llm::provider::LLMError> {
        // Use a default model string since we can't access the private model field
        let model = "gemini-1.5-flash".to_string();

        let request = LLMRequest {
            messages: vec![Message::user(prompt.to_string())],
            system_prompt: None,
            tools: None,
            model: model.clone(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let response = LLMProvider::generate(self, request).await?;

        Ok(LegacyLLMResponse {
            content: response.content.unwrap_or_default(),
            model,
            usage: response.usage.map(|u| crate::llm::types::Usage {
                prompt_tokens: u.prompt_tokens as usize,
                completion_tokens: u.completion_tokens as usize,
                total_tokens: u.total_tokens as usize,
                cached_prompt_tokens: u.cached_prompt_tokens.map(|t| t as usize),
                cache_creation_tokens: u.cache_creation_tokens.map(|t| t as usize),
                cache_read_tokens: u.cache_read_tokens.map(|t| t as usize),
            }),
            reasoning: response.reasoning,
        })
    }

    fn backend_kind(&self) -> BackendKind {
        BackendKind::Gemini
    }

    fn model_id(&self) -> &str {
        "gemini-1.5-flash"
    }
}

#[async_trait]
impl LLMClient for OpenRouterProvider {
    async fn generate(&mut self, prompt: &str) -> Result<LegacyLLMResponse, crate::llm::provider::LLMError> {
        // Use a default model string since we can't access the private model field
        let model = "openai/gpt-4".to_string();

        let request = LLMRequest {
            messages: vec![Message::user(prompt.to_string())],
            system_prompt: None,
            tools: None,
            model: model.clone(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let response = LLMProvider::generate(self, request).await?;

        Ok(LegacyLLMResponse {
            content: response.content.unwrap_or_default(),
            model,
            usage: response.usage.map(|u| crate::llm::types::Usage {
                prompt_tokens: u.prompt_tokens as usize,
                completion_tokens: u.completion_tokens as usize,
                total_tokens: u.total_tokens as usize,
                cached_prompt_tokens: u.cached_prompt_tokens.map(|t| t as usize),
                cache_creation_tokens: u.cache_creation_tokens.map(|t| t as usize),
                cache_read_tokens: u.cache_read_tokens.map(|t| t as usize),
            }),
            reasoning: response.reasoning,
        })
    }

    fn backend_kind(&self) -> BackendKind {
        BackendKind::OpenRouter
    }

    fn model_id(&self) -> &str {
        "openai/gpt-4"
    }
}
