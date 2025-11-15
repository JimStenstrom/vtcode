//! Configuration utilities for LLM providers

use reqwest::Client as HttpClient;
use std::time::Duration;
use vtcode_config::core::{PromptCachingConfig, ProviderPromptCachingConfig};

/// Create a standard HTTP client with default timeout
pub fn create_http_client() -> HttpClient {
    HttpClient::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| HttpClient::new())
}

/// Resolve model name, falling back to default if none provided or empty
///
/// # Example
///
/// ```
/// use vtcode_llm_common::resolve_model;
///
/// let model = resolve_model(Some("gpt-4".to_string()), "gpt-3.5");
/// assert_eq!(model, "gpt-4");
///
/// let model = resolve_model(None, "gpt-3.5");
/// assert_eq!(model, "gpt-3.5");
/// ```
pub fn resolve_model(model: Option<String>, default_model: &str) -> String {
    model
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_model.to_string())
}

/// Override base URL with config or environment variable
///
/// Priority: config parameter > environment variable > default
///
/// # Example
///
/// ```
/// use vtcode_llm_common::override_base_url;
///
/// let url = override_base_url(
///     "https://api.example.com",
///     Some("https://custom.example.com".to_string()),
///     Some("EXAMPLE_BASE_URL")
/// );
/// assert_eq!(url, "https://custom.example.com");
/// ```
pub fn override_base_url(
    default_base_url: &str,
    base_url: Option<String>,
    env_var_name: Option<&str>,
) -> String {
    if let Some(url) = base_url {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(var_name) = env_var_name {
        if let Ok(value) = std::env::var(var_name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    default_base_url.to_string()
}

/// Extract provider-specific prompt cache settings from global config
pub fn extract_prompt_cache_settings<T, SelectFn, EnabledFn>(
    prompt_cache: Option<PromptCachingConfig>,
    select_settings: SelectFn,
    enabled: EnabledFn,
) -> (bool, T)
where
    T: Clone + Default,
    SelectFn: Fn(&ProviderPromptCachingConfig) -> &T,
    EnabledFn: Fn(&PromptCachingConfig, &T) -> bool,
{
    if let Some(cfg) = prompt_cache {
        let provider_settings = select_settings(&cfg.providers).clone();
        let is_enabled = enabled(&cfg, &provider_settings);
        (is_enabled, provider_settings)
    } else {
        (false, T::default())
    }
}

/// Generic builder for LLM providers with common initialization logic
///
/// This builder eliminates duplication across all provider implementations by
/// centralizing common patterns like HTTP client creation, base URL resolution,
/// and prompt cache settings extraction.
///
/// # Example
///
/// ```ignore
/// use vtcode_llm_common::ProviderBuilder;
///
/// let builder = ProviderBuilder::new("my-api-key", "default-model", "https://api.example.com")
///     .with_base_url(Some("https://custom.example.com".to_string()), Some("EXAMPLE_BASE_URL"));
///
/// let provider = MyProvider {
///     api_key: builder.api_key,
///     http_client: builder.http_client,
///     base_url: builder.base_url,
///     model: builder.model,
///     prompt_cache_enabled: builder.prompt_cache_enabled,
/// };
/// ```
pub struct ProviderBuilder<T: Clone + Default> {
    pub api_key: String,
    pub http_client: HttpClient,
    pub base_url: String,
    pub model: String,
    pub prompt_cache_enabled: bool,
    pub prompt_cache_settings: T,
}

impl<T: Clone + Default> ProviderBuilder<T> {
    /// Create a new provider builder with required parameters
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key for the provider
    /// * `model` - Model identifier
    /// * `default_base_url` - Default API base URL
    pub fn new(api_key: String, model: String, default_base_url: &str) -> Self {
        Self {
            api_key,
            http_client: create_http_client(),
            base_url: default_base_url.to_string(),
            model,
            prompt_cache_enabled: false,
            prompt_cache_settings: T::default(),
        }
    }

    /// Override the base URL from config or environment variable
    ///
    /// # Arguments
    ///
    /// * `base_url_override` - Optional base URL from configuration
    /// * `env_var_name` - Optional environment variable name to check
    pub fn with_base_url(
        mut self,
        base_url_override: Option<String>,
        env_var_name: Option<&str>,
    ) -> Self {
        self.base_url = override_base_url(&self.base_url, base_url_override, env_var_name);
        self
    }

    /// Extract and configure prompt cache settings
    ///
    /// # Arguments
    ///
    /// * `prompt_cache` - Optional prompt caching configuration
    /// * `select_settings` - Function to extract provider-specific settings
    /// * `enabled` - Function to determine if caching is enabled
    pub fn with_prompt_cache<SelectFn, EnabledFn>(
        mut self,
        prompt_cache: Option<PromptCachingConfig>,
        select_settings: SelectFn,
        enabled: EnabledFn,
    ) -> Self
    where
        SelectFn: Fn(&ProviderPromptCachingConfig) -> &T,
        EnabledFn: Fn(&PromptCachingConfig, &T) -> bool,
    {
        let (is_enabled, settings) =
            extract_prompt_cache_settings(prompt_cache, select_settings, enabled);
        self.prompt_cache_enabled = is_enabled;
        self.prompt_cache_settings = settings;
        self
    }

    /// Use a custom HTTP client instead of the default one
    pub fn with_http_client(mut self, client: HttpClient) -> Self {
        self.http_client = client;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_model_with_value() {
        let result = resolve_model(Some("custom-model".to_string()), "default-model");
        assert_eq!(result, "custom-model");
    }

    #[test]
    fn test_resolve_model_with_none() {
        let result = resolve_model(None, "default-model");
        assert_eq!(result, "default-model");
    }

    #[test]
    fn test_resolve_model_with_empty() {
        let result = resolve_model(Some("  ".to_string()), "default-model");
        assert_eq!(result, "default-model");
    }

    #[test]
    fn test_override_base_url_with_config() {
        let result = override_base_url(
            "https://default.com",
            Some("https://custom.com".to_string()),
            None,
        );
        assert_eq!(result, "https://custom.com");
    }

    #[test]
    fn test_override_base_url_fallback_to_default() {
        let result = override_base_url("https://default.com", None, None);
        assert_eq!(result, "https://default.com");
    }

    #[test]
    fn test_create_http_client() {
        let client = create_http_client();
        // Just verify it doesn't panic
        assert!(client.get("https://example.com").build().is_ok());
    }
}
