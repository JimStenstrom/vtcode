//! Core ToolRegistry struct and initialization

use super::builtins::register_builtin_tools;
use super::inventory::ToolInventory;
use super::policy::ToolPolicyGateway;
use super::pty::PtySessionManager;
use super::timeout::ToolTimeoutPolicy;
use crate::config::PtyConfig;
use crate::tool_policy::ToolPolicyManager;
use crate::tools::file_ops::FileOpsTool;
use crate::tools::grep_file::GrepSearchManager;
use crate::tools::names::tool_aliases;
use crate::tools::plan::PlanManager;
use crate::tools::pty::PtyManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use vtcode_mcp::McpClient;

/// Central tool registry managing all available tools
#[derive(Clone)]
pub struct ToolRegistry {
    pub(super) inventory: ToolInventory,
    pub(super) policy_gateway: ToolPolicyGateway,
    pub(super) pty_sessions: PtySessionManager,
    pub(super) mcp_client: Option<Arc<McpClient>>,
    pub(super) mcp_tool_index: HashMap<String, Vec<String>>,
    pub(super) mcp_tool_presence: HashMap<String, bool>,
    pub(super) timeout_policy: ToolTimeoutPolicy,
}

impl ToolRegistry {
    pub async fn new(workspace_root: PathBuf) -> Self {
        Self::build(workspace_root, PtyConfig::default(), true).await
    }

    pub async fn new_with_config(workspace_root: PathBuf, pty_config: PtyConfig) -> Self {
        Self::build(workspace_root, pty_config, true).await
    }

    pub async fn new_with_features(workspace_root: PathBuf, todo_planning_enabled: bool) -> Self {
        Self::build(workspace_root, PtyConfig::default(), todo_planning_enabled).await
    }

    pub async fn new_with_config_and_features(
        workspace_root: PathBuf,
        pty_config: PtyConfig,
        todo_planning_enabled: bool,
    ) -> Self {
        Self::build(workspace_root, pty_config, todo_planning_enabled).await
    }

    pub async fn new_with_custom_policy(
        workspace_root: PathBuf,
        policy_manager: ToolPolicyManager,
    ) -> Self {
        Self::build_with_policy(
            workspace_root,
            PtyConfig::default(),
            true,
            Some(policy_manager),
        )
        .await
    }

    pub async fn new_with_custom_policy_and_config(
        workspace_root: PathBuf,
        pty_config: PtyConfig,
        todo_planning_enabled: bool,
        policy_manager: ToolPolicyManager,
    ) -> Self {
        Self::build_with_policy(
            workspace_root,
            pty_config,
            todo_planning_enabled,
            Some(policy_manager),
        )
        .await
    }

    async fn build(
        workspace_root: PathBuf,
        pty_config: PtyConfig,
        todo_planning_enabled: bool,
    ) -> Self {
        Self::build_with_policy(workspace_root, pty_config, todo_planning_enabled, None).await
    }

    async fn build_with_policy(
        workspace_root: PathBuf,
        pty_config: PtyConfig,
        todo_planning_enabled: bool,
        policy_manager: Option<ToolPolicyManager>,
    ) -> Self {
        let mut inventory = ToolInventory::new(workspace_root.clone());
        register_builtin_tools(&mut inventory, todo_planning_enabled);

        let pty_sessions = PtySessionManager::new(workspace_root.clone(), pty_config);

        let policy_gateway = match policy_manager {
            Some(pm) => ToolPolicyGateway::with_policy_manager(pm),
            None => ToolPolicyGateway::new(&workspace_root).await,
        };

        let mut registry = Self {
            inventory,
            policy_gateway,
            pty_sessions,
            mcp_client: None,
            mcp_tool_index: HashMap::new(),
            mcp_tool_presence: HashMap::new(),
            timeout_policy: ToolTimeoutPolicy::default(),
        };

        registry.sync_policy_catalog().await;
        registry
    }

    pub(super) async fn sync_policy_catalog(&mut self) {
        let mut available = self.inventory.available_tools();
        let mut alias_entries = Vec::new();
        for tool in &available {
            for alias in tool_aliases(tool) {
                alias_entries.push(alias.to_string());
            }
        }
        available.extend(alias_entries);
        let mcp_keys = self.mcp_policy_keys();
        self.policy_gateway
            .sync_available_tools(available, &mcp_keys)
            .await;
    }

    pub(super) fn mcp_policy_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for (provider, tools) in &self.mcp_tool_index {
            for tool in tools {
                keys.push(format!("mcp::{}::{}", provider, tool));
            }
        }
        keys
    }

    pub(super) fn find_mcp_provider(&self, tool_name: &str) -> Option<String> {
        for (provider, tools) in &self.mcp_tool_index {
            if tools.iter().any(|candidate| candidate == tool_name) {
                return Some(provider.clone());
            }
        }
        None
    }

    // Accessor methods
    pub fn workspace_root(&self) -> &PathBuf {
        self.inventory.workspace_root()
    }

    pub fn file_ops_tool(&self) -> &FileOpsTool {
        self.inventory.file_ops_tool()
    }

    pub fn grep_file_manager(&self) -> Arc<GrepSearchManager> {
        self.inventory.grep_file_manager()
    }

    pub fn pty_manager(&self) -> &PtyManager {
        self.pty_sessions.manager()
    }

    pub fn pty_config(&self) -> &PtyConfig {
        self.pty_sessions.config()
    }

    pub fn can_start_pty_session(&self) -> bool {
        self.pty_sessions.can_start_session()
    }

    pub fn start_pty_session(&self) -> anyhow::Result<()> {
        self.pty_sessions.start_session()
    }

    pub fn end_pty_session(&self) {
        self.pty_sessions.end_session();
    }

    pub fn active_pty_sessions(&self) -> usize {
        self.pty_sessions.active_sessions()
    }

    pub fn plan_manager(&self) -> PlanManager {
        self.inventory.plan_manager()
    }

    pub fn current_plan(&self) -> crate::tools::TaskPlan {
        self.inventory.plan_manager().snapshot()
    }

    pub fn timeout_policy(&self) -> &ToolTimeoutPolicy {
        &self.timeout_policy
    }
}
