//! Error handling for Gemini provider
//!
//! This module provides error mapping and handling utilities for converting
//! Gemini streaming errors to LLMError types.

use crate::gemini::streaming::StreamingError;
use vtcode_llm_types::LLMError;

/// Map Gemini streaming errors to LLMError
pub fn map_streaming_error(error: StreamingError) -> LLMError {
    match error {
        StreamingError::NetworkError { message, .. } => {
            let formatted = format!("Gemini: Network error: {}", message);
            LLMError::NetworkError(formatted)
        }
        StreamingError::ApiError {
            status_code,
            message,
            ..
        } => {
            if status_code == 401 || status_code == 403 {
                let formatted = format!("Gemini: HTTP {}: {}", status_code, message);
                LLMError::AuthenticationError(formatted)
            } else if status_code == 429 {
                LLMError::RateLimit
            } else {
                let formatted = format!("Gemini: API error ({}): {}", status_code, message);
                LLMError::Provider(formatted)
            }
        }
        StreamingError::ParseError { message, .. } => {
            let formatted = format!("Gemini: Parse error: {}", message);
            LLMError::Provider(formatted)
        }
        StreamingError::TimeoutError {
            operation,
            duration,
        } => {
            let formatted = format!(
                "Gemini: Streaming timeout during {} after {:?}",
                operation, duration
            );
            LLMError::NetworkError(formatted)
        }
        StreamingError::ContentError { message } => {
            let formatted = format!("Gemini: Content error: {}", message);
            LLMError::Provider(formatted)
        }
        StreamingError::StreamingError { message, .. } => {
            let formatted = format!("Gemini: Streaming error: {}", message);
            LLMError::Provider(formatted)
        }
    }
}

/// Handle HTTP response errors for non-streaming requests
pub fn handle_http_error(status: u16, error_text: String) -> LLMError {
    // Handle authentication errors
    if status == 401 || status == 403 {
        let formatted_error = format!(
            "Gemini: Authentication failed: {}. Check your GOOGLE_API_KEY or GEMINI_API_KEY environment variable.",
            error_text
        );
        return LLMError::AuthenticationError(formatted_error);
    }

    // Handle rate limit and quota errors
    if status == 429
        || error_text.contains("insufficient_quota")
        || error_text.contains("RESOURCE_EXHAUSTED")
        || error_text.contains("quota")
        || error_text.contains("rate limit")
        || error_text.contains("rateLimitExceeded")
    {
        return LLMError::RateLimit;
    }

    // Handle invalid request errors
    if status == 400 {
        let formatted_error = format!("Gemini: Invalid request: {}", error_text);
        return LLMError::InvalidRequest(formatted_error);
    }

    // Generic error for other cases
    let formatted_error = format!("Gemini: HTTP {}: {}", status, error_text);
    LLMError::Provider(formatted_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_streaming_error_network() {
        let error = StreamingError::NetworkError {
            message: "Connection failed".to_string(),
            is_retryable: true,
        };
        let mapped = map_streaming_error(error);
        match mapped {
            LLMError::NetworkError(msg) => {
                assert!(msg.contains("Connection failed"));
                assert!(msg.starts_with("Gemini:"));
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_map_streaming_error_auth() {
        let error = StreamingError::ApiError {
            status_code: 401,
            message: "Unauthorized".to_string(),
            is_retryable: false,
        };
        let mapped = map_streaming_error(error);
        match mapped {
            LLMError::AuthenticationError(msg) => {
                assert!(msg.contains("401"));
                assert!(msg.contains("Unauthorized"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[test]
    fn test_map_streaming_error_rate_limit() {
        let error = StreamingError::ApiError {
            status_code: 429,
            message: "Too many requests".to_string(),
            is_retryable: true,
        };
        let mapped = map_streaming_error(error);
        match mapped {
            LLMError::RateLimit => {}
            _ => panic!("Expected RateLimit"),
        }
    }

    #[test]
    fn test_handle_http_error_auth() {
        let error = handle_http_error(403, "Forbidden".to_string());
        match error {
            LLMError::AuthenticationError(msg) => {
                assert!(msg.contains("GOOGLE_API_KEY"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[test]
    fn test_handle_http_error_rate_limit() {
        let error = handle_http_error(429, "Rate limit exceeded".to_string());
        match error {
            LLMError::RateLimit => {}
            _ => panic!("Expected RateLimit"),
        }
    }

    #[test]
    fn test_handle_http_error_quota() {
        let error = handle_http_error(400, "insufficient_quota".to_string());
        match error {
            LLMError::RateLimit => {}
            _ => panic!("Expected RateLimit for quota error"),
        }
    }
}
