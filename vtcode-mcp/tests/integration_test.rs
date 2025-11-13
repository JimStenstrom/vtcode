//! Integration tests for vtcode-mcp
//!
//! These tests verify the complete MCP client functionality including
//! configuration, validation, and basic operations.

use vtcode_config::mcp::{
    McpAllowListConfig, McpClientConfig, McpProviderConfig, McpServerConfig,
    McpStdioServerConfig, McpTransportConfig,
};
use vtcode_mcp::{
    EnhancedMcpSecurityConfig, McpClient, ValidatedMcpClientConfig, ValidationError,
};

/// Create a minimal test configuration
fn create_test_config() -> McpClientConfig {
    McpClientConfig {
        enabled: true,
        ui: Default::default(),
        providers: vec![],
        server: McpServerConfig {
            enabled: false,
            bind_address: "127.0.0.1".to_string(),
            port: 3000,
            transport: vtcode_config::mcp::McpServerTransport::Sse,
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            exposed_tools: vec![],
        },
        allowlist: McpAllowListConfig::default(),
        max_concurrent_connections: 10,
        request_timeout_seconds: 30,
        retry_attempts: 3,
        startup_timeout_seconds: Some(60),
        tool_timeout_seconds: Some(300),
        experimental_use_rmcp_client: true,
        security: Default::default(),
    }
}

#[test]
fn test_client_creation() {
    let config = create_test_config();
    let client = McpClient::new(config);
    let status = client.get_status();

    assert!(status.enabled);
    assert_eq!(status.provider_count, 0);
    assert_eq!(status.active_connections, 0);
}

#[test]
fn test_client_with_disabled_config() {
    let mut config = create_test_config();
    config.enabled = false;

    let client = McpClient::new(config);
    let status = client.get_status();

    assert!(!status.enabled);
}

#[test]
fn test_validated_config_creation() {
    let config = create_test_config();
    let validated = ValidatedMcpClientConfig::new(config);

    assert!(validated.is_valid());
    assert_eq!(validated.validate().len(), 0);
}

#[test]
fn test_validation_invalid_port() {
    let mut config = create_test_config();
    config.server.enabled = true;
    config.server.port = 0;

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(!errors.is_empty());
    assert!(matches!(errors[0], ValidationError::InvalidPort(_)));
}

#[test]
fn test_validation_empty_bind_address() {
    let mut config = create_test_config();
    config.server.enabled = true;
    config.server.bind_address = String::new();

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::EmptyBindAddress)));
}

#[test]
fn test_validation_startup_timeout_too_long() {
    let mut config = create_test_config();
    config.startup_timeout_seconds = Some(400); // > 300 seconds

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidStartupTimeout(_))));
}

#[test]
fn test_validation_tool_timeout_too_long() {
    let mut config = create_test_config();
    config.tool_timeout_seconds = Some(4000); // > 3600 seconds

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidToolTimeout(_))));
}

#[test]
fn test_validation_zero_concurrent_requests() {
    let mut config = create_test_config();
    config.providers.push(McpProviderConfig {
        name: "test".to_string(),
        transport: McpTransportConfig::Stdio(McpStdioServerConfig {
            command: "echo".to_string(),
            args: vec![],
            working_directory: None,
        }),
        env: Default::default(),
        enabled: true,
        max_concurrent_requests: 0, // Invalid
        startup_timeout_ms: None,
    });

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidMaxConcurrentRequests(_, _))));
}

#[test]
fn test_validation_empty_provider_name() {
    let mut config = create_test_config();
    config.providers.push(McpProviderConfig {
        name: String::new(), // Invalid
        transport: McpTransportConfig::Stdio(McpStdioServerConfig {
            command: "echo".to_string(),
            args: vec![],
            working_directory: None,
        }),
        env: Default::default(),
        enabled: true,
        max_concurrent_requests: 5,
        startup_timeout_ms: None,
    });

    let validated = ValidatedMcpClientConfig::new(config);

    assert!(!validated.is_valid());
    let errors = validated.validate();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::EmptyProviderName)));
}

#[test]
fn test_enhanced_security_config_defaults() {
    let security = EnhancedMcpSecurityConfig::default();

    assert!(!security.auth_enabled);
    assert!(security.api_key_env.is_none());
    assert_eq!(security.rate_limit.requests_per_minute, 100);
    assert_eq!(security.rate_limit.concurrent_requests, 10);
    assert!(security.validation.schema_validation_enabled);
    assert!(security.validation.path_traversal_protection);
    assert_eq!(security.validation.max_argument_size, 1024 * 1024); // 1MB
}

#[test]
fn test_allowlist_default_mode() {
    let allowlist = McpAllowListConfig::default();

    // Default should allow all
    assert!(allowlist.is_tool_allowed("any_provider", "any_tool"));
    assert!(allowlist.is_resource_allowed("any_provider", "any_resource"));
    assert!(allowlist.is_prompt_allowed("any_provider", "any_prompt"));
}

#[tokio::test]
async fn test_client_list_tools_empty() {
    let config = create_test_config();
    let client = McpClient::new(config);

    // Without initialization, should return empty list
    let result = client.list_tools().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_client_execute_tool_when_disabled() {
    let mut config = create_test_config();
    config.enabled = false;

    let client = McpClient::new(config);

    let result = client
        .execute_tool("test_tool", serde_json::json!({}))
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("MCP support is disabled"));
}

#[test]
fn test_validation_error_display() {
    let errors = vec![
        ValidationError::InvalidPort(99999),
        ValidationError::EmptyBindAddress,
        ValidationError::MissingApiKeyEnv,
        ValidationError::InvalidStartupTimeout(400),
        ValidationError::InvalidToolTimeout(5000),
        ValidationError::EmptyProviderName,
        ValidationError::InvalidMaxConcurrentRequests("test".to_string(), 0),
    ];

    for error in errors {
        let message = error.to_string();
        assert!(!message.is_empty());
        // Each error should have meaningful text
        assert!(message.len() > 10);
    }
}

#[test]
fn test_client_status_with_providers() {
    let mut config = create_test_config();
    config.providers.push(McpProviderConfig {
        name: "provider1".to_string(),
        transport: McpTransportConfig::Stdio(McpStdioServerConfig {
            command: "echo".to_string(),
            args: vec![],
            working_directory: None,
        }),
        env: Default::default(),
        enabled: true,
        max_concurrent_requests: 5,
        startup_timeout_ms: None,
    });

    let client = McpClient::new(config);
    let status = client.get_status();

    assert!(status.enabled);
    // Provider count is 0 until initialized
    assert_eq!(status.provider_count, 0);
}
