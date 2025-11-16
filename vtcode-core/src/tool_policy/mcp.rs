//! MCP-specific policy management

/// Parse MCP policy key in format "mcp::provider::tool"
///
/// Returns Some((provider, tool)) if the key matches the MCP format,
/// None otherwise.
pub fn parse_mcp_policy_key(tool_name: &str) -> Option<(String, String)> {
    let mut parts = tool_name.splitn(3, "::");
    match (parts.next()?, parts.next(), parts.next()) {
        ("mcp", Some(provider), Some(tool)) if !provider.is_empty() && !tool.is_empty() => {
            Some((provider.to_string(), tool.to_string()))
        }
        _ => None,
    }
}

/// Check if a tool name is an MCP tool
pub fn is_mcp_tool(tool_name: &str) -> bool {
    parse_mcp_policy_key(tool_name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mcp_policy_key() {
        // Valid MCP keys
        assert_eq!(
            parse_mcp_policy_key("mcp::time::get_current_time"),
            Some(("time".to_string(), "get_current_time".to_string()))
        );

        assert_eq!(
            parse_mcp_policy_key("mcp::context7::search_docs"),
            Some(("context7".to_string(), "search_docs".to_string()))
        );

        // Invalid MCP keys
        assert_eq!(parse_mcp_policy_key("read_file"), None);
        assert_eq!(parse_mcp_policy_key("mcp::provider"), None);
        assert_eq!(parse_mcp_policy_key("mcp::::tool"), None);
        assert_eq!(parse_mcp_policy_key("not_mcp::provider::tool"), None);
    }

    #[test]
    fn test_is_mcp_tool() {
        assert!(is_mcp_tool("mcp::time::get_current_time"));
        assert!(is_mcp_tool("mcp::context7::search"));
        assert!(!is_mcp_tool("read_file"));
        assert!(!is_mcp_tool("write_file"));
    }
}
