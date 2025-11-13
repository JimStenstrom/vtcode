//! Mock providers and utilities for validation testing

use async_trait::async_trait;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use vtcode_core::llm::provider::{
    FinishReason, LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream, Message, ToolCall,
    ToolDefinition, Usage,
};

/// Mock provider for testing that records requests and returns predefined responses
#[derive(Clone)]
pub struct MockProvider {
    state: Arc<Mutex<MockProviderState>>,
}

struct MockProviderState {
    responses: VecDeque<Result<LLMResponse, LLMError>>,
    request_log: Vec<LLMRequest>,
    name: String,
}

impl MockProvider {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockProviderState {
                responses: VecDeque::new(),
                request_log: Vec::new(),
                name: name.into(),
            })),
        }
    }

    /// Add an expected successful response
    pub fn expect_response(&self, response: LLMResponse) {
        let mut state = self.state.lock().unwrap();
        state.responses.push_back(Ok(response));
    }

    /// Add an expected error
    pub fn expect_error(&self, error: LLMError) {
        let mut state = self.state.lock().unwrap();
        state.responses.push_back(Err(error));
    }

    /// Get all requests that were made to this provider
    pub fn get_requests(&self) -> Vec<LLMRequest> {
        let state = self.state.lock().unwrap();
        state.request_log.clone()
    }

    /// Clear all recorded requests
    pub fn clear_requests(&self) {
        let mut state = self.state.lock().unwrap();
        state.request_log.clear();
    }

    /// Get the number of requests made
    pub fn request_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.request_log.len()
    }

    /// Get the last request made, if any
    pub fn last_request(&self) -> Option<LLMRequest> {
        let state = self.state.lock().unwrap();
        state.request_log.last().cloned()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        &self.state.lock().unwrap().name
    }

    fn model_id(&self) -> &str {
        "mock-model"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["mock-model".to_string()]
    }

    fn validate_request(&self, _request: &LLMRequest) -> Result<(), LLMError> {
        Ok(())
    }

    async fn send(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let mut state = self.state.lock().unwrap();
        state.request_log.push(request.clone());

        state
            .responses
            .pop_front()
            .unwrap_or_else(|| Err(LLMError::Internal("No mock response configured".into())))
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError> {
        // For now, just call send and convert to a single-item stream
        let response = self.send(request).await?;
        Err(LLMError::Internal(
            "Mock streaming not implemented yet".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_records_requests() {
        let mock = MockProvider::new("test-provider");

        let response = LLMResponse {
            content: "Hello!".to_string(),
            model: "mock-model".to_string(),
            stop_reason: FinishReason::EndTurn,
            tool_calls: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
                total_tokens: 15,
            },
            reasoning: None,
        };

        mock.expect_response(response.clone());

        let request = LLMRequest {
            messages: vec![Message::user("Hello".to_string())],
            system_prompt: None,
            tools: None,
            model: "mock-model".to_string(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let result = mock.send(request.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello!");

        assert_eq!(mock.request_count(), 1);
        assert_eq!(mock.last_request().unwrap().messages[0].content.as_text(), "Hello");
    }

    #[tokio::test]
    async fn test_mock_provider_returns_errors() {
        let mock = MockProvider::new("test-provider");
        mock.expect_error(LLMError::RateLimit("Rate limit exceeded".into()));

        let request = LLMRequest {
            messages: vec![Message::user("Hello".to_string())],
            system_prompt: None,
            tools: None,
            model: "mock-model".to_string(),
            max_tokens: None,
            temperature: None,
            stream: false,
            tool_choice: None,
            parallel_tool_calls: None,
            parallel_tool_config: None,
            reasoning_effort: None,
        };

        let result = mock.send(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LLMError::RateLimit(_)));
    }
}
