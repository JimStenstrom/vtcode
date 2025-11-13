use crate::config::constants::{env_vars, models, urls};
use crate::config::core::{DeepSeekPromptCacheSettings, PromptCachingConfig};
use crate::impl_provider_constructors;
use crate::llm::client::LLMClient;
use crate::llm::error_display;
use crate::llm::provider::{
    FinishReason, LLMError, LLMProvider, LLMRequest, LLMResponse, Message, MessageContent,
    MessageRole, ToolCall, ToolDefinition, Usage,
};
use crate::llm::types as llm_types;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::{Map, Value, json};

use super::{
    common::resolve_model,
    extract_reasoning_trace,
};

const PROVIDER_NAME: &str = "DeepSeek";
const PROVIDER_KEY: &str = "deepseek";

pub struct DeepSeekProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_enabled: bool,
    prompt_cache_settings: DeepSeekPromptCacheSettings,
}

impl DeepSeekProvider {
    impl_provider_constructors!(default_model: models::deepseek::DEFAULT_MODEL, resolve_fn: resolve_model);

    fn with_model_internal(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        use super::common::ProviderBuilder;

        let builder = ProviderBuilder::new(api_key, model, urls::DEEPSEEK_API_BASE)
            .with_base_url(base_url, Some(env_vars::DEEPSEEK_BASE_URL))
            .with_prompt_cache(
                prompt_cache,
                |providers| &providers.deepseek,
                |cfg, provider_settings| cfg.enabled && provider_settings.enabled,
            );

        Self {
            api_key: builder.api_key,
            http_client: builder.http_client,
            base_url: builder.base_url,
            model: builder.model,
            prompt_cache_enabled: builder.prompt_cache_enabled,
            prompt_cache_settings: builder.prompt_cache_settings,
        }
    }

    fn default_request(&self, prompt: &str) -> LLMRequest {
        LLMRequest {
            messages: vec![Message::user(prompt.to_string())],
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
        }
    }

    fn parse_client_prompt(&self, prompt: &str) -> LLMRequest {
        let trimmed = prompt.trim_start();
        if trimmed.starts_with('{') {
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                if let Some(request) = self.parse_chat_request(&value) {
                    return request;
                }
            }
        }

        self.default_request(prompt)
    }

    fn parse_chat_request(&self, value: &Value) -> Option<LLMRequest> {
        let messages_value = value.get("messages")?.as_array()?;
        let mut system_prompt = value
            .get("system")
            .and_then(|entry| entry.as_str())
            .map(|text| text.to_string());
        let mut messages = Vec::new();

        for entry in messages_value {
            let role = entry
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or(crate::config::constants::message_roles::USER);
            let content = entry
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .to_string();

            match role {
                "system" => {
                    if system_prompt.is_none() && !content.is_empty() {
                        system_prompt = Some(content);
                    }
                }
                "assistant" => {
                    let tool_calls = entry
                        .get("tool_calls")
                        .and_then(|tc| tc.as_array())
                        .map(|calls| {
                            calls
                                .iter()
                                .filter_map(|call| Self::parse_tool_call(call))
                                .collect::<Vec<_>>()
                        })
                        .filter(|calls| !calls.is_empty());

                    messages.push(Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::from(content),
                        reasoning: None,
                        reasoning_details: None,
                        tool_calls,
                        tool_call_id: None,
                        origin_tool: None,
                    });
                }
                "tool" => {
                    if let Some(tool_call_id) = entry.get("tool_call_id").and_then(|v| v.as_str()) {
                        messages.push(Message::tool_response(tool_call_id.to_string(), content));
                    }
                }
                _ => {
                    messages.push(Message::user(content));
                }
            }
        }

        Some(LLMRequest {
            messages,
            system_prompt,
            model: value
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or(&self.model)
                .to_string(),
            max_tokens: value
                .get("max_tokens")
                .and_then(|m| m.as_u64())
                .map(|m| m as u32),
            temperature: value
                .get("temperature")
                .and_then(|t| t.as_f64())
                .map(|t| t as f32),
            stream: value
                .get("stream")
                .and_then(|s| s.as_bool())
                .unwrap_or(false),
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        })
    }

    fn parse_tool_call(value: &Value) -> Option<ToolCall> {
        let id = value.get("id").and_then(|v| v.as_str())?;
        let function = value.get("function")?.as_object()?;
        let name = function.get("name").and_then(|v| v.as_str())?;
        let arguments = function.get("arguments").map(|arg| match arg {
            Value::String(text) => text.to_string(),
            _ => arg.to_string(),
        });

        Some(ToolCall::function(
            id.to_string(),
            name.to_string(),
            arguments.unwrap_or_else(|| "{}".to_string()),
        ))
    }

    fn convert_to_deepseek_format(&self, request: &LLMRequest) -> Result<Value, LLMError> {
        let mut payload = Map::new();

        payload.insert("model".to_string(), Value::String(request.model.clone()));
        payload.insert(
            "messages".to_string(),
            Value::Array(self.serialize_messages(request)?),
        );

        if let Some(system_prompt) = &request.system_prompt {
            payload.insert(
                "system".to_string(),
                Value::String(system_prompt.trim().to_string()),
            );
        }

        if let Some(max_tokens) = request.max_tokens {
            payload.insert(
                "max_tokens".to_string(),
                Value::Number(serde_json::Number::from(max_tokens as u64)),
            );
        }

        if let Some(temperature) = request.temperature {
            payload.insert(
                "temperature".to_string(),
                Value::Number(serde_json::Number::from_f64(temperature as f64).ok_or_else(
                    || LLMError::InvalidRequest("Invalid temperature value".to_string()),
                )?),
            );
        }

        if request.stream {
            payload.insert("stream".to_string(), Value::Bool(true));
        }

        if let Some(tools) = &request.tools {
            if let Some(serialized_tools) = Self::serialize_tools(tools) {
                payload.insert("tools".to_string(), Value::Array(serialized_tools));
            }
        }

        if let Some(choice) = &request.tool_choice {
            payload.insert(
                "tool_choice".to_string(),
                choice.to_provider_format(PROVIDER_KEY),
            );
        }

        if let Some(effort) = request.reasoning_effort {
            payload.insert(
                "reasoning_effort".to_string(),
                Value::String(effort.as_str().to_string()),
            );
        }

        Ok(Value::Object(payload))
    }

    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError> {
        let mut messages = Vec::with_capacity(request.messages.len());

        for message in &request.messages {
            message
                .validate_for_provider(PROVIDER_KEY)
                .map_err(LLMError::InvalidRequest)?;

            let mut message_map = Map::new();
            message_map.insert(
                "role".to_string(),
                Value::String(message.role.as_generic_str().to_string()),
            );
            message_map.insert(
                "content".to_string(),
                Value::String(message.content.as_text()),
            );

            if let Some(tool_calls) = &message.tool_calls {
                let serialized_calls = tool_calls
                    .iter()
                    .map(|call| {
                        json!({
                            "id": call.id.clone(),
                            "type": "function",
                            "function": {
                                "name": call.function.name.clone(),
                                "arguments": call.function.arguments.clone()
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                message_map.insert("tool_calls".to_string(), Value::Array(serialized_calls));
            }

            if let Some(tool_call_id) = &message.tool_call_id {
                message_map.insert(
                    "tool_call_id".to_string(),
                    Value::String(tool_call_id.clone()),
                );
            }

            messages.push(Value::Object(message_map));
        }

        Ok(messages)
    }

    fn serialize_tools(tools: &[ToolDefinition]) -> Option<Vec<Value>> {
        if tools.is_empty() {
            return None;
        }

        Some(tools.iter().map(|tool| json!(tool)).collect::<Vec<_>>())
    }

    fn parse_response(&self, response_json: Value) -> Result<LLMResponse, LLMError> {
        let choices = response_json
            .get("choices")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                let formatted_error = error_display::format_llm_error(
                    PROVIDER_NAME,
                    "Invalid response format: missing choices",
                );
                LLMError::Provider(formatted_error)
            })?;

        if choices.is_empty() {
            let formatted_error =
                error_display::format_llm_error(PROVIDER_NAME, "No choices in response");
            return Err(LLMError::Provider(formatted_error));
        }

        let choice = &choices[0];
        let message = choice.get("message").ok_or_else(|| {
            let formatted_error = error_display::format_llm_error(
                PROVIDER_NAME,
                "Invalid response format: missing message",
            );
            LLMError::Provider(formatted_error)
        })?;

        let content = message
            .get("content")
            .and_then(|value| match value {
                Value::String(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }
                Value::Array(parts) => Some(
                    parts
                        .iter()
                        .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
                _ => None,
            })
            .filter(|text| !text.is_empty());

        let tool_calls = message
            .get("tool_calls")
            .and_then(|tc| tc.as_array())
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|call| Self::parse_tool_call(call))
                    .collect::<Vec<_>>()
            })
            .filter(|calls| !calls.is_empty());

        let reasoning = message
            .get("reasoning_content")
            .and_then(extract_reasoning_trace)
            .or_else(|| message.get("reasoning").and_then(extract_reasoning_trace))
            .or_else(|| {
                choice
                    .get("reasoning_content")
                    .and_then(extract_reasoning_trace)
            });

        let finish_reason = choice
            .get("finish_reason")
            .and_then(|value| value.as_str())
            .map(|reason| match reason {
                "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                "tool_calls" => FinishReason::ToolCalls,
                other => FinishReason::Error(other.to_string()),
            })
            .unwrap_or(FinishReason::Stop);

        let usage = response_json.get("usage").map(|usage_value| Usage {
            prompt_tokens: usage_value
                .get("prompt_tokens")
                .and_then(|value| value.as_u64())
                .unwrap_or(0) as u32,
            completion_tokens: usage_value
                .get("completion_tokens")
                .and_then(|value| value.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: usage_value
                .get("total_tokens")
                .and_then(|value| value.as_u64())
                .unwrap_or(0) as u32,
            cached_prompt_tokens: if self.prompt_cache_enabled
                && self.prompt_cache_settings.surface_metrics
            {
                usage_value
                    .get("prompt_cache_hit_tokens")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as u32)
            } else {
                None
            },
            cache_creation_tokens: if self.prompt_cache_enabled
                && self.prompt_cache_settings.surface_metrics
            {
                usage_value
                    .get("prompt_cache_miss_tokens")
                    .and_then(|value| value.as_u64())
                    .map(|value| value as u32)
            } else {
                None
            },
            cache_read_tokens: None,
        });

        Ok(LLMResponse {
            content,
            tool_calls,
            usage,
            finish_reason,
            reasoning,
            reasoning_details: None,
        })
    }
}

#[async_trait]
impl LLMProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        PROVIDER_KEY
    }

    fn supports_reasoning(&self, model: &str) -> bool {
        let target = if model.trim().is_empty() {
            &self.model
        } else {
            model
        };
        target == models::deepseek::DEEPSEEK_REASONER
    }

    fn supports_reasoning_effort(&self, _model: &str) -> bool {
        false
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let mut request = request;
        if request.model.trim().is_empty() {
            request.model = self.model.clone();
        }

        let payload = self.convert_to_deepseek_format(&request)?;
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                let formatted_error = error_display::format_llm_error(
                    PROVIDER_NAME,
                    &format!("Network error: {}", e),
                );
                LLMError::Network(formatted_error)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
                let formatted_error = error_display::format_llm_error(
                    PROVIDER_NAME,
                    "Authentication failed (check DEEPSEEK_API_KEY)",
                );
                return Err(LLMError::Authentication(formatted_error));
            }

            if status.as_u16() == 429 || error_text.contains("quota") {
                return Err(LLMError::RateLimit);
            }

            let formatted_error = error_display::format_llm_error(
                PROVIDER_NAME,
                &format!("HTTP {}: {}", status, error_text),
            );
            return Err(LLMError::Provider(formatted_error));
        }

        let response_json: Value = response.json().await.map_err(|e| {
            let formatted_error = error_display::format_llm_error(
                PROVIDER_NAME,
                &format!("Failed to parse response: {}", e),
            );
            LLMError::Provider(formatted_error)
        })?;

        self.parse_response(response_json)
    }

    fn supported_models(&self) -> Vec<String> {
        models::deepseek::SUPPORTED_MODELS
            .iter()
            .map(|model| model.to_string())
            .collect()
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        for message in &request.messages {
            message
                .validate_for_provider(PROVIDER_KEY)
                .map_err(LLMError::InvalidRequest)?;
        }
        Ok(())
    }
}

#[async_trait]
impl LLMClient for DeepSeekProvider {
    async fn generate(&mut self, prompt: &str) -> Result<llm_types::LLMResponse, LLMError> {
        let request = self.parse_client_prompt(prompt);
        let model = request.model.clone();
        let response = LLMProvider::generate(self, request).await?;

        Ok(llm_types::LLMResponse {
            content: response.content.unwrap_or_default(),
            model,
            usage: response.usage.map(|usage| llm_types::Usage {
                prompt_tokens: usage.prompt_tokens as usize,
                completion_tokens: usage.completion_tokens as usize,
                total_tokens: usage.total_tokens as usize,
                cached_prompt_tokens: usage.cached_prompt_tokens.map(|value| value as usize),
                cache_creation_tokens: usage.cache_creation_tokens.map(|value| value as usize),
                cache_read_tokens: usage.cache_read_tokens.map(|value| value as usize),
            }),
            reasoning: response.reasoning,
        })
    }

    fn backend_kind(&self) -> llm_types::BackendKind {
        llm_types::BackendKind::DeepSeek
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::constants::models;
    use crate::config::core::PromptCachingConfig;
    use crate::config::types::ReasoningEffortLevel;
    use crate::llm::providers::test_utils::*;

    fn create_test_provider() -> DeepSeekProvider {
        DeepSeekProvider::with_model("test_key".to_string(), models::deepseek::DEFAULT_MODEL.to_string())
    }

    // ==================== Constructor Tests ====================

    #[test]
    fn new_creates_provider_with_default_model() {
        let provider = DeepSeekProvider::new("test_key".to_string());
        assert_eq!(provider.model, models::deepseek::DEFAULT_MODEL);
        assert_eq!(provider.api_key, "test_key");
    }

    #[test]
    fn with_model_creates_provider_with_custom_model() {
        let custom_model = "deepseek-chat";
        let provider = DeepSeekProvider::with_model("test_key".to_string(), custom_model.to_string());
        assert_eq!(provider.model, custom_model);
    }

    #[test]
    fn from_config_uses_defaults_when_none() {
        let provider = DeepSeekProvider::from_config(None, None, None, None);
        assert_eq!(provider.model, models::deepseek::DEFAULT_MODEL);
        assert_eq!(provider.api_key, "");
    }

    #[test]
    fn from_config_uses_provided_values() {
        let provider = DeepSeekProvider::from_config(
            Some("custom_key".to_string()),
            Some("custom_model".to_string()),
            None,
            None,
        );
        assert_eq!(provider.api_key, "custom_key");
        assert_eq!(provider.model, "custom_model");
    }

    // ==================== Message Serialization Tests ====================

    #[test]
    fn serialize_messages_simple_user_message() {
        let provider = create_test_provider();
        let request = simple_request(models::deepseek::DEFAULT_MODEL);
        
        let messages = provider.serialize_messages(&request).expect("serialization should succeed");
        
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello, world!");
    }

    #[test]
    fn serialize_messages_multiple_messages() {
        let provider = create_test_provider();
        let request = multi_message_request(models::deepseek::DEFAULT_MODEL);
        
        let messages = provider.serialize_messages(&request).expect("serialization should succeed");
        
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[2]["role"], "user");
    }

    #[test]
    fn serialize_messages_with_tool_calls() {
        let provider = create_test_provider();
        let tool_call = sample_tool_call();
        let mut request = simple_request(models::deepseek::DEFAULT_MODEL);
        request.messages = vec![Message::assistant_with_tool_calls(vec![tool_call])];
        
        let messages = provider.serialize_messages(&request).expect("serialization should succeed");
        
        assert_eq!(messages.len(), 1);
        assert!(messages[0].get("tool_calls").is_some());
        let tool_calls = messages[0]["tool_calls"].as_array().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], "call_123");
        assert_eq!(tool_calls[0]["type"], "function");
    }

    #[test]
    fn serialize_messages_with_tool_result() {
        let provider = create_test_provider();
        let request = request_with_tool_result(models::deepseek::DEFAULT_MODEL);
        
        let messages = provider.serialize_messages(&request).expect("serialization should succeed");
        
        assert_eq!(messages.len(), 3);
        // Check tool result message
        let tool_result_msg = &messages[2];
        assert_eq!(tool_result_msg["role"], "tool");
        assert!(tool_result_msg.get("tool_call_id").is_some());
    }

    #[test]
    fn serialize_messages_with_special_characters() {
        let provider = create_test_provider();
        let request = request_with_special_chars(models::deepseek::DEFAULT_MODEL);
        
        let messages = provider.serialize_messages(&request).expect("serialization should succeed");
        
        assert_eq!(messages.len(), 1);
        let content = messages[0]["content"].as_str().unwrap();
        assert_contains(content, "quotes");
        assert_contains(content, "backslash");
        assert_contains(content, "🎉");
    }

    // ==================== Tool Serialization Tests ====================

    #[test]
    fn serialize_tools_single_tool() {
        let tools = vec![weather_tool()];
        let serialized = DeepSeekProvider::serialize_tools(&tools);
        
        assert!(serialized.is_some());
        let tools_array = serialized.unwrap();
        assert_eq!(tools_array.len(), 1);
        assert_eq!(tools_array[0]["name"], "get_weather");
    }

    #[test]
    fn serialize_tools_multiple_tools() {
        let tools = vec![weather_tool(), calculator_tool()];
        let serialized = DeepSeekProvider::serialize_tools(&tools);
        
        assert!(serialized.is_some());
        let tools_array = serialized.unwrap();
        assert_eq!(tools_array.len(), 2);
    }

    #[test]
    fn serialize_tools_empty_array_returns_none() {
        let tools: Vec<ToolDefinition> = vec![];
        let serialized = DeepSeekProvider::serialize_tools(&tools);
        
        assert!(serialized.is_none());
    }

    #[test]
    fn serialize_tools_with_complex_parameters() {
        let tools = vec![complex_tool()];
        let serialized = DeepSeekProvider::serialize_tools(&tools);
        
        assert!(serialized.is_some());
        let tools_array = serialized.unwrap();
        assert_eq!(tools_array[0]["name"], "search_database");
        assert!(tools_array[0]["parameters"].is_object());
    }

    // ==================== Request Building Tests ====================

    #[test]
    fn convert_to_deepseek_format_includes_required_fields() {
        let provider = create_test_provider();
        let request = simple_request(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "model");
        assert_json_has_field(&payload, "messages");
        assert_eq!(payload["model"], models::deepseek::DEFAULT_MODEL);
    }

    #[test]
    fn convert_to_deepseek_format_includes_system_prompt() {
        let provider = create_test_provider();
        let request = request_with_system_prompt(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "system");
        assert!(payload["system"].as_str().unwrap().contains("weather"));
    }

    #[test]
    fn convert_to_deepseek_format_includes_tools() {
        let provider = create_test_provider();
        let request = request_with_tools(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "tools");
        assert_json_array_length(&payload, "tools", 2);
    }

    #[test]
    fn convert_to_deepseek_format_includes_tool_choice() {
        let provider = create_test_provider();
        let request = request_with_tool_choice(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "tool_choice");
    }

    #[test]
    fn convert_to_deepseek_format_includes_max_tokens() {
        let provider = create_test_provider();
        let request = simple_request(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "max_tokens");
        assert_eq!(payload["max_tokens"], 100);
    }

    #[test]
    fn convert_to_deepseek_format_includes_temperature() {
        let provider = create_test_provider();
        let request = simple_request(models::deepseek::DEFAULT_MODEL);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "temperature");
        assert_eq!(payload["temperature"].as_f64().unwrap(), 0.7);
    }

    #[test]
    fn convert_to_deepseek_format_includes_stream_when_true() {
        let provider = create_test_provider();
        let mut request = simple_request(models::deepseek::DEFAULT_MODEL);
        request.stream = true;
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "stream");
        assert_eq!(payload["stream"], true);
    }

    #[test]
    fn convert_to_deepseek_format_omits_stream_when_false() {
        let provider = create_test_provider();
        let mut request = simple_request(models::deepseek::DEFAULT_MODEL);
        request.stream = false;
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert!(payload.get("stream").is_none());
    }

    #[test]
    fn convert_to_deepseek_format_includes_reasoning_effort() {
        let provider = create_test_provider();
        let mut request = simple_request(models::deepseek::DEFAULT_MODEL);
        request.reasoning_effort = Some(ReasoningEffortLevel::High);
        
        let payload = provider.convert_to_deepseek_format(&request).expect("conversion should succeed");
        
        assert_json_has_field(&payload, "reasoning_effort");
        assert_eq!(payload["reasoning_effort"], "high");
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn handles_empty_message_content() {
        let provider = create_test_provider();
        let request = request_with_empty_message(models::deepseek::DEFAULT_MODEL);
        
        let messages = provider.serialize_messages(&request).expect("should handle empty message");
        
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["content"], "");
    }

    #[test]
    fn handles_long_message() {
        let provider = create_test_provider();
        let request = request_with_long_message(models::deepseek::DEFAULT_MODEL);
        
        let result = provider.serialize_messages(&request);
        
        assert!(result.is_ok(), "should handle long messages");
    }

    #[test]
    fn supported_models_returns_non_empty_list() {
        let provider = create_test_provider();
        let models = provider.supported_models();
        
        assert!(!models.is_empty());
        assert!(models.contains(&models::deepseek::DEFAULT_MODEL.to_string()));
    }

    #[test]
    fn validate_request_accepts_valid_request() {
        let provider = create_test_provider();
        let request = simple_request(models::deepseek::DEFAULT_MODEL);
        
        let result = provider.validate_request(&request);
        
        assert!(result.is_ok());
    }
}
