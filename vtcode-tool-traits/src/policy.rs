//! Tool policy types for permission and access control

use serde::{Deserialize, Serialize};

/// Tool execution policy
///
/// Defines the permission level for a tool to be executed by the agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl ToolPolicy {
    /// Check if the policy allows execution without prompting
    pub fn is_allow(&self) -> bool {
        matches!(self, ToolPolicy::Allow)
    }

    /// Check if the policy requires prompting
    pub fn is_prompt(&self) -> bool {
        matches!(self, ToolPolicy::Prompt)
    }

    /// Check if the policy denies execution
    pub fn is_deny(&self) -> bool {
        matches!(self, ToolPolicy::Deny)
    }
}

impl std::fmt::Display for ToolPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolPolicy::Allow => write!(f, "allow"),
            ToolPolicy::Prompt => write!(f, "prompt"),
            ToolPolicy::Deny => write!(f, "deny"),
        }
    }
}

impl std::str::FromStr for ToolPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(ToolPolicy::Allow),
            "prompt" => Ok(ToolPolicy::Prompt),
            "deny" => Ok(ToolPolicy::Deny),
            _ => Err(format!("Invalid tool policy: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_policy_default() {
        assert_eq!(ToolPolicy::default(), ToolPolicy::Prompt);
    }

    #[test]
    fn test_tool_policy_is_methods() {
        assert!(ToolPolicy::Allow.is_allow());
        assert!(!ToolPolicy::Allow.is_prompt());
        assert!(!ToolPolicy::Allow.is_deny());

        assert!(!ToolPolicy::Prompt.is_allow());
        assert!(ToolPolicy::Prompt.is_prompt());
        assert!(!ToolPolicy::Prompt.is_deny());

        assert!(!ToolPolicy::Deny.is_allow());
        assert!(!ToolPolicy::Deny.is_prompt());
        assert!(ToolPolicy::Deny.is_deny());
    }

    #[test]
    fn test_tool_policy_display() {
        assert_eq!(ToolPolicy::Allow.to_string(), "allow");
        assert_eq!(ToolPolicy::Prompt.to_string(), "prompt");
        assert_eq!(ToolPolicy::Deny.to_string(), "deny");
    }

    #[test]
    fn test_tool_policy_from_str() {
        assert_eq!("allow".parse::<ToolPolicy>().unwrap(), ToolPolicy::Allow);
        assert_eq!("prompt".parse::<ToolPolicy>().unwrap(), ToolPolicy::Prompt);
        assert_eq!("deny".parse::<ToolPolicy>().unwrap(), ToolPolicy::Deny);
        assert_eq!("ALLOW".parse::<ToolPolicy>().unwrap(), ToolPolicy::Allow);
        assert!("invalid".parse::<ToolPolicy>().is_err());
    }

    #[test]
    fn test_tool_policy_serialization() {
        let policy = ToolPolicy::Allow;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"allow\"");

        let deserialized: ToolPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ToolPolicy::Allow);
    }
}
