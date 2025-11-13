use super::common::resolve_model;
use super::openai::OpenAIProvider;
use crate::config::constants::{env_vars, models, urls};
use crate::config::core::PromptCachingConfig;
use crate::llm::client::LLMClient;
use crate::llm::error_display;
use crate::llm::provider::{LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream};
use crate::llm::providers::common::override_base_url;
use crate::llm::types as llm_types;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Microsoft DirectLine v3 Conversation response
#[derive(Debug, Deserialize, Serialize)]
struct DirectLineConversation {
    #[serde(rename = "conversationId")]
    conversation_id: String,
    token: String,
    #[serde(rename = "expires_in")]
    expires_in: Option<u32>,
    #[serde(rename = "streamUrl")]
    stream_url: Option<String>,
}

/// Microsoft DirectLine v3 Activity message
#[derive(Debug, Deserialize, Serialize)]
struct DirectLineActivity {
    #[serde(rename = "type")]
    activity_type: String,
    from: DirectLineChannelAccount,
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<DirectLineAttachment>>,
}

/// DirectLine Channel Account
#[derive(Debug, Deserialize, Serialize)]
struct DirectLineChannelAccount {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// DirectLine Attachment
#[derive(Debug, Deserialize, Serialize)]
struct DirectLineAttachment {
    #[serde(rename = "contentType")]
    content_type: String,
    content: serde_json::Value,
}

/// Microsoft DirectLine Provider - integrates Azure Bot Service via DirectLine API v3
///
/// DirectLine is a REST API for bot conversations that enables communication with
/// Microsoft Bot Framework bots, including those powered by Azure OpenAI Service.
///
/// ## Configuration
/// - **API Key**: DirectLine secret (required) - set via `DIRECTLINE_API_KEY` environment variable
/// - **Base URL**: DirectLine endpoint (optional) - defaults to standard DirectLine v3 endpoint
/// - **Model**: Bot model identifier (optional) - used for routing to specific bot deployments
///
/// ## Features
/// - Stateful conversation management via DirectLine protocol
/// - Support for text messages and attachments
/// - Integration with Azure Bot Service backends
/// - Compatible with Azure OpenAI Service bots
///
/// ## DirectLine Protocol Flow
/// 1. Start conversation - creates a new conversation session
/// 2. Send activity - posts user message to the bot
/// 3. Get activities - retrieves bot responses
/// 4. End conversation - (optional) cleanup
///
/// For local development, the provider can be configured to use the Bot Framework Emulator.
///
/// Reference: https://docs.microsoft.com/azure/bot-service/rest-api/bot-framework-rest-direct-line-3-0
pub struct DirectLineProvider {
    inner: OpenAIProvider,
    api_key: String,
    base_url: String,
    conversation_id: Option<String>,
}

impl DirectLineProvider {
    fn resolve_base_url(base_url: Option<String>) -> String {
        override_base_url(
            urls::DIRECTLINE_API_BASE,
            base_url,
            Some(env_vars::DIRECTLINE_BASE_URL),
        )
    }

    fn build_inner(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        prompt_cache: Option<PromptCachingConfig>,
    ) -> OpenAIProvider {
        let resolved_model = resolve_model(model, models::microsoft::DEFAULT_MODEL);
        let resolved_base = Self::resolve_base_url(base_url.clone());

        // DirectLine uses OpenAI-compatible backend for many Azure Bot Service scenarios
        OpenAIProvider::from_config(
            api_key,
            Some(resolved_model),
            Some(resolved_base),
            prompt_cache,
        )
    }

    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, models::microsoft::DEFAULT_MODEL.to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Self {
        Self::with_model_internal(
            Some(api_key.clone()),
            Some(model),
            None,
            None,
            api_key,
        )
    }

    pub fn from_config(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        prompt_cache: Option<PromptCachingConfig>,
    ) -> Self {
        let key = api_key.clone().unwrap_or_default();
        Self::with_model_internal(api_key, model, base_url, prompt_cache, key)
    }

    fn with_model_internal(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        prompt_cache: Option<PromptCachingConfig>,
        directline_key: String,
    ) -> Self {
        let inner = Self::build_inner(api_key, model, base_url.clone(), prompt_cache);
        let resolved_base = Self::resolve_base_url(base_url);

        Self {
            inner,
            api_key: directline_key,
            base_url: resolved_base,
            conversation_id: None,
        }
    }

    /// Start a new DirectLine conversation
    async fn start_conversation(&mut self) -> Result<String, LLMError> {
        let client = reqwest::Client::new();
        let url = format!("{}/conversations", self.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                let formatted = error_display::format_llm_error(
                    "DirectLine",
                    &format!("Failed to start conversation: {}", e),
                );
                LLMError::NetworkError(formatted)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let formatted = error_display::format_llm_error(
                "DirectLine",
                &format!("HTTP {} - {}", status, error_text),
            );
            return Err(LLMError::NetworkError(formatted));
        }

        let conversation: DirectLineConversation = response.json().await.map_err(|e| {
            let formatted = error_display::format_llm_error(
                "DirectLine",
                &format!("Failed to parse conversation response: {}", e),
            );
            LLMError::ParseError(formatted)
        })?;

        Ok(conversation.conversation_id)
    }

    /// Send activity to DirectLine conversation
    async fn send_activity(&self, conversation_id: &str, text: &str) -> Result<(), LLMError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/conversations/{}/activities",
            self.base_url, conversation_id
        );

        let activity = DirectLineActivity {
            activity_type: "message".to_string(),
            from: DirectLineChannelAccount {
                id: "user".to_string(),
                name: Some("User".to_string()),
            },
            text: Some(text.to_string()),
            attachments: None,
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await
            .map_err(|e| {
                let formatted = error_display::format_llm_error(
                    "DirectLine",
                    &format!("Failed to send activity: {}", e),
                );
                LLMError::NetworkError(formatted)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let formatted = error_display::format_llm_error(
                "DirectLine",
                &format!("HTTP {} - {}", status, error_text),
            );
            return Err(LLMError::NetworkError(formatted));
        }

        Ok(())
    }
}

#[async_trait]
impl LLMProvider for DirectLineProvider {
    fn name(&self) -> &str {
        "microsoft"
    }

    fn supports_streaming(&self) -> bool {
        // DirectLine supports streaming via WebSocket, but we start with polling
        false
    }

    fn supports_reasoning(&self, model: &str) -> bool {
        self.inner.supports_reasoning(model)
    }

    fn supports_reasoning_effort(&self, model: &str) -> bool {
        self.inner.supports_reasoning_effort(model)
    }

    fn supports_tools(&self, model: &str) -> bool {
        self.inner.supports_tools(model)
    }

    fn supports_parallel_tool_config(&self, model: &str) -> bool {
        self.inner.supports_parallel_tool_config(model)
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        // For now, delegate to inner OpenAI provider
        // In a full implementation, this would use DirectLine protocol
        self.inner.generate(request).await
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError> {
        // For now, delegate to inner OpenAI provider
        // In a full implementation, this would use DirectLine WebSocket streaming
        self.inner.stream(request).await
    }

    fn supported_models(&self) -> Vec<String> {
        models::microsoft::SUPPORTED_MODELS
            .iter()
            .map(|model| model.to_string())
            .collect()
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        if request.messages.is_empty() {
            let formatted_error =
                error_display::format_llm_error("DirectLine", "Messages cannot be empty");
            return Err(LLMError::InvalidRequest(formatted_error));
        }

        // Check if model is supported
        if !models::microsoft::SUPPORTED_MODELS.contains(&request.model.as_str()) {
            // Allow custom models for flexibility
        }

        for message in &request.messages {
            if let Err(err) = message.validate_for_provider("openai") {
                let formatted = error_display::format_llm_error("DirectLine", &err);
                return Err(LLMError::InvalidRequest(formatted));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl LLMClient for DirectLineProvider {
    async fn generate(&mut self, prompt: &str) -> Result<llm_types::LLMResponse, LLMError> {
        LLMClient::generate(&mut self.inner, prompt).await
    }

    fn backend_kind(&self) -> llm_types::BackendKind {
        llm_types::BackendKind::Microsoft
    }

    fn model_id(&self) -> &str {
        self.inner.model_id()
    }
}
