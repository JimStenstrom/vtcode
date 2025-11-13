use crate::config::constants::{env_vars, models, urls};
use crate::config::core::PromptCachingConfig;
use crate::llm::client::LLMClient;
use crate::llm::error_display;
use crate::llm::provider::{
    FinishReason, LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream, LLMStreamEvent,
    Message, MessageContent, MessageRole, ToolCall, ToolChoice, ToolDefinition,
};
use crate::llm::types as llm_types;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use super::common::{override_base_url, resolve_model};

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

#[derive(Debug, Serialize)]
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
}

impl MicrosoftProvider {
    pub fn new(secret: String) -> Self {
        Self::from_config(Some(secret), None, None, None)
    }

    pub fn with_model(secret: String, model: String) -> Self {
        Self::from_config(Some(secret), Some(model), None, None)
    }

    pub fn from_config(
        secret: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
        _prompt_cache: Option<PromptCachingConfig>,
    ) -> Self {
        let resolved_secret = secret.unwrap_or_else(|| {
            std::env::var("MICROSOFT_DIRECTLINE_SECRET").unwrap_or_default()
        });
        let resolved_model = resolve_model(model, models::microsoft::DEFAULT_MODEL);
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
            secret: resolved_secret,
            base_url: resolved_base_url,
            model: resolved_model,
            http_client,
        }
    }

    async fn start_conversation(&self) -> Result<ConversationResponse, LLMError> {
        let url = format!("{}/conversations", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .send()
            .await
            .map_err(|e| LLMError::Network(format!("Failed to start conversation: {}", e)))?;

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

        response.json::<ConversationResponse>().await.map_err(|e| {
            LLMError::Provider(error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to parse conversation response: {}", e),
            ))
        })
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

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.secret))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await
            .map_err(|e| LLMError::Network(format!("Failed to send activity: {}", e)))?;

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
            .map_err(|e| LLMError::Network(format!("Failed to get activities: {}", e)))?;

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

        response.json::<ActivitiesResponse>().await.map_err(|e| {
            LLMError::Provider(error_display::format_llm_error(
                "Microsoft",
                &format!("Failed to parse activities response: {}", e),
            ))
        })
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
                        if let Some(text) = &part.text {
                            combined.push_str(text);
                            combined.push('\n');
                        }
                    }
                }
            }
        }

        combined.trim().to_string()
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

        for _ in 0..max_attempts {
            tokio::time::sleep(Duration::from_secs(1)).await;

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
                        if let Some(text) = activity.text {
                            return Ok(LLMResponse {
                                content: text,
                                finish_reason: FinishReason::Stop,
                                usage: None,
                                cached_prompt: None,
                                reasoning: None,
                            });
                        }
                    }
                }
            }
        }

        Err(LLMError::Provider(error_display::format_llm_error(
            "Microsoft",
            "Timeout waiting for bot response",
        )))
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
            return Err(LLMError::Authentication(formatted));
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
                tool_calls: None,
                tool_call_id: None,
            }],
            system_prompt: None,
            tools: None,
            model: self.model.clone(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            reasoning_effort: None,
        };

        let response = LLMProvider::generate(self, request).await?;

        Ok(llm_types::LLMResponse {
            content: response.content,
            model: self.model.clone(),
            usage: response.usage.map(|u| llm_types::Usage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
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
