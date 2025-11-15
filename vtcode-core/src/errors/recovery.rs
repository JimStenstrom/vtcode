//! Error recovery strategies
//!
//! This module provides utilities for graceful error recovery and degradation.
//! These patterns help the application continue functioning even when
//! non-critical operations fail.
//!
//! # Examples
//!
//! ## Fallback Values
//!
//! ```rust,ignore
//! use crate::errors::recovery::try_with_fallback;
//!
//! let cache = try_with_fallback(
//!     || load_cache_from_disk(),
//!     Cache::default(),
//!     "Failed to load cache, using empty cache",
//! );
//! ```
//!
//! ## Retry with Backoff
//!
//! ```rust,ignore
//! use crate::errors::recovery::retry_with_backoff;
//!
//! let response = retry_with_backoff(
//!     || api_client.fetch_data(),
//!     3,  // max attempts
//!     1000,  // initial delay in ms
//! ).await?;
//! ```
//!
//! ## Try Alternatives
//!
//! ```rust,ignore
//! use crate::errors::recovery::try_alternatives;
//!
//! let config = try_alternatives(vec![
//!     ("project config", Box::new(|| load_project_config())),
//!     ("user config", Box::new(|| load_user_config())),
//!     ("default config", Box::new(|| Ok(Config::default()))),
//! ])?;
//! ```

use anyhow::{anyhow, Result};
use tracing::warn;

/// Recoverable operation - try operation, log failure, return default
///
/// This is useful for non-critical operations where a sensible default exists.
///
/// # Arguments
///
/// * `operation` - The operation to attempt
/// * `fallback` - The default value to use if operation fails
/// * `context` - Description of what failed (for logging)
///
/// # Example
///
/// ```rust,ignore
/// use crate::errors::recovery::try_with_fallback;
///
/// let theme = try_with_fallback(
///     || load_user_theme(),
///     Theme::default(),
///     "Failed to load user theme preferences",
/// );
/// ```
pub fn try_with_fallback<T, F>(operation: F, fallback: T, context: &str) -> T
where
    F: FnOnce() -> Result<T>,
{
    match operation() {
        Ok(value) => value,
        Err(e) => {
            warn!("{}: {}", context, e);
            fallback
        }
    }
}

/// Retry operation with exponential backoff
///
/// Useful for transient failures like network errors or rate limiting.
///
/// # Arguments
///
/// * `operation` - The async operation to retry
/// * `max_attempts` - Maximum number of attempts (must be at least 1)
/// * `initial_delay_ms` - Initial delay in milliseconds (doubles each retry)
///
/// # Returns
///
/// Returns the successful result or the last error if all attempts fail.
///
/// # Example
///
/// ```rust,ignore
/// use crate::errors::recovery::retry_with_backoff;
///
/// let data = retry_with_backoff(
///     || async { api_client.fetch_data().await },
///     3,  // max attempts
///     1000,  // start with 1 second delay
/// ).await?;
/// ```
pub async fn retry_with_backoff<T, F, Fut>(
    mut operation: F,
    max_attempts: u32,
    initial_delay_ms: u64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if max_attempts == 0 {
        return Err(anyhow!("max_attempts must be at least 1"));
    }

    let mut attempt = 0;
    let mut delay = initial_delay_ms;

    loop {
        attempt += 1;
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt >= max_attempts => {
                warn!("All {} attempts failed, giving up", max_attempts);
                return Err(e);
            }
            Err(e) => {
                warn!(
                    "Attempt {}/{} failed: {}. Retrying in {}ms...",
                    attempt, max_attempts, e, delay
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

/// Try multiple operations, return first success
///
/// Useful when there are multiple ways to accomplish a task (e.g., multiple
/// config file locations, multiple API endpoints).
///
/// # Arguments
///
/// * `operations` - Vector of (name, operation) pairs to try in order
///
/// # Returns
///
/// Returns the first successful result, or the last error if all fail.
///
/// # Example
///
/// ```rust,ignore
/// use crate::errors::recovery::try_alternatives;
///
/// let config = try_alternatives(vec![
///     ("project", Box::new(|| load_from_path(".vtcode/config.toml"))),
///     ("home", Box::new(|| load_from_path("~/.vtcode/config.toml"))),
///     ("system", Box::new(|| load_from_path("/etc/vtcode/config.toml"))),
/// ])?;
/// ```
pub fn try_alternatives<T>(
    operations: Vec<(&str, Box<dyn FnOnce() -> Result<T>>)>,
) -> Result<T> {
    if operations.is_empty() {
        return Err(anyhow!("No alternatives provided"));
    }

    let mut last_error = None;

    for (name, operation) in operations {
        match operation() {
            Ok(value) => {
                if last_error.is_some() {
                    warn!("Alternative '{}' succeeded after previous failures", name);
                }
                return Ok(value);
            }
            Err(e) => {
                warn!("Alternative '{}' failed: {}", name, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("All alternatives failed")))
}

/// Try an operation with a timeout
///
/// Useful for operations that might hang indefinitely.
///
/// # Arguments
///
/// * `operation` - The async operation to execute
/// * `timeout_ms` - Timeout in milliseconds
///
/// # Returns
///
/// Returns the result or an error if the timeout is exceeded.
///
/// # Example
///
/// ```rust,ignore
/// use crate::errors::recovery::try_with_timeout;
///
/// let result = try_with_timeout(
///     || async { slow_operation().await },
///     5000,  // 5 second timeout
/// ).await?;
/// ```
pub async fn try_with_timeout<T, F, Fut>(operation: F, timeout_ms: u64) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let timeout = tokio::time::Duration::from_millis(timeout_ms);
    match tokio::time::timeout(timeout, operation()).await {
        Ok(result) => result,
        Err(_) => Err(anyhow!("Operation timed out after {}ms", timeout_ms)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_try_with_fallback_success() {
        let result = try_with_fallback(|| Ok(42), 0, "test context");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_try_with_fallback_failure() {
        let result = try_with_fallback(
            || Err(anyhow!("test error")) as Result<i32>,
            99,
            "test context",
        );
        assert_eq!(result, 99);
    }

    #[tokio::test]
    async fn test_retry_backoff_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    let attempt = c.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(anyhow!("fail"))
                    } else {
                        Ok("success")
                    }
                }
            },
            3,
            10,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_backoff_all_fail() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(anyhow!("always fails"))
                }
            },
            3,
            10,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_backoff_zero_attempts() {
        let result = retry_with_backoff(|| async { Ok::<_, anyhow::Error>(()) }, 0, 10).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 1"));
    }

    #[test]
    fn test_try_alternatives_first_success() {
        let result = try_alternatives(vec![
            ("first", Box::new(|| Ok(1))),
            ("second", Box::new(|| Ok(2))),
        ]);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_try_alternatives_second_success() {
        let result = try_alternatives(vec![
            ("first", Box::new(|| Err(anyhow!("fail")))),
            ("second", Box::new(|| Ok(2))),
        ]);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_try_alternatives_all_fail() {
        let result: Result<i32> = try_alternatives(vec![
            ("first", Box::new(|| Err(anyhow!("fail 1")))),
            ("second", Box::new(|| Err(anyhow!("fail 2")))),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_alternatives_empty() {
        let result: Result<i32> = try_alternatives(vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No alternatives"));
    }

    #[tokio::test]
    async fn test_try_with_timeout_success() {
        let result = try_with_timeout(|| async { Ok::<_, anyhow::Error>(42) }, 1000).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_try_with_timeout_exceeds() {
        let result = try_with_timeout(
            || async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok::<_, anyhow::Error>(42)
            },
            50,
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }
}
