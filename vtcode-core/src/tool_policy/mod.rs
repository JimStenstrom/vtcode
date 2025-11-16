//! Tool policy management system
//!
//! Manages user preferences for tool usage, storing choices in
//! ~/.vtcode/tool-policy.json to minimize repeated prompts while maintaining
//! user control over which tools the agent can use.
//!
//! ## Module Organization
//!
//! - `types`: Policy configuration types and enums
//! - `storage`: Policy file loading and persistence
//! - `evaluator`: Policy evaluation engine
//! - `enforcement`: Constraint enforcement
//! - `interactive`: User prompting and interaction
//! - `mcp`: MCP-specific policy logic
//! - `audit`: Audit logging and history

pub mod audit;
pub mod enforcement;
pub mod evaluator;
pub mod interactive;
pub mod mcp;
pub mod storage;
pub mod types;

// Re-export main types for backward compatibility
pub use types::{
    AlternativeArgsPolicy, AlternativeDefaultPolicy, AlternativeToolPolicy,
    AlternativeToolPolicyConfig, McpPolicyStore, McpProviderPolicy, ToolConstraints, ToolPolicy,
    ToolPolicyConfig, AUTO_ALLOW_TOOLS,
};

use anyhow::Result;
use dialoguer::console::style;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::core::tools::{ToolPolicy as ConfigToolPolicy, ToolsConfig};
use crate::config::mcp::McpAllowListConfig;
use crate::tools::names::canonical_tool_name;

use evaluator::PolicyEvaluator;
use interactive::InteractivePolicyPrompt;
use mcp::parse_mcp_policy_key;
use storage::PolicyStorage;

/// Tool policy manager - main orchestrator for all policy operations
#[derive(Clone)]
pub struct ToolPolicyManager {
    storage: PolicyStorage,
    config: ToolPolicyConfig,
}

impl ToolPolicyManager {
    /// Create a new tool policy manager
    pub async fn new() -> Result<Self> {
        let storage = PolicyStorage::new().await?;
        let config = storage.load_or_create_config().await?;

        Ok(Self { storage, config })
    }

    /// Create a new tool policy manager with workspace-specific config
    pub async fn new_with_workspace(workspace_root: &PathBuf) -> Result<Self> {
        let storage = PolicyStorage::new_with_workspace(workspace_root).await?;
        let config = storage.load_or_create_config().await?;

        Ok(Self { storage, config })
    }

    /// Create a new tool policy manager backed by a custom configuration path
    pub async fn new_with_config_path<P: Into<PathBuf>>(config_path: P) -> Result<Self> {
        let storage = PolicyStorage::new_with_config_path(config_path).await?;
        let config = storage.load_or_create_config().await?;

        Ok(Self { storage, config })
    }

    /// Apply policies defined in vtcode.toml to the runtime policy manager
    pub async fn apply_tools_config(&mut self, tools_config: &ToolsConfig) -> Result<()> {
        if self.config.available_tools.is_empty() {
            return Ok(());
        }

        for tool in self.config.available_tools.clone() {
            let config_policy = Self::resolve_config_policy(tools_config, &tool);
            self.apply_config_policy(&tool, config_policy);
        }

        storage::PolicyStorage::apply_auto_allow_defaults(&mut self.config);
        self.save_config().await
    }

    fn apply_config_policy(&mut self, tool_name: &str, policy: ConfigToolPolicy) {
        let canonical = canonical_tool_name(tool_name);
        let runtime_policy = match policy {
            ConfigToolPolicy::Allow => ToolPolicy::Allow,
            ConfigToolPolicy::Prompt => ToolPolicy::Prompt,
            ConfigToolPolicy::Deny => ToolPolicy::Deny,
        };

        self.config
            .policies
            .insert(canonical.into_owned(), runtime_policy);
    }

    fn resolve_config_policy(tools_config: &ToolsConfig, tool_name: &str) -> ConfigToolPolicy {
        use crate::config::constants::tools;

        let canonical = canonical_tool_name(tool_name);
        let lookup = canonical.as_ref();

        if let Some(policy) = tools_config.policies.get(lookup) {
            return policy.clone();
        }

        match tool_name {
            tools::LIST_FILES => tools_config
                .policies
                .get("list_dir")
                .or_else(|| tools_config.policies.get("list_directory"))
                .cloned(),
            _ => None,
        }
        .unwrap_or_else(|| tools_config.default_policy.clone())
    }

    /// Update the tool list and save configuration
    pub async fn update_available_tools(&mut self, tools: Vec<String>) -> Result<()> {
        let mut canonical_tools = Vec::new();
        for tool in tools {
            let canonical = canonical_tool_name(&tool).into_owned();
            if !canonical_tools.contains(&canonical) {
                canonical_tools.push(canonical);
            }
        }
        canonical_tools.sort();

        let current_tools: HashSet<_> = self.config.policies.keys().cloned().collect();
        let new_tools: HashSet<_> = canonical_tools
            .iter()
            .filter(|name| !name.starts_with("mcp::"))
            .cloned()
            .collect();

        let mut has_changes = false;

        for tool in canonical_tools
            .iter()
            .filter(|tool| !tool.starts_with("mcp::") && !current_tools.contains(*tool))
        {
            let default_policy = if AUTO_ALLOW_TOOLS.contains(&tool.as_str()) {
                ToolPolicy::Allow
            } else {
                ToolPolicy::Prompt
            };
            self.config.policies.insert(tool.clone(), default_policy);
            has_changes = true;
        }

        let tools_to_remove: Vec<_> = self
            .config
            .policies
            .keys()
            .filter(|tool| !new_tools.contains(*tool))
            .cloned()
            .collect();

        for tool in tools_to_remove {
            self.config.policies.shift_remove(&tool);
            has_changes = true;
        }

        let mut current_available = self.config.available_tools.clone();
        current_available.sort();
        if current_available != canonical_tools {
            self.config.available_tools = canonical_tools;
            has_changes = true;
        }

        if has_changes {
            self.save_config().await
        } else {
            Ok(())
        }
    }

    /// Synchronize MCP provider tool lists with persisted policies
    pub async fn update_mcp_tools(
        &mut self,
        provider_tools: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
        let stored_providers: HashSet<String> = self.config.mcp.providers.keys().cloned().collect();
        let mut has_changes = false;

        // Update or insert provider entries
        for (provider, tools) in provider_tools {
            let entry = self
                .config
                .mcp
                .providers
                .entry(provider.clone())
                .or_insert_with(McpProviderPolicy::default);

            let existing_tools: HashSet<String> = entry.tools.keys().cloned().collect();
            let advertised: HashSet<String> = tools.iter().cloned().collect();

            // Add new tools with default Prompt policy
            for tool in tools {
                if !existing_tools.contains(tool) {
                    entry.tools.insert(tool.clone(), ToolPolicy::Prompt);
                    has_changes = true;
                }
            }

            // Remove tools no longer advertised
            for stale in existing_tools.difference(&advertised) {
                entry.tools.shift_remove(stale.as_str());
                has_changes = true;
            }
        }

        // Remove providers that are no longer present
        let advertised_providers: HashSet<String> = provider_tools.keys().cloned().collect();
        for provider in stored_providers
            .difference(&advertised_providers)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.config.mcp.providers.shift_remove(provider.as_str());
            has_changes = true;
        }

        // Remove any stale MCP keys from the primary policy map
        let stale_runtime_keys: Vec<_> = self
            .config
            .policies
            .keys()
            .filter(|name| name.starts_with("mcp::"))
            .cloned()
            .collect();

        for key in stale_runtime_keys {
            self.config.policies.shift_remove(&key);
            has_changes = true;
        }

        // Refresh available tools list with MCP entries included
        let mut available: Vec<String> = self
            .config
            .available_tools
            .iter()
            .filter(|name| !name.starts_with("mcp::"))
            .cloned()
            .collect();

        for (provider, policy) in &self.config.mcp.providers {
            for tool in policy.tools.keys() {
                available.push(format!("mcp::{}::{}", provider, tool));
            }
        }

        available.sort();
        available.dedup();

        // Check if the available tools list has actually changed
        if self.config.available_tools != available {
            self.config.available_tools = available;
            has_changes = true;
        }

        if has_changes {
            self.save_config().await
        } else {
            Ok(())
        }
    }

    /// Retrieve policy for a specific MCP tool
    pub fn get_mcp_tool_policy(&self, provider: &str, tool: &str) -> ToolPolicy {
        PolicyEvaluator::get_mcp_tool_policy(&self.config, provider, tool)
    }

    /// Update policy for a specific MCP tool
    pub async fn set_mcp_tool_policy(
        &mut self,
        provider: &str,
        tool: &str,
        policy: ToolPolicy,
    ) -> Result<()> {
        let entry = self
            .config
            .mcp
            .providers
            .entry(provider.to_string())
            .or_insert_with(McpProviderPolicy::default);
        entry.tools.insert(tool.to_string(), policy);
        self.save_config().await
    }

    /// Access the persisted MCP allow list configuration
    pub fn mcp_allowlist(&self) -> &McpAllowListConfig {
        &self.config.mcp.allowlist
    }

    /// Replace the persisted MCP allow list configuration
    pub async fn set_mcp_allowlist(&mut self, allowlist: McpAllowListConfig) -> Result<()> {
        self.config.mcp.allowlist = allowlist;
        self.save_config().await
    }

    /// Get policy for a specific tool
    pub fn get_policy(&self, tool_name: &str) -> ToolPolicy {
        PolicyEvaluator::get_policy(&self.config, tool_name)
    }

    /// Get optional constraints for a specific tool
    pub fn get_constraints(&self, tool_name: &str) -> Option<&ToolConstraints> {
        enforcement::ConstraintEnforcer::get_constraints(&self.config, tool_name)
    }

    /// Check if tool should be executed based on policy
    pub async fn should_execute_tool(&mut self, tool_name: &str) -> Result<bool> {
        if let Some((provider, tool)) = parse_mcp_policy_key(tool_name) {
            return match self.get_mcp_tool_policy(&provider, &tool) {
                ToolPolicy::Allow => Ok(true),
                ToolPolicy::Deny => Ok(false),
                ToolPolicy::Prompt => {
                    if ToolPolicyManager::is_auto_allow_tool(tool_name) {
                        self.set_mcp_tool_policy(&provider, &tool, ToolPolicy::Allow)
                            .await?;
                        Ok(true)
                    } else {
                        self.prompt_user_for_tool(tool_name).await
                    }
                }
            };
        }

        let canonical = canonical_tool_name(tool_name);

        match self.get_policy(canonical.as_ref()) {
            ToolPolicy::Allow => Ok(true),
            ToolPolicy::Deny => Ok(false),
            ToolPolicy::Prompt => {
                let canonical_name = canonical.as_ref();
                if AUTO_ALLOW_TOOLS.contains(&canonical_name) {
                    self.set_policy(canonical_name, ToolPolicy::Allow).await?;
                    return Ok(true);
                }
                let should_execute = self.prompt_user_for_tool(canonical_name).await?;
                Ok(should_execute)
            }
        }
    }

    pub fn is_auto_allow_tool(tool_name: &str) -> bool {
        PolicyEvaluator::is_auto_allow_tool(tool_name)
    }

    /// Prompt user for tool execution permission
    async fn prompt_user_for_tool(&mut self, tool_name: &str) -> Result<bool> {
        InteractivePolicyPrompt::prompt_user_for_tool(tool_name).await
    }

    /// Set policy for a specific tool
    pub async fn set_policy(&mut self, tool_name: &str, policy: ToolPolicy) -> Result<()> {
        if let Some((provider, tool)) = parse_mcp_policy_key(tool_name) {
            return self.set_mcp_tool_policy(&provider, &tool, policy).await;
        }

        let canonical = canonical_tool_name(tool_name);
        self.config.policies.insert(canonical.into_owned(), policy);
        self.save_config().await
    }

    /// Reset all tools to prompt
    pub async fn reset_all_to_prompt(&mut self) -> Result<()> {
        for policy in self.config.policies.values_mut() {
            *policy = ToolPolicy::Prompt;
        }
        for provider in self.config.mcp.providers.values_mut() {
            for policy in provider.tools.values_mut() {
                *policy = ToolPolicy::Prompt;
            }
        }
        self.save_config().await
    }

    /// Allow all tools
    pub async fn allow_all_tools(&mut self) -> Result<()> {
        for policy in self.config.policies.values_mut() {
            *policy = ToolPolicy::Allow;
        }
        for provider in self.config.mcp.providers.values_mut() {
            for policy in provider.tools.values_mut() {
                *policy = ToolPolicy::Allow;
            }
        }
        self.save_config().await
    }

    /// Deny all tools
    pub async fn deny_all_tools(&mut self) -> Result<()> {
        for policy in self.config.policies.values_mut() {
            *policy = ToolPolicy::Deny;
        }
        for provider in self.config.mcp.providers.values_mut() {
            for policy in provider.tools.values_mut() {
                *policy = ToolPolicy::Deny;
            }
        }
        self.save_config().await
    }

    /// Get summary of current policies
    pub fn get_policy_summary(&self) -> IndexMap<String, ToolPolicy> {
        let mut summary = self.config.policies.clone();
        for (provider, policy) in &self.config.mcp.providers {
            for (tool, status) in &policy.tools {
                summary.insert(format!("mcp::{}::{}", provider, tool), status.clone());
            }
        }
        summary
    }

    /// Save configuration to file
    async fn save_config(&self) -> Result<()> {
        self.storage.write_config(&self.config).await
    }

    /// Print current policy status
    pub fn print_status(&self) {
        println!("{}", style("Tool Policy Status").cyan().bold());
        println!("Config file: {}", self.storage.config_path().display());
        println!();

        let summary = self.get_policy_summary();

        if summary.is_empty() {
            println!("No tools configured yet.");
            return;
        }

        let mut allow_count = 0;
        let mut prompt_count = 0;
        let mut deny_count = 0;

        for (tool, policy) in &summary {
            let (status, color_name) = match policy {
                ToolPolicy::Allow => {
                    allow_count += 1;
                    ("ALLOW", "green")
                }
                ToolPolicy::Prompt => {
                    prompt_count += 1;
                    ("PROMPT", "yellow")
                }
                ToolPolicy::Deny => {
                    deny_count += 1;
                    ("DENY", "red")
                }
            };

            let status_styled = match color_name {
                "green" => style(status).green(),
                "yellow" => style(status).yellow(),
                "red" => style(status).red(),
                _ => style(status),
            };

            println!(
                "  {} {}",
                style(format!("{:15}", tool)).cyan(),
                status_styled
            );
        }

        println!();
        println!(
            "Summary: {} allowed, {} prompt, {} denied",
            style(allow_count).green(),
            style(prompt_count).yellow(),
            style(deny_count).red()
        );
    }

    /// Expose path of the underlying policy configuration file
    pub fn config_path(&self) -> &Path {
        self.storage.config_path()
    }
}
