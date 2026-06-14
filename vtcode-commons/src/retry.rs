//! Minimal shared retry policy for wire-level HTTP clients.
//!
//! This module provides a lightweight [`RetryPolicy`] that classifies errors
//! into retryable vs non-retryable categories. It is intentionally simpler
//! than the full retry system in `vtcode-core` (which adds jitter, multipliers,
//! tool-aware timeouts, and `VtCodeError` integration). Wire clients that only
//! need "should I retry this HTTP call?" use this shared policy; richer retry
//! loops keep their own domain-specific version.

use crate::error_category::{ErrorCategory, classify_anyhow_error};

/// Lightweight retry policy for HTTP wire clients.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries (not counting the initial attempt).
    pub max_retries: u32,
    /// Base delay between retries in milliseconds.
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30_000,
        }
    }
}

/// Result of classifying a failure for retry handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryDecision {
    /// Whether the operation should be retried.
    pub retryable: bool,
    /// The error category determined during classification.
    pub category: ErrorCategory,
}

impl RetryPolicy {
    /// Classify an `anyhow::Error` for retry eligibility.
    ///
    /// Uses the shared [`classify_anyhow_error`] classifier from
    /// [`crate::error_category`].
    pub fn classify_anyhow(&self, error: &anyhow::Error) -> RetryDecision {
        let category = classify_anyhow_error(error);
        RetryDecision {
            retryable: category.is_retryable(),
            category,
        }
    }

    /// Classify an HTTP status code for retry eligibility.
    pub fn classify_status(&self, status: u16) -> RetryDecision {
        let category = match status {
            429 => ErrorCategory::RateLimit,
            500 | 502 | 504 => ErrorCategory::Network,
            503 => ErrorCategory::ServiceUnavailable,
            401 | 403 => ErrorCategory::Authentication,
            _ => ErrorCategory::ExecutionError,
        };
        RetryDecision {
            retryable: category.is_retryable(),
            category,
        }
    }

    /// Compute the backoff delay for a given attempt index (0-based).
    ///
    /// Returns the delay in milliseconds, capped at `max_delay_ms`.
    pub fn delay_ms_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self.base_delay_ms.saturating_mul(1u64 << attempt.min(16));
        delay.min(self.max_delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_has_reasonable_values() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay_ms, 1000);
        assert_eq!(policy.max_delay_ms, 30_000);
    }

    #[test]
    fn classify_status_rate_limit() {
        let policy = RetryPolicy::default();
        let decision = policy.classify_status(429);
        assert!(decision.retryable);
        assert_eq!(decision.category, ErrorCategory::RateLimit);
    }

    #[test]
    fn classify_status_server_error() {
        let policy = RetryPolicy::default();
        let decision = policy.classify_status(503);
        assert!(decision.retryable);
        assert_eq!(decision.category, ErrorCategory::ServiceUnavailable);
    }

    #[test]
    fn classify_status_auth_not_retryable() {
        let policy = RetryPolicy::default();
        let decision = policy.classify_status(401);
        assert!(!decision.retryable);
        assert_eq!(decision.category, ErrorCategory::Authentication);
    }

    #[test]
    fn classify_anyhow_network_error() {
        let policy = RetryPolicy::default();
        let err = anyhow::anyhow!("connection refused");
        let decision = policy.classify_anyhow(&err);
        assert!(decision.retryable);
    }

    #[test]
    fn delay_capped_at_max() {
        let policy = RetryPolicy {
            max_retries: 10,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
        };
        assert_eq!(policy.delay_ms_for_attempt(0), 1000);
        assert_eq!(policy.delay_ms_for_attempt(1), 2000);
        assert_eq!(policy.delay_ms_for_attempt(2), 4000);
        assert_eq!(policy.delay_ms_for_attempt(3), 5000); // capped
        assert_eq!(policy.delay_ms_for_attempt(10), 5000); // still capped
    }
}
