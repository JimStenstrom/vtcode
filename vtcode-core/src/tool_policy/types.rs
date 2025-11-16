//! Policy configuration types and enums

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::constants::tools;
use crate::config::mcp::{McpAllowListConfig, McpAllowListRules};

/// Auto-allow tool list (read-only safe tools)
pub const AUTO_ALLOW_TOOLS: &[&str] = &[
    tools::GREP_FILE,
    tools::LIST_FILES,
    tools::UPDATE_PLAN,
    tools::RUN_COMMAND,
    tools::READ_FILE,
    tools::EDIT_FILE,
];

/// Tool execution policy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolPolicy {
    /// Allow tool execution without prompting
    Allow,
    /// Prompt user for confirmation each time
    Prompt,
    /// Never allow tool execution
    Deny,
}

impl Default for ToolPolicy {
    fn default() -> Self {
        ToolPolicy::Prompt
    }
}

/// Tool policy configuration stored in ~/.vtcode/tool-policy.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicyConfig {
    /// Configuration version for future compatibility
    pub version: u32,
    /// Available tools at time of last update
    pub available_tools: Vec<String>,
    /// Policy for each tool
    pub policies: IndexMap<String, ToolPolicy>,
    /// Optional per-tool constraints to scope permissions and enforce safety
    #[serde(default)]
    pub constraints: IndexMap<String, ToolConstraints>,
    /// MCP-specific policy configuration
    #[serde(default)]
    pub mcp: McpPolicyStore,
}

impl Default for ToolPolicyConfig {
    fn default() -> Self {
        Self {
            version: 1,
            available_tools: Vec::new(),
            policies: IndexMap::new(),
            constraints: IndexMap::new(),
            mcp: McpPolicyStore::default(),
        }
    }
}

/// Stored MCP policy state, persisted alongside standard tool policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPolicyStore {
    /// Active MCP allow list configuration
    #[serde(default = "default_secure_mcp_allowlist")]
    pub allowlist: McpAllowListConfig,
    /// Provider-specific tool policies (allow/prompt/deny)
    #[serde(default)]
    pub providers: IndexMap<String, McpProviderPolicy>,
}

impl Default for McpPolicyStore {
    fn default() -> Self {
        Self {
            allowlist: default_secure_mcp_allowlist(),
            providers: IndexMap::new(),
        }
    }
}

/// MCP provider policy entry containing per-tool permissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpProviderPolicy {
    #[serde(default)]
    pub tools: IndexMap<String, ToolPolicy>,
}

/// Create default secure MCP allowlist configuration
pub fn default_secure_mcp_allowlist() -> McpAllowListConfig {
    let mut allowlist = McpAllowListConfig::default();
    allowlist.enforce = true;

    allowlist.default.logging = Some(vec![
        "mcp.provider_initialized".to_string(),
        "mcp.provider_initialization_failed".to_string(),
        "mcp.tool_filtered".to_string(),
        "mcp.tool_execution".to_string(),
        "mcp.tool_failed".to_string(),
        "mcp.tool_denied".to_string(),
    ]);

    allowlist.default.configuration = Some(BTreeMap::from([
        (
            "client".to_string(),
            vec![
                "max_concurrent_connections".to_string(),
                "request_timeout_seconds".to_string(),
                "retry_attempts".to_string(),
                "startup_timeout_seconds".to_string(),
                "tool_timeout_seconds".to_string(),
                "experimental_use_rmcp_client".to_string(),
            ],
        ),
        (
            "ui".to_string(),
            vec![
                "mode".to_string(),
                "max_events".to_string(),
                "show_provider_names".to_string(),
            ],
        ),
        (
            "server".to_string(),
            vec![
                "enabled".to_string(),
                "bind_address".to_string(),
                "port".to_string(),
                "transport".to_string(),
                "name".to_string(),
                "version".to_string(),
            ],
        ),
    ]));

    let mut time_rules = McpAllowListRules::default();
    time_rules.tools = Some(vec![
        "get_*".to_string(),
        "list_*".to_string(),
        "convert_timezone".to_string(),
        "describe_timezone".to_string(),
        "time_*".to_string(),
    ]);
    time_rules.resources = Some(vec!["timezone:*".to_string(), "location:*".to_string()]);
    time_rules.logging = Some(vec![
        "mcp.tool_execution".to_string(),
        "mcp.tool_failed".to_string(),
        "mcp.tool_denied".to_string(),
        "mcp.tool_filtered".to_string(),
        "mcp.provider_initialized".to_string(),
    ]);
    time_rules.configuration = Some(BTreeMap::from([
        (
            "provider".to_string(),
            vec!["max_concurrent_requests".to_string()],
        ),
        (
            "time".to_string(),
            vec!["local_timezone_override".to_string()],
        ),
    ]));
    allowlist.providers.insert("time".to_string(), time_rules);

    let mut context_rules = McpAllowListRules::default();
    context_rules.tools = Some(vec![
        "search_*".to_string(),
        "fetch_*".to_string(),
        "list_*".to_string(),
        "context7_*".to_string(),
        "get_*".to_string(),
    ]);
    context_rules.resources = Some(vec![
        "docs::*".to_string(),
        "snippets::*".to_string(),
        "repositories::*".to_string(),
        "context7::*".to_string(),
    ]);
    context_rules.prompts = Some(vec![
        "context7::*".to_string(),
        "support::*".to_string(),
        "docs::*".to_string(),
    ]);
    context_rules.logging = Some(vec![
        "mcp.tool_execution".to_string(),
        "mcp.tool_failed".to_string(),
        "mcp.tool_denied".to_string(),
        "mcp.tool_filtered".to_string(),
        "mcp.provider_initialized".to_string(),
    ]);
    context_rules.configuration = Some(BTreeMap::from([
        (
            "provider".to_string(),
            vec!["max_concurrent_requests".to_string()],
        ),
        (
            "context7".to_string(),
            vec![
                "workspace".to_string(),
                "search_scope".to_string(),
                "max_results".to_string(),
            ],
        ),
    ]));
    allowlist
        .providers
        .insert("context7".to_string(), context_rules);

    let mut seq_rules = McpAllowListRules::default();
    seq_rules.tools = Some(vec![
        "plan".to_string(),
        "critique".to_string(),
        "reflect".to_string(),
        "decompose".to_string(),
        "sequential_*".to_string(),
    ]);
    seq_rules.prompts = Some(vec![
        "sequential-thinking::*".to_string(),
        "plan".to_string(),
        "reflect".to_string(),
        "critique".to_string(),
    ]);
    seq_rules.logging = Some(vec![
        "mcp.tool_execution".to_string(),
        "mcp.tool_failed".to_string(),
        "mcp.tool_denied".to_string(),
        "mcp.tool_filtered".to_string(),
        "mcp.provider_initialized".to_string(),
    ]);
    seq_rules.configuration = Some(BTreeMap::from([
        (
            "provider".to_string(),
            vec!["max_concurrent_requests".to_string()],
        ),
        (
            "sequencing".to_string(),
            vec!["max_depth".to_string(), "max_branches".to_string()],
        ),
    ]));
    allowlist
        .providers
        .insert("sequential-thinking".to_string(), seq_rules);

    allowlist
}

/// Scoped, optional constraints for a tool to align with safe defaults
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolConstraints {
    /// Whitelisted modes for tools that support modes (e.g., 'terminal')
    #[serde(default)]
    pub allowed_modes: Option<Vec<String>>,
    /// Cap on results for list/search-like tools
    #[serde(default)]
    pub max_results_per_call: Option<usize>,
    /// Cap on items scanned for file listing
    #[serde(default)]
    pub max_items_per_call: Option<usize>,
    /// Default response format if unspecified by caller
    #[serde(default)]
    pub default_response_format: Option<String>,
    /// Cap maximum bytes when reading files
    #[serde(default)]
    pub max_bytes_per_read: Option<usize>,
    /// Cap maximum bytes when fetching over the network
    #[serde(default)]
    pub max_response_bytes: Option<usize>,
    /// Allowed URL schemes for network tools
    #[serde(default)]
    pub allowed_url_schemes: Option<Vec<String>>,
    /// Denied URL hosts or suffixes for network tools
    #[serde(default)]
    pub denied_url_hosts: Option<Vec<String>>,
}

/// Alternative tool policy configuration format (user's format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeToolPolicyConfig {
    /// Configuration version for future compatibility
    pub version: u32,
    /// Default policy settings
    pub default: AlternativeDefaultPolicy,
    /// Tool-specific policies
    pub tools: IndexMap<String, AlternativeToolPolicy>,
    /// Optional per-tool constraints (ignored if absent)
    #[serde(default)]
    pub constraints: IndexMap<String, ToolConstraints>,
}

/// Default policy in alternative format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeDefaultPolicy {
    /// Whether to allow by default
    pub allow: bool,
    /// Rate limit per run
    pub rate_limit_per_run: u32,
    /// Max concurrent executions
    pub max_concurrent: u32,
    /// Allow filesystem writes
    pub fs_write: bool,
    /// Allow network access
    pub network: bool,
}

/// Tool policy in alternative format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeToolPolicy {
    /// Whether to allow this tool
    pub allow: bool,
    /// Allow filesystem writes (optional)
    #[serde(default)]
    pub fs_write: bool,
    /// Allow network access (optional)
    #[serde(default)]
    pub network: bool,
    /// Arguments policy (optional)
    #[serde(default)]
    pub args_policy: Option<AlternativeArgsPolicy>,
}

/// Arguments policy in alternative format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeArgsPolicy {
    /// Substrings to deny
    pub deny_substrings: Vec<String>,
}
