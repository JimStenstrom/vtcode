//! Anthropic provider implementation for Claude models

use crate::provider::LLMProvider;
use crate::types::*;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use std::env;

// Constants
const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";
const DEFAULT_MAX_TOKENS: u32 = 8192;

const REASONING_MODELS: &[&str] = &[
    "claude-3-7-sonnet-20250219",
    "claude-3-7-sonnet-latest",
    "claude-sonnet-4-5-20250929",
    "claude-sonnet-4-5",
];

const SUPPORTED_MODELS: &[&str] = &[
    "claude-3-5-sonnet-20241022",
    "claude-3-5-sonnet-20240620",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
    "claude-2.1",
    "claude-2.0",
    "claude-instant-1.2",
    "claude-3-7-sonnet-20250219",
    "claude-3-7-sonnet-latest",
    "claude-sonnet-4-5-20250929",
    "claude-sonnet-4-5",
];

pub struct AnthropicProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_config: PromptCachingConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the default model
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, DEFAULT_MODEL.to_string())
    }

    /// Create a new Anthropic provider with a specific model
    pub fn with_model(api_key: String, model: String) -> Self {
        Self::from_config(api_key, model, None, None)
    }

    /// Create a new Anthropic provider with full configuration
    pub fn from_config(
        api_key: String,
        model: String,
        base_url: Option<String>,
        prompt_cache_config: Option<PromptCachingConfig>,
    ) -> Self {
        let base_url_value = base_url
            .or_else(|| env::var("ANTHROPIC_BASE_URL").ok())
            .unwrap_or_else(|| ANTHROPIC_API_BASE.to_string());

        Self {
            api_key,
            http_client: HttpClient::new(),
            base_url: base_url_value,
            model,
            prompt_cache_config: prompt_cache_config.unwrap_or_default(),
        }
    }

    /// Determines the TTL string for cache control.
    /// Anthropic only supports "5m" (5 minutes) or "1h" (1 hour).
    fn get_cache_ttl(&self) -> &'static str {
        self.prompt_cache_config
            .extended_ttl_seconds
            .filter(|&ttl| ttl >= 3600)
            .map(|_| "1h")
            .unwrap_or("5m")
    }

    /// Returns the cache control JSON block for Anthropic API.
    fn cache_control_value(&self) -> Option<Value> {
        if !self.prompt_cache_config.enabled {
            return None;
        }

        Some(json!({
            "type": "ephemeral",
            "ttl": self.get_cache_ttl()
        }))
    }

    /// Returns the beta header value for Anthropic API prompt caching.
    fn prompt_cache_beta_header_value(&self) -> Option<String> {
        if !self.prompt_cache_config.enabled {
            return None;
        }

        let mut betas = vec!["prompt-caching-2024-07-31"];

        // Only add extended TTL beta if we're actually using 1h cache
        if self.get_cache_ttl() == "1h" {
            betas.push("extended-cache-ttl-2025-04-11");
        }

        Some(betas.join(", "))
    }

    fn tool_result_blocks(content: &str) -> Vec<Value> {
        if content.trim().is_empty() {
            return vec![json!({"type": "text", "text": ""})];
        }

        if let Ok(parsed) = serde_json::from_str::<Value>(content) {
            match parsed {
                Value::String(text) => vec![json!({"type": "text", "text": text})],
                Value::Array(items) => {
                    let mut blocks = Vec::new();
                    for item in items {
                        if let Some(text) = item.as_str() {
                            blocks.push(json!({"type": "text", "text": text}));
                        } else {
                            blocks.push(json!({"type": "json", "json": item}));
                        }
                    }
                    if blocks.is_empty() {
                        vec![json!({"type": "json", "json": Value::Array(vec![])})]
                    } else {
                        blocks
                    }
                }
                other => vec![json!({"type": "json", "json": other})],
            }
        } else {
            vec![json!({"type": "text", "text": content})]
        }
    }

    fn convert_to_anthropic_format(&self, request: &LLMRequest) -> Result<Value, LLMError> {
        let cache_control_template = if self.prompt_cache_config.enabled {
            self.cache_control_value()
        } else {
            None
        };

        let mut breakpoints_remaining = cache_control_template
            .as_ref()
            .map(|_| self.prompt_cache_config.max_breakpoints as usize)
            .unwrap_or(0);

        // Convert tools
        let mut tools_json: Option<Vec<Value>> = None;
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let mut built_tools: Vec<Value> = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "name": tool.function.name,
                            "description": tool.function.description,
                            "input_schema": tool.function.parameters
                        })
                    })
                    .collect();

                // Add cache control to last tool if enabled
                if breakpoints_remaining > 0 {
                    if let Some(cache_control) = cache_control_template.as_ref() {
                        if let Some(last_tool) = built_tools.last_mut() {
                            last_tool["cache_control"] = cache_control.clone();
                            breakpoints_remaining -= 1;
                        }
                    }
                }

                tools_json = Some(built_tools);
            }
        }

        // Convert system prompt
        let mut system_value: Option<Value> = None;
        if let Some(system_prompt) = &request.system_prompt {
            if self.prompt_cache_config.cache_system_messages && breakpoints_remaining > 0 {
                if let Some(cache_control) = cache_control_template.as_ref() {
                    let mut block = json!({
                        "type": "text",
                        "text": system_prompt
                    });
                    block["cache_control"] = cache_control.clone();
                    system_value = Some(Value::Array(vec![block]));
                    breakpoints_remaining -= 1;
                } else {
                    system_value = Some(Value::String(system_prompt.clone()));
                }
            } else {
                system_value = Some(Value::String(system_prompt.clone()));
            }
        }

        // Convert messages
        let mut messages = Vec::new();

        for msg in &request.messages {
            if msg.role == MessageRole::System {
                continue;
            }

            let content_text = msg.content.as_text();

            match msg.role {
                MessageRole::Assistant => {
                    let mut content_blocks = Vec::new();
                    if !msg.content.is_empty() {
                        content_blocks.push(json!({"type": "text", "text": content_text.clone()}));
                    }
                    if let Some(tool_calls) = &msg.tool_calls {
                        for call in tool_calls {
                            let args: Value = serde_json::from_str(&call.function.arguments)
                                .unwrap_or_else(|_| json!({}));
                            content_blocks.push(json!({
                                "type": "tool_use",
                                "id": call.id,
                                "name": call.function.name,
                                "input": args
                            }));
                        }
                    }
                    if content_blocks.is_empty() {
                        content_blocks.push(json!({"type": "text", "text": ""}));
                    }
                    messages.push(json!({
                        "role": "assistant",
                        "content": content_blocks
                    }));
                }
                MessageRole::Tool => {
                    if let Some(tool_call_id) = &msg.tool_call_id {
                        let blocks = Self::tool_result_blocks(&content_text);
                        messages.push(json!({
                            "role": "user",
                            "content": [{
                                "type": "tool_result",
                                "tool_use_id": tool_call_id,
                                "content": blocks
                            }]
                        }));
                    } else if !msg.content.is_empty() {
                        messages.push(json!({
                            "role": "user",
                            "content": [{"type": "text", "text": content_text.clone()}]
                        }));
                    }
                }
                _ => {
                    if msg.content.is_empty() {
                        continue;
                    }

                    let mut block = json!({
                        "type": "text",
                        "text": content_text.clone()
                    });

                    if msg.role == MessageRole::User
                        && self.prompt_cache_config.cache_user_messages
                        && breakpoints_remaining > 0
                    {
                        if let Some(cache_control) = cache_control_template.as_ref() {
                            block["cache_control"] = cache_control.clone();
                            breakpoints_remaining -= 1;
                        }
                    }

                    messages.push(json!({
                        "role": msg.role.as_anthropic_str(),
                        "content": [block]
                    }));
                }
            }
        }

        if messages.is_empty() {
            return Err(LLMError::InvalidRequest(
                "No convertible messages for Anthropic request".to_string(),
            ));
        }

        // Build request
        let mut anthropic_request = json!({
            "model": request.model,
            "messages": messages,
            "stream": request.stream,
            "max_tokens": request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
        });

        if let Some(system) = system_value {
            anthropic_request["system"] = system;
        }

        if let Some(temperature) = request.temperature {
            anthropic_request["temperature"] = json!(temperature);
        }

        if let Some(tools) = tools_json {
            anthropic_request["tools"] = Value::Array(tools);
        }

        if let Some(tool_choice) = &request.tool_choice {
            anthropic_request["tool_choice"] = tool_choice.to_anthropic_format();
        }

        Ok(anthropic_request)
    }

    fn parse_anthropic_response(&self, response_json: Value) -> Result<LLMResponse, LLMError> {
        let content = response_json
            .get("content")
            .and_then(|c| c.as_array())
            .ok_or_else(|| {
                LLMError::Provider("Invalid response format: missing content".to_string())
            })?;

        let mut text_parts = Vec::new();
        let mut reasoning_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for block in content {
            match block.get("type").and_then(|t| t.as_str()) {
                Some("text") => {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                Some("thinking") => {
                    if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                        reasoning_parts.push(thinking.to_string());
                    } else if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        reasoning_parts.push(text.to_string());
                    }
                }
                Some("tool_use") => {
                    let id = block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                    let arguments =
                        serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string());
                    if !id.is_empty() && !name.is_empty() {
                        tool_calls.push(ToolCall::function(id, name, arguments));
                    }
                }
                _ => {}
            }
        }

        let reasoning = if reasoning_parts.is_empty() {
            None
        } else {
            let joined = reasoning_parts.join("\n");
            let trimmed = joined.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };

        let stop_reason = response_json
            .get("stop_reason")
            .and_then(|sr| sr.as_str())
            .unwrap_or("end_turn");

        let finish_reason = Some(match stop_reason {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "stop_sequence" => FinishReason::Stop,
            "tool_use" => FinishReason::ToolCalls,
            other => FinishReason::Error(other.to_string()),
        });

        let usage = response_json.get("usage").map(|usage_value| {
            let cache_creation_tokens = usage_value
                .get("cache_creation_input_tokens")
                .and_then(|value| value.as_u64())
                .map(|value| value as usize);
            let cache_read_tokens = usage_value
                .get("cache_read_input_tokens")
                .and_then(|value| value.as_u64())
                .map(|value| value as usize);

            Usage {
                prompt_tokens: usage_value
                    .get("input_tokens")
                    .and_then(|it| it.as_u64())
                    .unwrap_or(0) as usize,
                completion_tokens: usage_value
                    .get("output_tokens")
                    .and_then(|ot| ot.as_u64())
                    .unwrap_or(0) as usize,
                total_tokens: (usage_value
                    .get("input_tokens")
                    .and_then(|it| it.as_u64())
                    .unwrap_or(0)
                    + usage_value
                        .get("output_tokens")
                        .and_then(|ot| ot.as_u64())
                        .unwrap_or(0)) as usize,
                cached_prompt_tokens: cache_read_tokens,
                cache_creation_tokens,
                cache_read_tokens,
            }
        });

        let model = response_json
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&self.model)
            .to_string();

        Ok(LLMResponse {
            content: text_parts.join(""),
            model,
            usage,
            reasoning,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            finish_reason,
        })
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports_reasoning(&self, _model: &str) -> bool {
        false
    }

    fn supports_reasoning_effort(&self, model: &str) -> bool {
        let requested = if model.trim().is_empty() {
            self.model.as_str()
        } else {
            model
        };

        REASONING_MODELS.iter().any(|candidate| *candidate == requested)
    }

    fn supports_parallel_tool_config(&self, _model: &str) -> bool {
        true
    }

    fn supports_context_caching(&self, _model: &str) -> bool {
        true
    }

    fn effective_context_size(&self, _model: &str) -> usize {
        200_000
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let anthropic_request = self.convert_to_anthropic_format(&request)?;
        let url = format!("{}/messages", self.base_url);

        let mut request_builder = self
            .http_client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION);

        if let Some(beta_header) = self.prompt_cache_beta_header_value() {
            request_builder = request_builder.header("anthropic-beta", beta_header);
        }

        let response = request_builder
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| LLMError::Network(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Handle specific HTTP status codes
            if status.as_u16() == 429
                || error_text.contains("insufficient_quota")
                || error_text.contains("quota")
                || error_text.contains("rate limit")
            {
                return Err(LLMError::RateLimit);
            }

            let error_message = if error_text.contains("cache_control") {
                format!(
                    "HTTP {} - Cache configuration error: {}. \
                    Note: Anthropic only supports cache_control with type='ephemeral' and ttl='5m' or '1h'.",
                    status, error_text
                )
            } else {
                format!("HTTP {}: {}", status, error_text)
            };

            return Err(LLMError::Provider(error_message));
        }

        let anthropic_response: Value = response
            .json()
            .await
            .map_err(|e| LLMError::Provider(format!("Failed to parse response: {}", e)))?;

        self.parse_anthropic_response(anthropic_response)
    }

    fn supported_models(&self) -> Vec<String> {
        SUPPORTED_MODELS.iter().map(|s| s.to_string()).collect()
    }

    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
        if request.messages.is_empty() {
            return Err(LLMError::InvalidRequest("Messages cannot be empty".to_string()));
        }

        if !self.supported_models().contains(&request.model) {
            return Err(LLMError::InvalidRequest(format!(
                "Unsupported model: {}",
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
    fn cache_headers_reflect_extended_ttl() {
        let mut config = PromptCachingConfig::default();
        config.enabled = true;
        config.extended_ttl_seconds = Some(3600);

        let provider = AnthropicProvider::from_config(
            "test-key".to_string(),
            DEFAULT_MODEL.to_string(),
            None,
            Some(config),
        );

        let beta_header = provider.prompt_cache_beta_header_value().unwrap();
        assert!(beta_header.contains("prompt-caching-2024-07-31"));
        assert!(beta_header.contains("extended-cache-ttl-2025-04-11"));
    }

    #[test]
    fn supports_reasoning_effort_for_reasoning_models() {
        let provider = AnthropicProvider::new("test-key".to_string());

        assert!(provider.supports_reasoning_effort("claude-3-7-sonnet-20250219"));
        assert!(provider.supports_reasoning_effort("claude-sonnet-4-5-20250929"));
        assert!(!provider.supports_reasoning_effort("claude-3-5-sonnet-20241022"));
    }
}
