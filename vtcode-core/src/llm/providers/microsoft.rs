use crate::config::constants::{env_vars, models, urls};
use crate::config::core::PromptCachingConfig;
use crate::llm::client::LLMClient;
use crate::llm::error_display;
use crate::llm::provider::{
    ContentPart, FinishReason, LLMError, LLMProvider, LLMRequest, LLMResponse, Message,
    MessageContent, MessageRole,
};
use crate::llm::types as llm_types;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
#[cfg(debug_assertions)]
use std::time::Instant;
#[cfg(debug_assertions)]
use tracing::debug;

use super::common::override_base_url;

/// Microsoft Direct Line v3 provider for Bot Framework and M365 Copilot
pub struct MicrosoftProvider {
    secret: String,
    base_url: String,
    model: String,
    http_client: HttpClient,
}

#[derive(Debug, Deserialize)]
struct ConversationResponse {
    #[serde(rename = "conversationId")]
    conversation_id: String,
    token: Option<String>,
    #[serde(rename = "expiresIn")]
    expires_in: Option<i32>,
}

#[derive(Debug, Serialize)]
struct Activity {
    #[serde(rename = "type")]
    activity_type: String,
    from: ActivityParticipant,
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActivityParticipant {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ActivitiesResponse {
    activities: Vec<BotActivity>,
    watermark: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BotActivity {
    #[serde(rename = "type")]
    activity_type: String,
    #[serde(default)]
    from: Option<ActivityParticipant>,
    text: Option<String>,
    value: Option<Value>,
    #[serde(default)]
    attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize)]
struct Attachment {
    #[serde(rename = "contentType")]
    content_type: String,
    content: Option<Value>,
}

impl MicrosoftProvider {
    /// Create a new Microsoft provider from configuration
    pub fn from_config(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        prompt_cache: Option<PromptCachingConfig>,
    ) -> Self {
        let api_key = api_key.unwrap_or_default();
        let model = model.unwrap_or_else(|| models::microsoft::DEFAULT_MODEL.to_string());
        Self::with_model_internal(api_key, model, prompt_cache, base_url)
    }

    fn with_model_internal(
        api_key: String,
        model: String,
        _prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        let resolved_base_url = override_base_url(
            urls::MICROSOFT_DIRECTLINE_API_BASE,
            base_url,
            Some(env_vars::MICROSOFT_DIRECTLINE_BASE_URL),
        );

        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Self {
            secret: api_key,
            base_url: resolved_base_url,
            model,
            http_client,
        }
    }

    async fn start_conversation(&self) -> Result<ConversationResponse, LLMError> {
        let url = format!("{}/conversations", self.base_url);

        #[cfg(debug_assertions)]
        let start_time = Instant::now();

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            "Starting Direct Line conversation"
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Failed to start conversation: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let formatted =
                error_display::format_llm_error("Microsoft", &format!("{}: {}", status, error_text));
            return Err(LLMError::Provider(formatted));
        }

        let conversation = response.json::<ConversationResponse>().await.map_err(|e| {
            LLMError::Provider(error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to parse conversation response: {}", e),
            ))
        })?;

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation.conversation_id,
            elapsed_ms = start_time.elapsed().as_millis(),
            "Conversation started successfully"
        );

        Ok(conversation)
    }

    async fn send_activity(
        &self,
        conversation_id: &str,
        activity: Activity,
    ) -> Result<(), LLMError> {
        let url = format!(
            "{}/conversations/{}/activities",
            self.base_url, conversation_id
        );

        #[cfg(debug_assertions)]
        let text_len = activity.text.as_ref().map_or(0, |t| t.len());

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation_id,
            activity_type = %activity.activity_type,
            text_len = text_len,
            "Sending activity to bot"
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Failed to send activity: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let formatted = error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to send activity {}: {}", status, error_text),
            );
            return Err(LLMError::Provider(formatted));
        }

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation_id,
            "Activity sent successfully"
        );

        Ok(())
    }

    async fn get_activities(
        &self,
        conversation_id: &str,
        watermark: Option<&str>,
    ) -> Result<ActivitiesResponse, LLMError> {
        let mut url = format!(
            "{}/conversations/{}/activities",
            self.base_url, conversation_id
        );

        if let Some(wm) = watermark {
            url = format!("{}?watermark={}", url, wm);
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Failed to get activities: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let formatted = error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to get activities {}: {}", status, error_text),
            );
            return Err(LLMError::Provider(formatted));
        }

        let activities_response = response.json::<ActivitiesResponse>().await.map_err(|e| {
            LLMError::Provider(error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to parse activities response: {}", e),
            ))
        })?;

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation_id,
            activity_count = activities_response.activities.len(),
            has_watermark = activities_response.watermark.is_some(),
            "Retrieved activities from conversation"
        );

        Ok(activities_response)
    }

    fn convert_messages_to_text(&self, messages: &[Message]) -> String {
        let mut combined = String::new();

        for message in messages {
            match &message.content {
                MessageContent::Text(text) => {
                    combined.push_str(text);
                    combined.push('\n');
                }
                MessageContent::Parts(parts) => {
                    for part in parts {
                        if let ContentPart::Text { text } = part {
                            combined.push_str(text);
                            combined.push('\n');
                        }
                    }
                }
            }
        }

        combined.trim().to_string()
    }

    /// Extract content from activity, including Adaptive Cards
    fn extract_activity_content(&self, activity: &BotActivity) -> Option<String> {
        // First, check for simple text response
        if let Some(text) = &activity.text {
            if !text.trim().is_empty() {
                return Some(text.clone());
            }
        }

        // Check for Adaptive Card attachments
        for attachment in &activity.attachments {
            if attachment.content_type == "application/vnd.microsoft.card.adaptive" {
                if let Some(card_content) = &attachment.content {
                    // Extract text from Adaptive Card body
                    if let Some(body) = card_content.get("body").and_then(|b| b.as_array()) {
                        let mut card_text = String::new();
                        for element in body {
                            if let Some(text) = element.get("text").and_then(|t| t.as_str()) {
                                if !card_text.is_empty() {
                                    card_text.push('\n');
                                }
                                card_text.push_str(text);
                            }
                        }
                        if !card_text.is_empty() {
                            return Some(card_text);
                        }
                    }
                }
            }
        }

        // Check for value field (can contain structured data)
        if let Some(value) = &activity.value {
            if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
        }

        None
    }
}

#[async_trait]
impl LLMProvider for MicrosoftProvider {
    fn name(&self) -> &str {
        "microsoft"
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn supports_reasoning(&self, _model: &str) -> bool {
        false
    }

    fn supports_tools(&self, _model: &str) -> bool {
        // Direct Line supports Adaptive Cards and rich content, but not OpenAI-style tool calling
        false
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        #[cfg(debug_assertions)]
        let request_timer = Instant::now();

        #[cfg(debug_assertions)]
        {
            let message_count = request.messages.len();
            let has_system = request.system_prompt.is_some();
            debug!(
                target = "vtcode::llm::microsoft",
                model = %request.model,
                message_count = message_count,
                has_system_prompt = has_system,
                "Starting Microsoft Direct Line request"
            );
        }

        // Start a conversation
        let conversation = self.start_conversation().await?;

        // Convert messages to a single text payload
        let text = self.convert_messages_to_text(&request.messages);

        // Add system prompt if present
        let final_text = if let Some(system_prompt) = &request.system_prompt {
            format!("{}\n\n{}", system_prompt, text)
        } else {
            text
        };

        // Send user message
        let activity = Activity {
            activity_type: "message".to_string(),
            from: ActivityParticipant {
                id: "user".to_string(),
            },
            text: Some(final_text),
            value: None,
        };

        self.send_activity(&conversation.conversation_id, activity)
            .await?;

        // Poll for bot response with timeout
        let max_attempts = 30; // 30 seconds max wait
        let mut watermark: Option<String> = None;
        let mut poll_count = 0;

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation.conversation_id,
            max_attempts = max_attempts,
            "Starting polling for bot response"
        );

        for _ in 0..max_attempts {
            tokio::time::sleep(Duration::from_secs(1)).await;
            poll_count += 1;

            let activities = self
                .get_activities(&conversation.conversation_id, watermark.as_deref())
                .await?;

            // Update watermark
            watermark = activities.watermark;

            // Look for bot responses
            for activity in activities.activities {
                if activity.activity_type == "message" {
                    // Check if this is from the bot (not from us)
                    let is_from_bot = activity
                        .from
                        .as_ref()
                        .map(|f| f.id != "user")
                        .unwrap_or(true);

                    if is_from_bot {
                        if let Some(content) = self.extract_activity_content(&activity) {
                            #[cfg(debug_assertions)]
                            {
                                let content_len = content.len();
                                let has_attachments = !activity.attachments.is_empty();
                                let has_adaptive_card = activity.attachments.iter().any(|a|
                                    a.content_type == "application/vnd.microsoft.card.adaptive"
                                );
                                debug!(
                                    target = "vtcode::llm::microsoft",
                                    model = %request.model,
                                    conversation_id = %conversation.conversation_id,
                                    elapsed_ms = request_timer.elapsed().as_millis(),
                                    poll_attempts = poll_count,
                                    content_len = content_len,
                                    has_attachments = has_attachments,
                                    has_adaptive_card = has_adaptive_card,
                                    "Completed Microsoft Direct Line request"
                                );
                            }

                            return Ok(LLMResponse {
                                content: Some(content),
                                tool_calls: None,
                                usage: None,
                                finish_reason: FinishReason::Stop,
                                reasoning: None,
                                reasoning_details: None,
                            });
                        }
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        debug!(
            target = "vtcode::llm::microsoft",
            conversation_id = %conversation.conversation_id,
            elapsed_ms = request_timer.elapsed().as_millis(),
            poll_attempts = poll_count,
            "Timeout waiting for bot response"
        );

        Err(LLMError::Provider(error_display::format_llm_error(
            "Microsoft",
            "Timeout waiting for bot response",
        )))
    }

    async fn stream(&self, request: LLMRequest) -> Result<crate::llm::provider::LLMStream, LLMError> {
        // Microsoft DirectLine doesn't support native streaming in this implementation, fall back to non-streaming
        use async_stream::try_stream;
        use crate::llm::provider::LLMStreamEvent;

        let response = self.generate(request).await?;
        let stream = try_stream! {
            yield LLMStreamEvent::Completed { response };
        };
        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<String> {
        models::microsoft::SUPPORTED_MODELS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        if request.messages.is_empty() {
            let formatted =
                error_display::format_llm_error("Microsoft", "Messages cannot be empty");
            return Err(LLMError::InvalidRequest(formatted));
        }

        if self.secret.is_empty() {
            let formatted = error_display::format_llm_error(
                "Microsoft",
                "Direct Line secret is required. Set MICROSOFT_DIRECTLINE_SECRET environment variable.",
            );
            return Err(LLMError::AuthenticationError(formatted));
        }

        Ok(())
    }
}

#[async_trait]
impl LLMClient for MicrosoftProvider {
    async fn generate(&mut self, prompt: &str) -> Result<llm_types::LLMResponse, LLMError> {
        let request = LLMRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text(prompt.to_string()),
                reasoning: None,
                reasoning_details: None,
                tool_calls: None,
                tool_call_id: None,
                origin_tool: None,
            }],
            system_prompt: None,
            tools: None,
            model: self.model.clone(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let response = LLMProvider::generate(self, request).await?;

        Ok(llm_types::LLMResponse {
            content: response.content.unwrap_or_default(),
            model: self.model.clone(),
            usage: response.usage.map(|u| llm_types::Usage {
                prompt_tokens: u.prompt_tokens as usize,
                completion_tokens: u.completion_tokens as usize,
                total_tokens: u.total_tokens as usize,
                cached_prompt_tokens: None,
                cache_creation_tokens: None,
                cache_read_tokens: None,
            }),
            reasoning: response.reasoning,
        })
    }

    fn backend_kind(&self) -> llm_types::BackendKind {
        llm_types::BackendKind::Microsoft
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
