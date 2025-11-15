//! OpenAI LLM provider implementation

use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::json;
use std::time::Duration;
use tracing::debug;
use vtcode_llm_types::{
    LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream, ToolChoice,
};

use super::config::{DEFAULT_MODEL, DEFAULT_TIMEOUT_SECS, OPENAI_API_BASE};
use super::request::{serialize_messages, serialize_tools};
use super::response::parse_response;
use super::streaming::stream_response;

/// OpenAI LLM provider
pub struct OpenAIProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with default model
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, DEFAULT_MODEL.to_string())
    }

    /// Create a new OpenAI provider with specified model
    pub fn with_model(api_key: String, model: String) -> Self {
        Self {
            api_key,
            http_client: HttpClient::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| OPENAI_API_BASE.to_string()),
            model,
        }
    }

    /// Create from configuration
    pub fn from_config<T>(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        _prompt_cache: Option<T>, // Generic placeholder for API compatibility
    ) -> Self {
        let api_key_value = api_key.unwrap_or_default();
        let model_value = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let base_url_value = base_url
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| OPENAI_API_BASE.to_string());

        Self {
            api_key: api_key_value,
            http_client: HttpClient::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
            base_url: base_url_value,
            model: model_value,
        }
    }

    async fn generate_internal(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let messages = serialize_messages(&request.messages, request.system_prompt.as_ref());

        let mut body = json!({
            "model": request.model,
            "messages": messages,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(tools) = &request.tools
            && let Some(tools_json) = serialize_tools(tools) {
                body["tools"] = json!(tools_json);

                if let Some(tool_choice) = &request.tool_choice {
                    body["tool_choice"] = match tool_choice {
                        ToolChoice::Auto => json!("auto"),
                        ToolChoice::None => json!("none"),
                        ToolChoice::Any => json!("required"),
                        ToolChoice::Specific(s) => json!(s),
                    };
                }

                if let Some(parallel_tool_calls) = request.parallel_tool_calls {
                    body["parallel_tool_calls"] = json!(parallel_tool_calls);
                }
            }

        let url = format!("{}/chat/completions", self.base_url);

        debug!(target: "vtcode_llm_openai", "Sending request to {}", url);

        let builder = self.http_client.post(&url);
        let builder = if self.api_key.trim().is_empty() {
            builder
        } else {
            builder.bearer_auth(&self.api_key)
        };

        let response = builder
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    LLMError::AuthenticationError("Invalid API key".to_string())
                } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    LLMError::RateLimit
                } else {
                    LLMError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError(format!("API error ({}): {}", status, error_text)));
        }

        let response_json = response
            .json()
            .await
            .map_err(|e| LLMError::Provider(format!("Failed to parse JSON response: {}", e)))?;
        parse_response(response_json)
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_reasoning(&self, model: &str) -> bool {
        model.contains("o1") || model.contains("o3")
    }

    fn supports_tools(&self, model: &str) -> bool {
        // Most OpenAI models support tools except o1-preview and o1-mini
        !model.contains("o1-preview") && !model.contains("o1-mini")
    }

    fn supports_context_caching(&self, model: &str) -> bool {
        // OpenAI supports prompt caching for certain models
        model.starts_with("gpt-4") || model.starts_with("gpt-3.5")
    }

    fn effective_context_size(&self, model: &str) -> usize {
        if model.contains("gpt-4o") || model.contains("gpt-4-turbo") {
            128_000
        } else if model.contains("gpt-4") {
            8_192
        } else if model.contains("gpt-3.5-turbo-16k") {
            16_385
        } else if model.contains("gpt-3.5") {
            4_096
        } else {
            128_000 // Default for newer models
        }
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        self.validate_request(&request)?;
        self.generate_internal(request).await
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError> {
        self.validate_request(&request)?;
        stream_response(&self.http_client, &self.base_url, &self.api_key, request).await
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
            "o1-preview".to_string(),
            "o1-mini".to_string(),
        ]
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        if request.messages.is_empty() {
            return Err(LLMError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }

        // Validate tool use for models that don't support it
        if request.tools.is_some() && !self.supports_tools(&request.model) {
            return Err(LLMError::InvalidRequest(format!(
                "Model {} does not support tool calling",
                request.model
            )));
        }

        Ok(())
    }
}
