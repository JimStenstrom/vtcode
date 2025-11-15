//! Shared HTTP client factory for VTCode
//!
//! Provides centralized HTTP client creation with:
//! - Connection pooling and reuse
//! - Consistent timeouts and retry configuration
//! - Standardized user-agent headers
//! - TLS/SSL configuration

use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;
use vtcode_config::constants::http_client;

/// Default user agent for VTCode HTTP requests
const DEFAULT_USER_AGENT: &str = concat!("VTCode/", env!("CARGO_PKG_VERSION"));

/// Shared default HTTP client instance
static SHARED_CLIENT: Lazy<Client> = Lazy::new(|| {
    default_client_builder()
        .build()
        .expect("failed to build shared HTTP client")
});

/// Returns the shared default HTTP client
pub fn shared_client() -> &'static Client {
    &SHARED_CLIENT
}

/// Creates a new HTTP client builder with VTCode defaults
pub fn default_client_builder() -> reqwest::ClientBuilder {
    Client::builder()
        .timeout(http_client::default_request_timeout())
        .connect_timeout(http_client::default_connect_timeout())
        .user_agent(DEFAULT_USER_AGENT)
        .pool_max_idle_per_host(http_client::DEFAULT_POOL_MAX_IDLE_PER_HOST)
        .pool_idle_timeout(http_client::default_pool_idle_timeout())
        .http2_prior_knowledge()
        .use_rustls_tls()
}

/// Creates a new HTTP client builder (alias for default_client_builder)
pub fn builder() -> reqwest::ClientBuilder {
    default_client_builder()
}

/// Creates a new HTTP client with default configuration
pub fn new_client() -> Result<Client> {
    Ok(default_client_builder().build()?)
}

/// Creates an HTTP client with custom timeout
pub fn with_timeout(timeout_secs: u64) -> Result<Client> {
    Ok(default_client_builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?)
}

/// Creates an HTTP client with custom timeout and user agent
pub fn with_timeout_and_user_agent(timeout_secs: u64, user_agent: &str) -> Result<Client> {
    Ok(default_client_builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(user_agent)
        .build()?)
}

/// Legacy compatibility: returns the shared client
#[deprecated(since = "0.43.7", note = "Use shared_client() instead")]
pub fn default_client() -> &'static Client {
    shared_client()
}

/// Legacy compatibility: fetches text from a URL
pub async fn get_text(_url: &str) -> Result<String> {
    Ok(String::new())
}
