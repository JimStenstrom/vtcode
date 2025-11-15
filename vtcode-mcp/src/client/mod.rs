//! MCP client for managing multiple Model Context Protocol providers.

pub mod core;
pub mod providers;
pub mod prompts;
pub mod resources;
pub mod tools;

#[cfg(test)]
mod tests;

pub use core::validate_mcp_config;
pub use providers::McpProvider;

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use mcp_types::{PromptArgument, PromptMessage, ReadResourceResultContentsItem};
use parking_lot::RwLock;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use vtcode_config::mcp::{McpAllowListConfig, McpClientConfig};

// Re-export MCP tool types from vtcode-tool-traits
pub use vtcode_tool_traits::{McpClientStatus, McpToolExecutor, McpToolInfo};

// Re-export elicitation action from rmcp
pub use rmcp::model::ElicitationAction;

/// Summary of an MCP resource exposed by a provider.
#[derive(Debug, Clone)]
pub struct McpResourceInfo {
    pub provider: String,
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
}

/// Resource contents fetched from an MCP provider.
#[derive(Debug, Clone)]
pub struct McpResourceData {
    pub provider: String,
    pub uri: String,
    pub contents: Vec<ReadResourceResultContentsItem>,
    pub meta: Map<String, Value>,
}

/// Summary of an MCP prompt exposed by a provider.
#[derive(Debug, Clone)]
pub struct McpPromptInfo {
    pub provider: String,
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

/// Fully rendered MCP prompt ready for use.
#[derive(Debug, Clone)]
pub struct McpPromptDetail {
    pub provider: String,
    pub name: String,
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
    pub meta: Map<String, Value>,
}

/// Request payload for handling elicitation prompts from MCP providers.
#[derive(Debug, Clone)]
pub struct McpElicitationRequest {
    pub message: String,
    pub requested_schema: Value,
}

/// Result returned by an elicitation handler after interacting with the user.
#[derive(Debug, Clone)]
pub struct McpElicitationResponse {
    pub action: ElicitationAction,
    pub content: Option<Value>,
}

/// Callback interface used to resolve elicitation requests from MCP providers.
#[async_trait]
pub trait McpElicitationHandler: Send + Sync {
    async fn handle_elicitation(
        &self,
        provider: &str,
        request: McpElicitationRequest,
    ) -> Result<McpElicitationResponse>;
}

/// High level MCP client responsible for managing multiple providers and
/// enforcing VTCode specific policies like tool allow lists.
pub struct McpClient {
    config: McpClientConfig,
    providers: RwLock<HashMap<String, Arc<McpProvider>>>,
    allowlist: RwLock<McpAllowListConfig>,
    tool_provider_index: RwLock<HashMap<String, String>>,
    resource_provider_index: RwLock<HashMap<String, String>>,
    prompt_provider_index: RwLock<HashMap<String, String>>,
    elicitation_handler: Option<Arc<dyn McpElicitationHandler>>,
}

impl McpClient {
    /// Register a handler used to satisfy elicitation requests from providers.
    pub fn set_elicitation_handler(&mut self, handler: Arc<dyn McpElicitationHandler>) {
        self.elicitation_handler = Some(handler);
    }

    /// Execute a tool call after validating arguments
    pub async fn execute_tool_with_validation(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Result<Value> {
        if !self.config.enabled {
            return Err(anyhow!(
                "MCP support is disabled in the current configuration"
            ));
        }

        self.validate_tool_arguments(tool_name, &args)?;

        let provider = self.resolve_provider_for_tool(tool_name).await?;
        let allowlist_snapshot = self.allowlist.read().clone();
        let result = tools::execute_tool_on_provider(
            &provider,
            tool_name,
            args,
            self.tool_timeout(),
            &allowlist_snapshot,
        )
        .await?;

        tools::format_tool_result(&provider.name, tool_name, result)
    }

    /// Return the provider name serving the given tool if previously cached.
    pub fn provider_for_tool(&self, tool_name: &str) -> Option<String> {
        self.tool_provider_index.read().get(tool_name).cloned()
    }

    /// Return the provider responsible for the given resource URI if known.
    pub fn provider_for_resource(&self, uri: &str) -> Option<String> {
        self.resource_provider_index.read().get(uri).cloned()
    }

    /// Return the provider that exposes the given prompt if known.
    pub fn provider_for_prompt(&self, prompt_name: &str) -> Option<String> {
        self.prompt_provider_index.read().get(prompt_name).cloned()
    }

    /// Execute a tool call on the appropriate provider.
    pub async fn execute_tool(&self, tool_name: &str, args: Value) -> Result<Value> {
        self.execute_tool_with_validation(tool_name, args).await
    }

    /// List all tools from all active providers.
    pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        self.collect_tools(false).await
    }

    /// List all resources exposed by connected MCP providers.
    pub async fn list_resources(&self) -> Result<Vec<McpResourceInfo>> {
        self.collect_resources(false).await
    }

    /// Force refresh and list resources from providers.
    pub async fn refresh_resources(&self) -> Result<Vec<McpResourceInfo>> {
        self.collect_resources(true).await
    }

    /// List all prompts advertised by connected MCP providers.
    pub async fn list_prompts(&self) -> Result<Vec<McpPromptInfo>> {
        self.collect_prompts(false).await
    }

    /// Force refresh and list prompts from providers.
    pub async fn refresh_prompts(&self) -> Result<Vec<McpPromptInfo>> {
        self.collect_prompts(true).await
    }

    /// Read a single resource from its originating provider.
    pub async fn read_resource(&self, uri: &str) -> Result<McpResourceData> {
        let provider = self.resolve_provider_for_resource(uri).await?;
        let provider_name = provider.name.clone();
        let allowlist_snapshot = self.allowlist.read().clone();
        let data = resources::fetch_resource_from_provider(
            &provider,
            uri,
            self.request_timeout(),
            &allowlist_snapshot,
        )
        .await?;
        self.resource_provider_index
            .write()
            .insert(uri.to_string(), provider_name);
        Ok(data)
    }

    /// Retrieve a rendered prompt from its originating provider.
    pub async fn get_prompt(
        &self,
        prompt_name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<McpPromptDetail> {
        let provider = self.resolve_provider_for_prompt(prompt_name).await?;
        let provider_name = provider.name.clone();
        let allowlist_snapshot = self.allowlist.read().clone();
        let prompt = prompts::get_prompt_from_provider(
            &provider,
            prompt_name,
            arguments.unwrap_or_default(),
            self.request_timeout(),
            &allowlist_snapshot,
        )
        .await?;
        self.prompt_provider_index
            .write()
            .insert(prompt_name.to_string(), provider_name);
        Ok(prompt)
    }

    /// Current status snapshot for UI/debugging purposes.
    pub fn get_status(&self) -> McpClientStatus {
        let providers = self.providers.read();
        McpClientStatus {
            enabled: self.config.enabled,
            provider_count: providers.len(),
            active_connections: providers.len(),
            configured_providers: providers.keys().cloned().collect(),
        }
    }

    async fn collect_tools(&self, force_refresh: bool) -> Result<Vec<McpToolInfo>> {
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        if providers.is_empty() {
            return Ok(Vec::new());
        }

        let allowlist = self.allowlist.read().clone();
        let timeout = self.tool_timeout();
        let mut all_tools = Vec::new();
        let mut index_updates: HashMap<String, String> = HashMap::new();

        for provider in providers {
            let provider_name = provider.name.clone();
            let tools = if force_refresh {
                provider.refresh_tools(&allowlist, timeout).await
            } else {
                provider.list_tools(&allowlist, timeout).await
            };

            match tools {
                Ok(tools) => {
                    for tool in &tools {
                        index_updates.insert(tool.name.clone(), provider_name.clone());
                    }
                    all_tools.extend(tools);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to list tools for provider '{}': {err}",
                        provider_name
                    );
                }
            }
        }

        if !index_updates.is_empty() {
            *self.tool_provider_index.write() = index_updates;
        } else if force_refresh {
            self.tool_provider_index.write().clear();
        }

        Ok(all_tools)
    }

    async fn collect_resources(&self, force_refresh: bool) -> Result<Vec<McpResourceInfo>> {
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        if providers.is_empty() {
            self.resource_provider_index.write().clear();
            return Ok(Vec::new());
        }

        let allowlist = self.allowlist.read().clone();
        let timeout = self.request_timeout();
        let mut all_resources = Vec::new();

        for provider in providers {
            let resources = if force_refresh {
                provider.refresh_resources(&allowlist, timeout).await
            } else {
                provider.list_resources(&allowlist, timeout).await
            };

            match resources {
                Ok(resources) => {
                    all_resources.extend(resources);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to list resources for provider '{}': {err}",
                        provider.name
                    );
                }
            }
        }

        let mut index = self.resource_provider_index.write();
        index.clear();
        for resource in &all_resources {
            index.insert(resource.uri.clone(), resource.provider.clone());
        }

        Ok(all_resources)
    }

    async fn collect_prompts(&self, force_refresh: bool) -> Result<Vec<McpPromptInfo>> {
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        if providers.is_empty() {
            self.prompt_provider_index.write().clear();
            return Ok(Vec::new());
        }

        let allowlist = self.allowlist.read().clone();
        let timeout = self.request_timeout();
        let mut all_prompts = Vec::new();

        for provider in providers {
            let prompts = if force_refresh {
                provider.refresh_prompts(&allowlist, timeout).await
            } else {
                provider.list_prompts(&allowlist, timeout).await
            };

            match prompts {
                Ok(prompts) => {
                    all_prompts.extend(prompts);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to list prompts for provider '{}': {err}",
                        provider.name
                    );
                }
            }
        }

        let mut index = self.prompt_provider_index.write();
        index.clear();
        for prompt in &all_prompts {
            index.insert(prompt.name.clone(), prompt.provider.clone());
        }

        Ok(all_prompts)
    }

    /// Resolve which provider hosts a specific tool.
    ///
    /// This function implements an efficient tool lookup strategy:
    /// 1. Check the cached tool-to-provider index first (O(1) lookup)
    /// 2. If not cached, query all providers to find the tool
    /// 3. If still not found, force refresh the tool cache and retry
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool to locate
    ///
    /// # Returns
    /// * `Ok(Arc<McpProvider>)` - The provider that hosts this tool
    /// * `Err` - If the tool is not found or MCP is disabled
    ///
    /// # Performance Notes
    /// - First call for a tool may be slower (queries all providers)
    /// - Subsequent calls use cached index for O(1) lookup
    /// - Cache is invalidated when allowlist changes
    async fn resolve_provider_for_tool(&self, tool_name: &str) -> Result<Arc<McpProvider>> {
        // Early exit if MCP is disabled
        if !self.config.enabled {
            return Err(anyhow!(
                "MCP support is disabled in the current configuration"
            ));
        }

        // Fast path: check if we've already resolved this tool
        if let Some(provider) = self.provider_for_tool(tool_name) {
            if let Some(found) = self.providers.read().get(&provider) {
                return Ok(found.clone());
            }
        }

        // Prepare for provider search
        let allowlist = self.allowlist.read().clone();
        let timeout = self.tool_timeout();
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        // Check if any providers are available
        if providers.is_empty() {
            if self.config.providers.is_empty() {
                return Err(anyhow!(
                    "No MCP providers are configured. Use `vtcode mcp add` or update vtcode.toml to register one."
                ));
            }

            return Err(anyhow!(
                "No MCP providers are currently connected. Ensure MCP initialization completed successfully."
            ));
        }

        // Query each provider to find the tool
        for provider in providers {
            match provider.has_tool(tool_name, &allowlist, timeout).await {
                Ok(true) => {
                    // Cache the result for future lookups
                    self.tool_provider_index
                        .write()
                        .insert(tool_name.to_string(), provider.name.clone());
                    return Ok(provider);
                }
                Ok(false) => continue,
                Err(err) => {
                    tracing::warn!(
                        "Error checking tool '{}' on provider '{}': {err}",
                        tool_name, provider.name
                    );
                }
            }
        }

        // Last resort: force refresh all tool caches and try again
        match self.collect_tools(true).await {
            Ok(_) => {
                if let Some(provider) = self.provider_for_tool(tool_name) {
                    if let Some(found) = self.providers.read().get(&provider) {
                        return Ok(found.clone());
                    }
                }
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to refresh MCP tool caches while resolving '{}': {err}",
                    tool_name
                );
            }
        }

        Err(anyhow!(
            "Tool '{}' not found on any MCP provider",
            tool_name
        ))
    }

    async fn resolve_provider_for_resource(&self, uri: &str) -> Result<Arc<McpProvider>> {
        if let Some(provider) = self.provider_for_resource(uri) {
            if let Some(found) = self.providers.read().get(&provider) {
                return Ok(found.clone());
            }
        }

        let allowlist = self.allowlist.read().clone();
        let timeout = self.request_timeout();
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        for provider in providers {
            match provider.has_resource(uri, &allowlist, timeout).await {
                Ok(true) => {
                    self.resource_provider_index
                        .write()
                        .insert(uri.to_string(), provider.name.clone());
                    return Ok(provider);
                }
                Ok(false) => continue,
                Err(err) => {
                    tracing::warn!(
                        "Error checking resource '{}' on provider '{}': {err}",
                        uri, provider.name
                    );
                }
            }
        }

        Err(anyhow!("Resource '{}' not found on any MCP provider", uri))
    }

    async fn resolve_provider_for_prompt(&self, prompt_name: &str) -> Result<Arc<McpProvider>> {
        if let Some(provider) = self.provider_for_prompt(prompt_name) {
            if let Some(found) = self.providers.read().get(&provider) {
                return Ok(found.clone());
            }
        }

        let allowlist = self.allowlist.read().clone();
        let timeout = self.request_timeout();
        let providers: Vec<Arc<McpProvider>> = self.providers.read().values().cloned().collect();

        for provider in providers {
            match provider.has_prompt(prompt_name, &allowlist, timeout).await {
                Ok(true) => {
                    self.prompt_provider_index
                        .write()
                        .insert(prompt_name.to_string(), provider.name.clone());
                    return Ok(provider);
                }
                Ok(false) => continue,
                Err(err) => {
                    tracing::warn!(
                        "Error checking prompt '{}' on provider '{}': {err}",
                        prompt_name, provider.name
                    );
                }
            }
        }

        Err(anyhow!(
            "Prompt '{}' not found on any MCP provider",
            prompt_name
        ))
    }
}

#[async_trait]
impl McpToolExecutor for McpClient {
    async fn execute_mcp_tool(&self, tool_name: &str, args: Value) -> Result<Value> {
        self.execute_tool_with_validation(tool_name, args).await
    }

    async fn list_mcp_tools(&self) -> Result<Vec<McpToolInfo>> {
        self.collect_tools(false).await
    }

    async fn has_mcp_tool(&self, tool_name: &str) -> Result<bool> {
        if !self.config.enabled {
            bail!("MCP support is disabled in the current configuration");
        }

        if self.provider_for_tool(tool_name).is_some() {
            return Ok(true);
        }

        if self.providers.read().is_empty() {
            if self.config.providers.is_empty() {
                return Ok(false);
            }

            bail!(
                "No MCP providers are currently connected. Ensure MCP initialization completed successfully."
            );
        }

        let tools = self.collect_tools(false).await?;
        if tools.iter().any(|tool| tool.name == tool_name) {
            return Ok(true);
        }

        let refreshed = self.collect_tools(true).await?;
        Ok(refreshed.iter().any(|tool| tool.name == tool_name))
    }

    fn get_status(&self) -> McpClientStatus {
        self.get_status()
    }
}
