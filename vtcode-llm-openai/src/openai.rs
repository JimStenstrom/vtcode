//! OpenAI LLM provider implementation

use async_stream::try_stream;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use std::time::Duration;
use tracing::debug;

use crate::provider::{LLMProvider, LLMStream, LLMStreamEvent};
use crate::types::*;

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";

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
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| OPENAI_API_BASE.to_string()),
            model,
        }
    }

    /// Create from configuration
    pub fn from_config(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
    ) -> Self {
        let api_key_value = api_key.unwrap_or_default();
        let model_value = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let base_url_value = base_url
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| OPENAI_API_BASE.to_string());

        Self {
            api_key: api_key_value,
            http_client: HttpClient::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
            base_url: base_url_value,
            model: model_value,
        }
    }

    fn authorize(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.trim().is_empty() {
            builder
        } else {
            builder.bearer_auth(&self.api_key)
        }
    }

    fn serialize_messages(&self, messages: &[Message], system_prompt: Option<&String>) -> Vec<Value> {
        let mut result = Vec::new();

        // Add system prompt if provided
        if let Some(system) = system_prompt {
            result.push(json!({
                "role": "system",
                "content": system
            }));
        }

        // Add messages
        for msg in messages {
            let mut message_json = json!({
                "role": match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                },
                "content": msg.content.as_text(),
            });

            // Add tool_call_id if present
            if let Some(tool_call_id) = &msg.tool_call_id {
                message_json["tool_call_id"] = json!(tool_call_id);
            }

            // Add tool_calls if present
            if let Some(tool_calls) = &msg.tool_calls {
                let tool_calls_json: Vec<Value> = tool_calls
                    .iter()
                    .map(|tc| {
                        json!({
                            "id": tc.id,
                            "type": tc.call_type,
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments,
                            }
                        })
                    })
                    .collect();
                message_json["tool_calls"] = json!(tool_calls_json);
            }

            result.push(message_json);
        }

        result
    }

    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Option<Vec<Value>> {
        if tools.is_empty() {
            return None;
        }

        Some(
            tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool.function.name,
                            "description": tool.function.description,
                            "parameters": tool.function.parameters,
                        }
                    })
                })
                .collect(),
        )
    }

    fn parse_response(&self, response_json: Value) -> Result<LLMResponse, LLMError> {
        let choices = response_json
            .get("choices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LLMError::Provider("Invalid response: missing choices".to_string()))?;

        if choices.is_empty() {
            return Err(LLMError::Provider("No choices in response".to_string()));
        }

        let choice = &choices[0];
        let message = choice.get("message").ok_or_else(|| {
            LLMError::Provider("Invalid response: missing message".to_string())
        })?;

        let content = message
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse tool calls if present
        let tool_calls = message.get("tool_calls").and_then(|tc| {
            tc.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|call| {
                        Some(ToolCall {
                            id: call.get("id")?.as_str()?.to_string(),
                            call_type: call.get("type")?.as_str()?.to_string(),
                            function: FunctionCall {
                                name: call.get("function")?.get("name")?.as_str()?.to_string(),
                                arguments: call.get("function")?.get("arguments")?.as_str()?.to_string(),
                            },
                        })
                    })
                    .collect()
            })
        });

        // Parse usage
        let usage = response_json.get("usage").and_then(|u| {
            Some(Usage {
                prompt_tokens: u.get("prompt_tokens")?.as_u64()? as usize,
                completion_tokens: u.get("completion_tokens")?.as_u64()? as usize,
                total_tokens: u.get("total_tokens")?.as_u64()? as usize,
                cached_prompt_tokens: u
                    .get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|t| t.as_u64())
                    .map(|t| t as usize),
                cache_creation_tokens: None,
                cache_read_tokens: None,
            })
        });

        // Parse finish reason
        let finish_reason = choice
            .get("finish_reason")
            .and_then(|fr| fr.as_str())
            .and_then(|fr_str| match fr_str {
                "stop" => Some(FinishReason::Stop),
                "length" => Some(FinishReason::Length),
                "tool_calls" => Some(FinishReason::ToolCalls),
                "content_filter" => Some(FinishReason::ContentFilter),
                "function_call" => Some(FinishReason::FunctionCall),
                _ => None,
            });

        Ok(LLMResponse {
            content,
            model: response_json
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or(&self.model)
                .to_string(),
            usage,
            reasoning: None,
            tool_calls,
            finish_reason,
        })
    }

    async fn generate_internal(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let messages = self.serialize_messages(&request.messages, request.system_prompt.as_ref());

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

        if let Some(tools) = &request.tools {
            if let Some(tools_json) = self.serialize_tools(tools) {
                body["tools"] = json!(tools_json);

                if let Some(tool_choice) = &request.tool_choice {
                    body["tool_choice"] = match tool_choice {
                        ToolChoice::Auto => json!("auto"),
                        ToolChoice::None => json!("none"),
                        ToolChoice::Required => json!("required"),
                        ToolChoice::Specific(s) => json!(s),
                    };
                }

                if let Some(parallel_tool_calls) = request.parallel_tool_calls {
                    body["parallel_tool_calls"] = json!(parallel_tool_calls);
                }
            }
        }

        let url = format!("{}/chat/completions", self.base_url);

        debug!(target: "vtcode_llm_openai", "Sending request to {}", url);

        let response = self
            .authorize(self.http_client.post(&url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    LLMError::Authentication("Invalid API key".to_string())
                } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    LLMError::RateLimit
                } else {
                    LLMError::Network(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError(format!("API error ({}): {}", status, error_text)));
        }

        let response_json: Value = response.json().await?;
        self.parse_response(response_json)
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
        if model.contains("gpt-4o") {
            128_000
        } else if model.contains("gpt-4-turbo") {
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

        let messages = self.serialize_messages(&request.messages, request.system_prompt.as_ref());

        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(tools) = &request.tools {
            if let Some(tools_json) = self.serialize_tools(tools) {
                body["tools"] = json!(tools_json);
            }
        }

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .authorize(self.http_client.post(&url))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError(error_text));
        }

        let model = self.model.clone();
        let mut byte_stream = response.bytes_stream();

        let stream = try_stream! {
            let mut buffer = String::new();
            let mut accumulated_content = String::new();

            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk.map_err(|e| LLMError::StreamError(e.to_string()))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() || !line.starts_with("data: ") {
                        continue;
                    }

                    let data = &line[6..];
                    if data == "[DONE]" {
                        yield LLMStreamEvent::Completed {
                            response: LLMResponse {
                                content: accumulated_content.clone(),
                                model: model.clone(),
                                usage: None,
                                reasoning: None,
                                tool_calls: None,
                                finish_reason: Some(FinishReason::Stop),
                            }
                        };
                        break;
                    }

                    if let Ok(json_value) = serde_json::from_str::<Value>(data) {
                        if let Some(choices) = json_value.get("choices").and_then(|v| v.as_array()) {
                            if let Some(choice) = choices.first() {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                                        accumulated_content.push_str(content);
                                        yield LLMStreamEvent::Delta {
                                            content: content.to_string(),
                                            reasoning: None,
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new("test_key".to_string());
        assert_eq!(provider.name(), "openai");
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_model_capabilities() {
        let provider = OpenAIProvider::new("test_key".to_string());

        assert!(provider.supports_reasoning("o1-preview"));
        assert!(!provider.supports_reasoning("gpt-4"));

        assert!(provider.supports_tools("gpt-4"));
        assert!(!provider.supports_tools("o1-preview"));
    }

    #[test]
    fn test_context_sizes() {
        let provider = OpenAIProvider::new("test_key".to_string());

        assert_eq!(provider.effective_context_size("gpt-4o"), 128_000);
        assert_eq!(provider.effective_context_size("gpt-4"), 8_192);
        assert_eq!(provider.effective_context_size("gpt-3.5-turbo"), 4_096);
    }
}
