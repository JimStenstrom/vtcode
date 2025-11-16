//! Policy evaluation engine

use super::mcp::parse_mcp_policy_key;
use super::types::{ToolPolicy, ToolPolicyConfig, AUTO_ALLOW_TOOLS};
use crate::tools::names::canonical_tool_name;

/// Evaluates tool execution policies
pub struct PolicyEvaluator;

impl PolicyEvaluator {
    /// Get policy for a specific tool
    pub fn get_policy(config: &ToolPolicyConfig, tool_name: &str) -> ToolPolicy {
        let canonical = canonical_tool_name(tool_name);
        if let Some((provider, tool)) = parse_mcp_policy_key(tool_name) {
            return Self::get_mcp_tool_policy(config, &provider, &tool);
        }

        config
            .policies
            .get(canonical.as_ref())
            .cloned()
            .unwrap_or(ToolPolicy::Prompt)
    }

    /// Retrieve policy for a specific MCP tool
    pub fn get_mcp_tool_policy(
        config: &ToolPolicyConfig,
        provider: &str,
        tool: &str,
    ) -> ToolPolicy {
        config
            .mcp
            .providers
            .get(provider)
            .and_then(|policy| policy.tools.get(tool))
            .cloned()
            .unwrap_or(ToolPolicy::Prompt)
    }

    /// Check if tool is in auto-allow list
    pub fn is_auto_allow_tool(tool_name: &str) -> bool {
        let canonical = canonical_tool_name(tool_name);
        AUTO_ALLOW_TOOLS.contains(&canonical.as_ref())
    }

    /// Evaluate whether a tool should be executed based on policy
    ///
    /// Returns:
    /// - `Ok(true)` if tool should execute (Allow policy)
    /// - `Ok(false)` if tool should not execute (Deny policy)
    /// - `Err(PolicyDecision::RequiresPrompt)` if user prompt is needed (Prompt policy)
    pub fn should_execute_tool(
        config: &ToolPolicyConfig,
        tool_name: &str,
    ) -> Result<bool, PolicyDecision> {
        let policy = Self::get_policy(config, tool_name);

        match policy {
            ToolPolicy::Allow => Ok(true),
            ToolPolicy::Deny => Ok(false),
            ToolPolicy::Prompt => {
                // Check if it's an auto-allow tool
                if Self::is_auto_allow_tool(tool_name) {
                    Ok(true)
                } else {
                    Err(PolicyDecision::RequiresPrompt)
                }
            }
        }
    }
}

/// Policy evaluation decision
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    /// Prompt user for confirmation is required
    RequiresPrompt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::constants::tools;

    #[test]
    fn test_auto_allow_tools() {
        // Auto-allow tools should be recognized
        assert!(PolicyEvaluator::is_auto_allow_tool(tools::READ_FILE));
        assert!(PolicyEvaluator::is_auto_allow_tool(tools::LIST_FILES));
        assert!(PolicyEvaluator::is_auto_allow_tool(tools::GREP_FILE));

        // Non-auto-allow tools should not be recognized
        assert!(!PolicyEvaluator::is_auto_allow_tool(tools::WRITE_FILE));
        assert!(!PolicyEvaluator::is_auto_allow_tool("unknown_tool"));
    }

    #[test]
    fn test_should_execute_tool() {
        let mut config = ToolPolicyConfig::default();

        // Allow policy
        config
            .policies
            .insert("allowed_tool".to_string(), ToolPolicy::Allow);
        assert_eq!(
            PolicyEvaluator::should_execute_tool(&config, "allowed_tool"),
            Ok(true)
        );

        // Deny policy
        config
            .policies
            .insert("denied_tool".to_string(), ToolPolicy::Deny);
        assert_eq!(
            PolicyEvaluator::should_execute_tool(&config, "denied_tool"),
            Ok(false)
        );

        // Prompt policy for auto-allow tool
        assert_eq!(
            PolicyEvaluator::should_execute_tool(&config, tools::READ_FILE),
            Ok(true)
        );

        // Prompt policy for non-auto-allow tool
        config
            .policies
            .insert("prompt_tool".to_string(), ToolPolicy::Prompt);
        assert_eq!(
            PolicyEvaluator::should_execute_tool(&config, "prompt_tool"),
            Err(PolicyDecision::RequiresPrompt)
        );
    }
}
