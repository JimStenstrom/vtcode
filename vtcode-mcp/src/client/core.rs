//! MCP client core initialization and lifecycle.

use super::providers::McpProvider;
use super::tools;
use super::{McpClient, McpToolInfo};
use anyhow::{Result, anyhow};
use mcp_types::{
    ClientCapabilities, ClientCapabilitiesRoots, Implementation, InitializeRequestParams,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use vtcode_config::mcp::{McpAllowListConfig, McpClientConfig, McpProviderConfig, McpTransportConfig};

const ELICITATION_SCHEMA_VALIDATION_FLAG: &str = "schemaValidation";

impl McpClient {
    /// Create a new MCP client from the configuration.
    pub fn new(config: McpClientConfig) -> Self {
        let allowlist = config.allowlist.clone();

        Self {
            config,
            providers: parking_lot::RwLock::new(HashMap::new()),
            allowlist: parking_lot::RwLock::new(allowlist),
            tool_provider_index: parking_lot::RwLock::new(HashMap::new()),
            resource_provider_index: parking_lot::RwLock::new(HashMap::new()),
            prompt_provider_index: parking_lot::RwLock::new(HashMap::new()),
            elicitation_handler: None,
        }
    }

    /// Establish connections to all configured providers and complete the MCP handshake.
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("MCP client is disabled in configuration");
            return Ok(());
        }

        info!(
            "Initializing MCP client with {} configured providers",
            self.config.providers.len()
        );

        let tool_timeout = self.tool_timeout();
        let allowlist_snapshot = self.allowlist.read().clone();

        let mut initialized = HashMap::new();

        for provider_config in &self.config.providers {
            if !provider_config.enabled {
                debug!(
                    "MCP provider '{}' is disabled; skipping",
                    provider_config.name
                );
                continue;
            }

            if matches!(provider_config.transport, McpTransportConfig::Http(_))
                && !self.config.experimental_use_rmcp_client
            {
                warn!(
                    "Skipping MCP HTTP provider '{}' because experimental_use_rmcp_client is disabled",
                    provider_config.name
                );
                continue;
            }

            match McpProvider::connect(provider_config.clone(), self.elicitation_handler.clone())
                .await
            {
                Ok(provider) => {
                    let provider_startup_timeout = self.resolve_startup_timeout(provider_config);
                    if let Err(err) = provider
                        .initialize(
                            self.build_initialize_params(&provider),
                            provider_startup_timeout,
                            tool_timeout,
                            &allowlist_snapshot,
                        )
                        .await
                    {
                        error!(
                            "Failed to initialize MCP provider '{}': {err}",
                            provider_config.name
                        );
                        continue;
                    }

                    if let Err(err) = provider
                        .refresh_tools(&allowlist_snapshot, tool_timeout)
                        .await
                    {
                        warn!(
                            "Failed to fetch tools for provider '{}': {err}",
                            provider_config.name
                        );
                    } else if let Some(cache) = provider.cached_tools().await {
                        self.record_tool_provider(&provider.name, &cache);
                    }

                    initialized.insert(provider.name.clone(), Arc::new(provider));
                    info!(
                        "Successfully initialized MCP provider '{}'",
                        provider_config.name
                    );
                }
                Err(err) => {
                    error!(
                        "Failed to connect to MCP provider '{}': {err}",
                        provider_config.name
                    );
                }
            }
        }

        *self.providers.write() = initialized;
        info!(
            "MCP client initialization complete. Active providers: {}",
            self.providers.read().len()
        );

        Ok(())
    }

    /// Shutdown all active provider connections.
    pub async fn shutdown(&self) -> Result<()> {
        let providers: Vec<Arc<McpProvider>> = {
            let mut guard = self.providers.write();
            let values = guard.values().cloned().collect();
            guard.clear();
            values
        };

        if providers.is_empty() {
            info!("No active MCP connections to shutdown");
            return Ok(());
        }

        info!("Shutting down {} MCP providers", providers.len());
        for provider in providers {
            if let Err(err) = provider.shutdown().await {
                warn!(
                    "Provider '{}' shutdown returned error: {err}",
                    provider.name
                );
            }
        }

        self.tool_provider_index.write().clear();
        self.resource_provider_index.write().clear();
        self.prompt_provider_index.write().clear();
        Ok(())
    }

    /// Refresh the internal allow list at runtime.
    pub fn update_allowlist(&self, allowlist: McpAllowListConfig) {
        *self.allowlist.write() = allowlist;
        self.tool_provider_index.write().clear();
        self.resource_provider_index.write().clear();
        self.prompt_provider_index.write().clear();

        for provider in self.providers.read().values() {
            provider.invalidate_caches();
        }
    }

    /// Current allow list snapshot.
    pub fn current_allowlist(&self) -> McpAllowListConfig {
        self.allowlist.read().clone()
    }

    /// Validate tool arguments based on security configuration.
    pub(crate) fn validate_tool_arguments(&self, _tool_name: &str, args: &Value) -> Result<()> {
        tools::validate_tool_arguments(
            _tool_name,
            args,
            self.config.security.validation.max_argument_size,
            self.config.security.validation.path_traversal_protection,
        )
    }

    pub(crate) fn record_tool_provider(&self, provider: &str, tools: &[McpToolInfo]) {
        let mut index = self.tool_provider_index.write();
        for tool in tools {
            index.insert(tool.name.clone(), provider.to_string());
        }
    }

    pub(crate) fn startup_timeout(&self) -> Option<Duration> {
        match self.config.startup_timeout_seconds {
            Some(0) => None,
            Some(value) => Some(Duration::from_secs(value)),
            None => self.request_timeout(),
        }
    }

    pub(crate) fn resolve_startup_timeout(&self, provider_config: &McpProviderConfig) -> Option<Duration> {
        if let Some(timeout_ms) = provider_config.startup_timeout_ms {
            if timeout_ms == 0 {
                None
            } else {
                Some(Duration::from_millis(timeout_ms))
            }
        } else {
            self.startup_timeout()
        }
    }

    pub(crate) fn tool_timeout(&self) -> Option<Duration> {
        match self.config.tool_timeout_seconds {
            Some(0) => None,
            Some(value) => Some(Duration::from_secs(value)),
            None => self.request_timeout(),
        }
    }

    pub(crate) fn request_timeout(&self) -> Option<Duration> {
        if self.config.request_timeout_seconds == 0 {
            None
        } else {
            Some(Duration::from_secs(self.config.request_timeout_seconds))
        }
    }

    pub(crate) fn build_initialize_params(&self, provider: &McpProvider) -> InitializeRequestParams {
        let mut capabilities = ClientCapabilities::default();
        capabilities.roots = Some(ClientCapabilitiesRoots {
            list_changed: Some(true),
        });

        if self.elicitation_handler.is_some() {
            let mut elicitation_capability = Map::new();
            elicitation_capability.insert(
                ELICITATION_SCHEMA_VALIDATION_FLAG.to_string(),
                Value::Bool(true),
            );
            // Use experimental field for elicitation support until it's in the stable published version
            capabilities
                .experimental
                .insert("elicitation".to_string(), elicitation_capability);
        }

        InitializeRequestParams {
            capabilities,
            client_info: Implementation {
                name: "vtcode".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            protocol_version: provider.protocol_version.clone(),
        }
    }
}

/// Validate MCP configuration settings
pub fn validate_mcp_config(config: &McpClientConfig) -> Result<()> {
    // Validate server configuration if enabled
    if config.server.enabled {
        // Validate port range
        if config.server.port == 0 {
            return Err(anyhow!("Invalid server port: {}", config.server.port));
        }

        // Validate bind address
        if config.server.bind_address.is_empty() {
            return Err(anyhow!("Server bind address cannot be empty"));
        }

        // Validate security settings if auth is enabled
        if config.security.auth_enabled && config.security.api_key_env.is_none() {
            return Err(anyhow!(
                "API key environment variable must be set when auth is enabled"
            ));
        }
    }

    // Validate timeouts
    if let Some(startup_timeout) = config.startup_timeout_seconds {
        if startup_timeout > 300 {
            // Max 5 minutes
            return Err(anyhow!("Startup timeout cannot exceed 300 seconds"));
        }
    }

    if let Some(tool_timeout) = config.tool_timeout_seconds {
        if tool_timeout > 3600 {
            // Max 1 hour
            return Err(anyhow!("Tool timeout cannot exceed 3600 seconds"));
        }
    }

    // Validate provider configurations
    for provider in &config.providers {
        if provider.name.is_empty() {
            return Err(anyhow!("MCP provider name cannot be empty"));
        }

        // Validate max_concurrent_requests
        if provider.max_concurrent_requests == 0 {
            return Err(anyhow!(
                "Max concurrent requests must be greater than 0 for provider '{}'",
                provider.name
            ));
        }
    }

    Ok(())
}
