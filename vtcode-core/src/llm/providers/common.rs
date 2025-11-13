use crate::config::core::{PromptCachingConfig, ProviderPromptCachingConfig};
use reqwest::Client as HttpClient;

/// Generic builder for LLM providers with common initialization logic
///
/// This builder eliminates duplication across all provider implementations by
/// centralizing common patterns like HTTP client creation, base URL resolution,
/// and prompt cache settings extraction.
///
/// # Example
///
/// ```ignore
/// let builder = ProviderBuilder::new("my-api-key", "default-model")
///     .with_base_url("https://api.example.com", Some("EXAMPLE_BASE_URL"))
///     .with_prompt_cache(cache_config, |providers| &providers.openai);
///
/// let provider = MyProvider {
///     api_key: builder.api_key,
///     http_client: builder.http_client,
///     base_url: builder.base_url,
///     model: builder.model,
///     prompt_cache_enabled: builder.prompt_cache_enabled,
///     ...
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
            http_client: crate::http_client::shared_client().clone(),
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

    /// Use a custom HTTP client instead of the shared one
    pub fn with_http_client(mut self, client: HttpClient) -> Self {
        self.http_client = client;
        self
    }

    /// Build the provider with the configured values
    pub fn build(self) -> Self {
        self
    }
}

/// Helper function to create a provider builder from common config parameters
///
/// # Arguments
///
/// * `api_key` - Optional API key (falls back to empty string)
/// * `model` - Optional model (falls back to default_model)
/// * `base_url` - Optional base URL override
/// * `prompt_cache` - Optional prompt caching config
/// * `default_model` - Default model to use if none provided
/// * `default_base_url` - Default API base URL
pub fn build_provider_from_config<T: Clone + Default, SelectFn, EnabledFn>(
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    prompt_cache: Option<PromptCachingConfig>,
    default_model: &str,
    default_base_url: &str,
    env_var_name: Option<&str>,
    select_settings: SelectFn,
    enabled: EnabledFn,
) -> ProviderBuilder<T>
where
    SelectFn: Fn(&ProviderPromptCachingConfig) -> &T,
    EnabledFn: Fn(&PromptCachingConfig, &T) -> bool,
{
    let api_key_value = api_key.unwrap_or_default();
    let model_value = resolve_model(model, default_model);

    ProviderBuilder::new(api_key_value, model_value, default_base_url)
        .with_base_url(base_url, env_var_name)
        .with_prompt_cache(prompt_cache, select_settings, enabled)
}

pub fn resolve_model(model: Option<String>, default_model: &str) -> String {
    model
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_model.to_string())
}

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

pub fn forward_prompt_cache_with_state<PredicateFn>(
    prompt_cache: Option<PromptCachingConfig>,
    predicate: PredicateFn,
    default_enabled: bool,
) -> (bool, Option<PromptCachingConfig>)
where
    PredicateFn: Fn(&PromptCachingConfig) -> bool,
{
    match prompt_cache {
        Some(cfg) => {
            if predicate(&cfg) {
                (true, Some(cfg))
            } else {
                (false, None)
            }
        }
        None => (default_enabled, None),
    }
}
