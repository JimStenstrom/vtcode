//! Utility functions and assertions for validation testing

use vtcode_core::llm::provider::{FinishReason, LLMError, LLMResponse, ToolCall};

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that two LLMResponse objects are equal
pub fn assert_llm_response_eq(actual: &LLMResponse, expected: &LLMResponse) {
    assert_eq!(actual.content, expected.content, "Content mismatch");
    assert_eq!(actual.model, expected.model, "Model mismatch");
    assert_eq!(
        actual.stop_reason, expected.stop_reason,
        "Stop reason mismatch"
    );

    match (&actual.tool_calls, &expected.tool_calls) {
        (None, None) => {}
        (Some(actual_calls), Some(expected_calls)) => {
            assert_eq!(
                actual_calls.len(),
                expected_calls.len(),
                "Tool calls count mismatch"
            );
            for (i, (actual_call, expected_call)) in
                actual_calls.iter().zip(expected_calls.iter()).enumerate()
            {
                assert_tool_call_eq(actual_call, expected_call, i);
            }
        }
        _ => panic!("Tool calls presence mismatch"),
    }

    assert_eq!(
        actual.usage.input_tokens, expected.usage.input_tokens,
        "Input tokens mismatch"
    );
    assert_eq!(
        actual.usage.output_tokens, expected.usage.output_tokens,
        "Output tokens mismatch"
    );
}

/// Assert that two ToolCall objects are equal
pub fn assert_tool_call_eq(actual: &ToolCall, expected: &ToolCall, index: usize) {
    assert_eq!(
        actual.id, expected.id,
        "Tool call {} ID mismatch",
        index
    );
    assert_eq!(
        actual.name, expected.name,
        "Tool call {} name mismatch",
        index
    );
    assert_eq!(
        actual.input, expected.input,
        "Tool call {} input mismatch",
        index
    );
}

/// Assert that a result is an error of a specific type
pub fn assert_provider_error<T>(result: Result<T, LLMError>, expected_error_type: &str) {
    match result {
        Err(error) => {
            let error_str = error.to_string();
            assert!(
                error_str.contains(expected_error_type),
                "Expected error containing '{}', got: {}",
                expected_error_type,
                error_str
            );
        }
        Ok(_) => panic!(
            "Expected error containing '{}', but got success",
            expected_error_type
        ),
    }
}

/// Assert that a result is a rate limit error
pub fn assert_rate_limit_error<T>(result: Result<T, LLMError>) {
    assert!(
        matches!(result, Err(LLMError::RateLimit(_))),
        "Expected RateLimit error, got: {:?}",
        result
    );
}

/// Assert that a result is an authentication error
pub fn assert_authentication_error<T>(result: Result<T, LLMError>) {
    assert!(
        matches!(result, Err(LLMError::Authentication(_))),
        "Expected Authentication error, got: {:?}",
        result
    );
}

/// Assert that a result is an invalid request error
pub fn assert_invalid_request_error<T>(result: Result<T, LLMError>) {
    assert!(
        matches!(result, Err(LLMError::InvalidRequest(_))),
        "Expected InvalidRequest error, got: {:?}",
        result
    );
}

/// Assert that a result is a network error
pub fn assert_network_error<T>(result: Result<T, LLMError>) {
    assert!(
        matches!(result, Err(LLMError::Network(_))),
        "Expected Network error, got: {:?}",
        result
    );
}

// ============================================================================
// Response Validation Helpers
// ============================================================================

/// Check if a response has non-empty content
pub fn has_content(response: &LLMResponse) -> bool {
    !response.content.is_empty()
}

/// Check if a response has tool calls
pub fn has_tool_calls(response: &LLMResponse) -> bool {
    response
        .tool_calls
        .as_ref()
        .map(|calls| !calls.is_empty())
        .unwrap_or(false)
}

/// Check if a response has reasoning
pub fn has_reasoning(response: &LLMResponse) -> bool {
    response
        .reasoning
        .as_ref()
        .map(|r| !r.is_empty())
        .unwrap_or(false)
}

/// Check if a response stopped due to tool use
pub fn stopped_for_tool_use(response: &LLMResponse) -> bool {
    matches!(response.stop_reason, FinishReason::ToolUse)
}

/// Check if a response stopped due to max tokens
pub fn stopped_for_max_tokens(response: &LLMResponse) -> bool {
    matches!(response.stop_reason, FinishReason::MaxTokens)
}

/// Check if a response stopped naturally
pub fn stopped_naturally(response: &LLMResponse) -> bool {
    matches!(response.stop_reason, FinishReason::EndTurn)
}

// ============================================================================
// Test Data Validation
// ============================================================================

/// Validate that usage tokens are reasonable
pub fn validate_usage(response: &LLMResponse) -> bool {
    let usage = &response.usage;
    usage.input_tokens > 0
        && usage.output_tokens > 0
        && usage.total_tokens == usage.input_tokens + usage.output_tokens
}

/// Validate that a response is well-formed
pub fn validate_response(response: &LLMResponse) -> Result<(), String> {
    // Check that we have either content or tool calls
    if response.content.is_empty() && !has_tool_calls(response) {
        return Err("Response has neither content nor tool calls".to_string());
    }

    // Check usage tokens
    if !validate_usage(response) {
        return Err("Invalid usage token counts".to_string());
    }

    // Check that tool use stop reason matches presence of tool calls
    if stopped_for_tool_use(response) && !has_tool_calls(response) {
        return Err("Stop reason is ToolUse but no tool calls present".to_string());
    }

    Ok(())
}

// ============================================================================
// Performance Measurement Helpers
// ============================================================================

/// Measure the time taken by an async operation in milliseconds
pub async fn measure_async<F, T>(f: F) -> (T, u128)
where
    F: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f.await;
    let duration = start.elapsed().as_millis();
    (result, duration)
}

/// Measure the time taken by a sync operation in milliseconds
pub fn measure_sync<F, T>(f: F) -> (T, u128)
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed().as_millis();
    (result, duration)
}

// ============================================================================
// Mock Network Helpers
// ============================================================================

#[cfg(feature = "test-utils")]
pub mod network {
    use mockito::{Matcher, ServerGuard};

    /// Set up a mock HTTP server
    pub fn setup_mock_server() -> ServerGuard {
        mockito::Server::new()
    }

    /// Mock an Anthropic API endpoint
    pub fn mock_anthropic_endpoint(
        server: &mut ServerGuard,
        response_body: &str,
        status_code: usize,
    ) -> mockito::Mock {
        server
            .mock("POST", "/v1/messages")
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    /// Mock an OpenAI API endpoint
    pub fn mock_openai_endpoint(
        server: &mut ServerGuard,
        response_body: &str,
        status_code: usize,
    ) -> mockito::Mock {
        server
            .mock("POST", "/v1/chat/completions")
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    /// Mock a Gemini API endpoint
    pub fn mock_gemini_endpoint(
        server: &mut ServerGuard,
        model: &str,
        response_body: &str,
        status_code: usize,
    ) -> mockito::Mock {
        server
            .mock(
                "POST",
                Matcher::Regex(format!(r"/v1beta/models/{}:.*", model)),
            )
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }
}

// ============================================================================
// Test Statistics
// ============================================================================

/// Collect statistics about test runs
#[derive(Debug, Default)]
pub struct TestStatistics {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl TestStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_pass(&mut self) {
        self.total += 1;
        self.passed += 1;
    }

    pub fn record_fail(&mut self) {
        self.total += 1;
        self.failed += 1;
    }

    pub fn record_skip(&mut self) {
        self.total += 1;
        self.skipped += 1;
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Total: {}, Passed: {}, Failed: {}, Skipped: {}, Pass Rate: {:.2}%",
            self.total,
            self.passed,
            self.failed,
            self.skipped,
            self.pass_rate()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::fixtures::*;

    #[test]
    fn test_response_validation() {
        let valid_response = simple_response();
        assert!(validate_response(&valid_response).is_ok());

        let tool_response = tool_response();
        assert!(validate_response(&tool_response).is_ok());
    }

    #[test]
    fn test_has_content() {
        let response = simple_response();
        assert!(has_content(&response));

        let tool_resp = tool_response();
        assert!(!has_content(&tool_resp));
    }

    #[test]
    fn test_has_tool_calls() {
        let response = simple_response();
        assert!(!has_tool_calls(&response));

        let tool_resp = tool_response();
        assert!(has_tool_calls(&tool_resp));
    }

    #[test]
    fn test_stop_reason_checks() {
        let response = simple_response();
        assert!(stopped_naturally(&response));
        assert!(!stopped_for_tool_use(&response));

        let tool_resp = tool_response();
        assert!(stopped_for_tool_use(&tool_resp));
        assert!(!stopped_naturally(&tool_resp));

        let max_tokens_resp = max_tokens_response();
        assert!(stopped_for_max_tokens(&max_tokens_resp));
    }

    #[test]
    fn test_statistics() {
        let mut stats = TestStatistics::new();
        stats.record_pass();
        stats.record_pass();
        stats.record_fail();

        assert_eq!(stats.total, 3);
        assert_eq!(stats.passed, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.pass_rate(), 66.666666666666);
    }

    #[tokio::test]
    async fn test_measure_async() {
        let (result, duration) = measure_async(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        })
        .await;

        assert_eq!(result, 42);
        assert!(duration >= 10);
    }

    #[test]
    fn test_measure_sync() {
        let (result, duration) = measure_sync(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= 10);
    }
}
