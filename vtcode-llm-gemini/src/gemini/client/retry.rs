use std::time::Duration;
use vtcode_config::constants::retry;

/// Retry configuration for streaming operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: retry::MAX_RETRY_ATTEMPTS,
            initial_delay: retry::initial_backoff(),
            max_delay: retry::max_backoff(),
            backoff_multiplier: retry::BACKOFF_MULTIPLIER as f64,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection".to_string(),
                "rate_limit".to_string(),
                "server_error".to_string(),
                "network".to_string(),
            ],
        }
    }
}
