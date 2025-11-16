//! Policy file loading and persistence

use super::types::{
    AlternativeToolPolicyConfig, ToolPolicy, ToolPolicyConfig, AUTO_ALLOW_TOOLS,
};
use anyhow::{Context, Result};
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

/// Handles loading and saving policy configuration
#[derive(Clone)]
pub struct PolicyStorage {
    config_path: PathBuf,
}

impl PolicyStorage {
    /// Create a new policy storage for the default location (~/.vtcode/tool-policy.json)
    pub async fn new() -> Result<Self> {
        let config_path = Self::get_default_config_path().await?;
        Ok(Self { config_path })
    }

    /// Create a new policy storage with workspace-specific config
    pub async fn new_with_workspace(workspace_root: &PathBuf) -> Result<Self> {
        let config_path = Self::get_workspace_config_path(workspace_root).await?;
        Ok(Self { config_path })
    }

    /// Create a new policy storage backed by a custom configuration path
    pub async fn new_with_config_path<P: Into<PathBuf>>(config_path: P) -> Result<Self> {
        let config_path = config_path.into();

        if let Some(parent) = config_path.parent() {
            if !tokio::fs::try_exists(parent).await.unwrap_or(false) {
                tokio::fs::create_dir_all(parent).await.with_context(|| {
                    format!(
                        "Failed to create directory for tool policy config at {}",
                        parent.display()
                    )
                })?;
            }
        }

        Ok(Self { config_path })
    }

    /// Get the path to the default tool policy configuration file
    async fn get_default_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        let vtcode_dir = home_dir.join(".vtcode");
        if !tokio::fs::try_exists(&vtcode_dir).await.unwrap_or(false) {
            tokio::fs::create_dir_all(&vtcode_dir)
                .await
                .context("Failed to create ~/.vtcode directory")?;
        }

        Ok(vtcode_dir.join("tool-policy.json"))
    }

    /// Get the path to the workspace-specific tool policy configuration file
    async fn get_workspace_config_path(workspace_root: &PathBuf) -> Result<PathBuf> {
        let workspace_vtcode_dir = workspace_root.join(".vtcode");

        if !tokio::fs::try_exists(&workspace_vtcode_dir)
            .await
            .unwrap_or(false)
        {
            tokio::fs::create_dir_all(&workspace_vtcode_dir)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create workspace policy directory at {}",
                        workspace_vtcode_dir.display()
                    )
                })?;
        }

        Ok(workspace_vtcode_dir.join("tool-policy.json"))
    }

    /// Load existing config or create new one with all tools as "prompt"
    pub async fn load_or_create_config(&self) -> Result<ToolPolicyConfig> {
        if tokio::fs::try_exists(&self.config_path)
            .await
            .unwrap_or(false)
        {
            let content = tokio::fs::read_to_string(&self.config_path)
                .await
                .context("Failed to read tool policy config")?;

            // Try to parse as alternative format first
            if let Ok(alt_config) = serde_json::from_str::<AlternativeToolPolicyConfig>(&content) {
                // Convert alternative format to standard format
                return Ok(Self::convert_from_alternative(alt_config));
            }

            // Fall back to standard format with graceful recovery on parse errors
            match serde_json::from_str(&content) {
                Ok(mut config) => {
                    Self::apply_auto_allow_defaults(&mut config);
                    Self::ensure_network_constraints(&mut config);
                    Ok(config)
                }
                Err(parse_err) => {
                    eprintln!(
                        "Warning: Invalid tool policy config at {} ({}). Resetting to defaults.",
                        self.config_path.display(),
                        parse_err
                    );
                    self.reset_to_default().await
                }
            }
        } else {
            // Create new config with empty tools list
            let mut config = ToolPolicyConfig::default();
            Self::apply_auto_allow_defaults(&mut config);
            Self::ensure_network_constraints(&mut config);
            Ok(config)
        }
    }

    /// Apply auto-allow defaults for safe tools
    pub fn apply_auto_allow_defaults(config: &mut ToolPolicyConfig) {
        for tool in AUTO_ALLOW_TOOLS {
            config
                .policies
                .entry((*tool).to_string())
                .and_modify(|policy| *policy = ToolPolicy::Allow)
                .or_insert(ToolPolicy::Allow);
            if !config.available_tools.contains(&tool.to_string()) {
                config.available_tools.push(tool.to_string());
            }
        }
        Self::ensure_network_constraints(config);
    }

    fn ensure_network_constraints(_config: &mut ToolPolicyConfig) {
        // Network constraints removed with curl tool removal
    }

    async fn reset_to_default(&self) -> Result<ToolPolicyConfig> {
        let backup_path = self.config_path.with_extension("json.bak");

        if let Err(err) = tokio::fs::rename(&self.config_path, &backup_path).await {
            eprintln!(
                "Warning: Unable to back up invalid tool policy config ({}). {}",
                self.config_path.display(),
                err
            );
        } else {
            eprintln!(
                "Backed up invalid tool policy config to {}",
                backup_path.display()
            );
        }

        let default_config = ToolPolicyConfig::default();
        self.write_config(&default_config).await?;
        Ok(default_config)
    }

    /// Write config to file
    pub async fn write_config(&self, config: &ToolPolicyConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            if !tokio::fs::try_exists(parent).await.unwrap_or(false) {
                tokio::fs::create_dir_all(parent).await.with_context(|| {
                    format!(
                        "Failed to create directory for tool policy config at {}",
                        parent.display()
                    )
                })?;
            }
        }

        let serialized = serde_json::to_string_pretty(config)
            .context("Failed to serialize tool policy config")?;

        tokio::fs::write(&self.config_path, serialized)
            .await
            .with_context(|| {
                format!(
                    "Failed to write tool policy config: {}",
                    self.config_path.display()
                )
            })
    }

    /// Convert alternative format to standard format
    fn convert_from_alternative(alt_config: AlternativeToolPolicyConfig) -> ToolPolicyConfig {
        let mut policies = IndexMap::new();

        // Convert tool policies
        for (tool_name, alt_policy) in alt_config.tools {
            let policy = if alt_policy.allow {
                ToolPolicy::Allow
            } else {
                ToolPolicy::Deny
            };
            policies.insert(tool_name, policy);
        }

        let mut config = ToolPolicyConfig {
            version: alt_config.version,
            available_tools: policies.keys().cloned().collect(),
            policies,
            constraints: alt_config.constraints,
            mcp: Default::default(),
        };
        Self::apply_auto_allow_defaults(&mut config);
        config
    }

    /// Get the path to the policy configuration file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}
