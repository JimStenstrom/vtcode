//! Gemini LLM Provider implementation
//!
//! This module provides the main GeminiProvider struct and implements the
//! LLMProvider trait for Gemini API integration.

use vtcode_config::constants::{env_vars, models, urls};
use vtcode_config::core::{GeminiPromptCacheMode, GeminiPromptCacheSettings, PromptCachingConfig};
use crate::gemini::streaming::{StreamingCandidate, StreamingError, StreamingProcessor};
use crate::gemini::{Content, GenerateContentResponse, Part};
use vtcode_llm_types::{
    LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream,
    LLMStreamEvent,
};
use async_stream::try_stream;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use tokio::sync::mpsc;

// Import from our new modules
use crate::config;
use crate::error::{handle_http_error, map_streaming_error};
use crate::request;
use crate::response;

/// Gemini LLM Provider
pub struct GeminiProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_enabled: bool,
    prompt_cache_settings: GeminiPromptCacheSettings,
}

impl GeminiProvider {
    /// Create a new Gemini provider with the default model
    pub fn new(api_key: String) -> Self {
        Self::with_model(
            api_key,
            vtcode_config::constants::models::google::DEFAULT_MODEL.to_string(),
        )
    }

    /// Create a new Gemini provider with a specific model
    pub fn with_model(api_key: String, model: String) -> Self {
        Self::with_model_full(api_key, model, None, None)
    }

    /// Create a new Gemini provider from configuration
    pub fn from_config(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        prompt_cache: Option<PromptCachingConfig>,
    ) -> Self {
        let api_key_value = api_key.unwrap_or_else(|| {
            std::env::var("GEMINI_API_KEY")
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .unwrap_or_default()
        });

        let model_value = model.unwrap_or_else(|| {
            vtcode_config::constants::models::google::DEFAULT_MODEL.to_string()
        });

        Self::with_model_full(api_key_value, model_value, prompt_cache, base_url)
    }

    /// Create a new Gemini provider with full configuration
    fn with_model_full(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        // Determine base URL
        let base_url = base_url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var(env_vars::GEMINI_BASE_URL).ok())
            .unwrap_or_else(|| urls::GEMINI_API_BASE.to_string());

        // Extract prompt cache settings
        let (prompt_cache_enabled, prompt_cache_settings) = if let Some(cfg) = prompt_cache {
            let settings = cfg.providers.gemini.clone();
            let enabled =
                cfg.enabled && settings.enabled && settings.mode != GeminiPromptCacheMode::Off;
            (enabled, settings)
        } else {
            (false, GeminiPromptCacheSettings::default())
        };

        Self {
            api_key,
            http_client: HttpClient::new(),
            base_url,
            model,
            prompt_cache_enabled,
            prompt_cache_settings,
        }
    }

    /// Check if model supports context caching
    pub fn supports_caching(model: &str) -> bool {
        config::supports_caching(model)
    }

    /// Check if model supports code execution
    pub fn supports_code_execution(model: &str) -> bool {
        config::supports_code_execution(model)
    }

    /// Get maximum input token limit for a model
    pub fn max_input_tokens(model: &str) -> usize {
        config::max_input_tokens(model)
    }

    /// Get maximum output token limit for a model
    pub fn max_output_tokens(model: &str) -> usize {
        config::max_output_tokens(model)
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_reasoning(&self, model: &str) -> bool {
        // Gemini 2.5 models support thinking/reasoning capability
        // Reference: https://ai.google.dev/gemini-api/docs/models
        models::google::REASONING_MODELS.contains(&model)
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let gemini_request = request::convert_to_gemini_request(
            &request,
            self.prompt_cache_enabled,
            &self.prompt_cache_settings,
        )?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let response = self
            .http_client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| {
                let formatted_error = format!("Gemini: Network error: {}", e);
                LLMError::NetworkError(formatted_error)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(handle_http_error(status.as_u16(), error_text));
        }

        let gemini_response: GenerateContentResponse = response.json().await.map_err(|e| {
            let formatted_error = format!("Gemini: Failed to parse response: {}", e);
            LLMError::Provider(formatted_error)
        })?;

        response::convert_from_gemini_response(gemini_response)
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError> {
        let gemini_request = request::convert_to_gemini_request(
            &request,
            self.prompt_cache_enabled,
            &self.prompt_cache_settings,
        )?;

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let response = self
            .http_client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| {
                let formatted_error = format!("Gemini: Network error: {}", e);
                LLMError::NetworkError(formatted_error)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(handle_http_error(status.as_u16(), error_text));
        }

        let (event_tx, event_rx) = mpsc::unbounded_channel::<Result<LLMStreamEvent, LLMError>>();
        let completion_sender = event_tx.clone();

        tokio::spawn(async move {
            let mut processor = StreamingProcessor::new();
            let token_sender = completion_sender.clone();
            let mut aggregated_text = String::new();
            let mut on_chunk = |chunk: &str| -> Result<(), StreamingError> {
                if chunk.is_empty() {
                    return Ok(());
                }

                if let Some(delta) = response::apply_stream_delta(&mut aggregated_text, chunk)
                    && !delta.is_empty() {
                        token_sender
                            .send(Ok(LLMStreamEvent::Token { delta }))
                            .map_err(|_| StreamingError::StreamingError {
                                message: "Streaming consumer dropped".to_string(),
                                partial_content: Some(chunk.to_string()),
                            })?;
                    }
                Ok(())
            };

            let result = processor.process_stream(response, &mut on_chunk).await;
            match result {
                Ok(mut streaming_response) => {
                    if streaming_response.candidates.is_empty()
                        && !aggregated_text.trim().is_empty()
                    {
                        streaming_response.candidates.push(StreamingCandidate {
                            content: Content {
                                role: "model".to_string(),
                                parts: vec![Part::Text {
                                    text: aggregated_text.clone(),
                                }],
                            },
                            finish_reason: None,
                            index: Some(0),
                        });
                    }

                    match response::convert_from_streaming_response(streaming_response) {
                        Ok(final_response) => {
                            let _ = completion_sender.send(Ok(LLMStreamEvent::Completed {
                                response: final_response,
                            }));
                        }
                        Err(err) => {
                            let _ = completion_sender.send(Err(err));
                        }
                    }
                }
                Err(error) => {
                    let mapped = map_streaming_error(error);
                    let _ = completion_sender.send(Err(mapped));
                }
            }
        });

        drop(event_tx);

        let stream = {
            let mut receiver = event_rx;
            try_stream! {
                while let Some(event) = receiver.recv().await {
                    yield event?;
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<String> {
        // Order: stable models first, then preview/experimental
        models::google::SUPPORTED_MODELS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        if !self.supported_models().contains(&request.model) {
            let formatted_error = format!("Gemini: Unsupported model: {}", request.model);
            return Err(LLMError::InvalidRequest(formatted_error));
        }

        // Validate token limits based on model capabilities
        if let Some(max_tokens) = request.max_tokens {
            let model = request.model.as_str();
            let max_output_tokens = config::max_output_tokens(model);

            if (max_tokens as usize) > max_output_tokens {
                let formatted_error = format!(
                    "Gemini: Requested max_tokens ({}) exceeds model limit ({}) for {}",
                    max_tokens, max_output_tokens, model
                );
                return Err(LLMError::InvalidRequest(formatted_error));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtcode_config::constants::models;

    #[test]
    fn supported_models_returns_non_empty_list() {
        let provider = GeminiProvider::new("test_key".to_string());
        let models = provider.supported_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn constructor_new_uses_default_model() {
        let provider = GeminiProvider::new("test_key".to_string());
        assert_eq!(provider.model, models::google::DEFAULT_MODEL);
    }

    #[test]
    fn constructor_with_model_uses_custom_model() {
        let custom_model = "gemini-1.5-pro";
        let provider = GeminiProvider::with_model("test_key".to_string(), custom_model.to_string());
        assert_eq!(provider.model, custom_model);
    }

    #[test]
    fn constructor_from_config_uses_provided_values() {
        let provider = GeminiProvider::from_config(
            Some("custom_key".to_string()),
            Some("custom_model".to_string()),
            None,
            None,
        );
        assert_eq!(provider.api_key, "custom_key");
        assert_eq!(provider.model, "custom_model");
    }

    #[test]
    fn name_returns_gemini() {
        let provider = GeminiProvider::new("test_key".to_string());
        assert_eq!(provider.name(), "gemini");
    }

    #[test]
    fn supports_streaming_returns_true() {
        let provider = GeminiProvider::new("test_key".to_string());
        assert!(provider.supports_streaming());
    }

    #[test]
    fn supports_caching_delegates_to_config() {
        assert!(GeminiProvider::supports_caching("gemini-2.5-flash"));
    }

    #[test]
    fn max_tokens_delegates_to_config() {
        assert_eq!(GeminiProvider::max_output_tokens("gemini-2.5-flash"), 65_536);
        assert_eq!(GeminiProvider::max_input_tokens("gemini-2.5-flash"), 1_048_576);
    }
}
