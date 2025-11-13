//! Tests for CLI command functionality

use vtcode_config::mcp::{McpHttpServerConfig, McpStdioServerConfig, McpTransportConfig};

#[test]
fn test_stdio_transport_config() {
    let config = McpStdioServerConfig {
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
        working_directory: Some("/workspace".to_string()),
    };

    assert_eq!(config.command, "npx");
    assert_eq!(config.args.len(), 2);
    assert_eq!(
        config.working_directory.as_deref(),
        Some("/workspace")
    );
}

#[test]
fn test_http_transport_config() {
    let config = McpHttpServerConfig {
        endpoint: "https://api.example.com/mcp".to_string(),
        api_key_env: Some("MCP_API_KEY".to_string()),
        protocol_version: "2024-11-05".to_string(),
        http_headers: Default::default(),
        env_http_headers: Default::default(),
    };

    assert_eq!(config.endpoint, "https://api.example.com/mcp");
    assert_eq!(config.api_key_env.as_deref(), Some("MCP_API_KEY"));
    assert_eq!(config.protocol_version, "2024-11-05");
}

#[test]
fn test_transport_variants() {
    let stdio = McpTransportConfig::Stdio(McpStdioServerConfig {
        command: "echo".to_string(),
        args: vec![],
        working_directory: None,
    });

    let http = McpTransportConfig::Http(McpHttpServerConfig {
        endpoint: "http://localhost:3000".to_string(),
        api_key_env: None,
        protocol_version: "2024-11-05".to_string(),
        http_headers: Default::default(),
        env_http_headers: Default::default(),
    });

    match stdio {
        McpTransportConfig::Stdio(s) => assert_eq!(s.command, "echo"),
        _ => panic!("Expected Stdio variant"),
    }

    match http {
        McpTransportConfig::Http(h) => assert_eq!(h.endpoint, "http://localhost:3000"),
        _ => panic!("Expected Http variant"),
    }
}

#[test]
fn test_provider_name_validation_rules() {
    // Valid names
    let valid_names = vec![
        "provider1",
        "my-provider",
        "test_provider",
        "Provider123",
        "a",
    ];

    for name in valid_names {
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "Name '{}' should be valid",
            name
        );
    }

    // Invalid names
    let invalid_names = vec![
        "",           // empty
        "provider.1", // contains dot
        "provider 1", // contains space
        "provider/1", // contains slash
        "provider@1", // contains special char
    ];

    for name in invalid_names {
        let is_valid = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        assert!(!is_valid, "Name '{}' should be invalid", name);
    }
}
