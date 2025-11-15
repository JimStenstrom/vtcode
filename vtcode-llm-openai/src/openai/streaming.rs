//! OpenAI streaming utilities

use async_stream::try_stream;
use futures::StreamExt;
use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use vtcode_llm_types::{
    FinishReason, LLMError, LLMRequest, LLMResponse, LLMStream, LLMStreamEvent,
};

use super::request::{serialize_messages, serialize_tools};

/// Create a streaming request to OpenAI
pub async fn stream_response(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request: LLMRequest,
) -> Result<LLMStream, LLMError> {
    let messages = serialize_messages(&request.messages, request.system_prompt.as_ref());

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

    if let Some(tools) = &request.tools
        && let Some(tools_json) = serialize_tools(tools) {
            body["tools"] = json!(tools_json);
        }

    let url = format!("{}/chat/completions", base_url);

    let builder = http_client.post(&url);
    let builder = if api_key.trim().is_empty() {
        builder
    } else {
        builder.bearer_auth(api_key)
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
                LLMError::NetworkError(format!("Stream request failed: {}", e))
            }
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(LLMError::ApiError(error_text));
    }

    let mut byte_stream = response.bytes_stream();

    let stream = try_stream! {
        let mut buffer = String::new();
        let mut accumulated_content = String::new();

        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk.map_err(|e| LLMError::Provider(format!("Stream error: {}", e)))?;
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
                            content: Some(accumulated_content.clone()),
                            usage: None,
                            reasoning: None,
                            tool_calls: None,
                            finish_reason: FinishReason::Stop,
                            reasoning_details: None,
                        }
                    };
                    break;
                }

                if let Ok(json_value) = serde_json::from_str::<Value>(data)
                    && let Some(choices) = json_value.get("choices").and_then(|v| v.as_array())
                        && let Some(choice) = choices.first()
                            && let Some(delta) = choice.get("delta")
                                && let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                                    accumulated_content.push_str(content);
                                    yield LLMStreamEvent::Token {
                                        delta: content.to_string(),
                                    };
                                }
            }
        }
    };

    Ok(Box::pin(stream))
}
