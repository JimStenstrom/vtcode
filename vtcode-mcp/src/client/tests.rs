//! Tests for MCP client functionality.

#[cfg(test)]
mod tests {
    use crate::client::providers::provider::{
        ensure_timezone_argument, schema_requires_field,
    };
    use crate::client::providers::handler::{
        build_elicitation_validator, validate_elicitation_payload,
    };
    use mcp_types::{ClientCapabilities, ClientCapabilitiesRoots, Implementation, InitializeRequestParams};
    use rmcp::model::ElicitationAction;
    use serde_json::{Map, Value, json};
    use std::collections::HashMap;
    use vtcode_config::mcp::{McpProviderConfig, McpStdioServerConfig, McpTransportConfig};

    const TIMEZONE_ARGUMENT: &str = "timezone";
    const LOCAL_TIMEZONE_ENV_VAR: &str = "VT_LOCAL_TIMEZONE";

    struct EnvGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            // SAFETY: Tests provide well-formed UTF-8 values and restore the
            // original value (if any) before dropping the guard, matching the
            // documented requirements for manipulating the process
            // environment.
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref original) = self.original {
                // SAFETY: Restores the previous UTF-8 environment value that
                // existed when the guard was created.
                unsafe {
                    std::env::set_var(self.key, original);
                }
            } else {
                // SAFETY: Removing the variable is safe because the guard is
                // the only code path mutating it during the test's lifetime.
                unsafe {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn schema_detection_handles_required_entries() {
        let schema = json!({
            "type": "object",
            "required": [TIMEZONE_ARGUMENT],
            "properties": {
                TIMEZONE_ARGUMENT: { "type": "string" }
            }
        });

        assert!(schema_requires_field(&schema, TIMEZONE_ARGUMENT));
        assert!(!schema_requires_field(&schema, "location"));
    }

    #[test]
    fn ensure_timezone_injects_from_override_env() {
        let _guard = EnvGuard::set(LOCAL_TIMEZONE_ENV_VAR, "Etc/UTC");
        let mut arguments = Map::new();

        ensure_timezone_argument(&mut arguments, true).unwrap();

        assert_eq!(
            arguments.get(TIMEZONE_ARGUMENT).and_then(Value::as_str),
            Some("Etc/UTC")
        );
    }

    #[test]
    fn ensure_timezone_does_not_override_existing_value() {
        let mut arguments = Map::new();
        arguments.insert(
            TIMEZONE_ARGUMENT.to_string(),
            Value::String("America/New_York".to_string()),
        );

        ensure_timezone_argument(&mut arguments, true).unwrap();

        assert_eq!(
            arguments.get(TIMEZONE_ARGUMENT).and_then(Value::as_str),
            Some("America/New_York")
        );
    }

    #[tokio::test]
    async fn convert_to_rmcp_round_trip() {
        use super::super::providers::rmcp_client::convert_to_mcp;
        use super::super::providers::rmcp_client::convert_to_rmcp;

        let params = InitializeRequestParams {
            capabilities: ClientCapabilities {
                roots: Some(ClientCapabilitiesRoots {
                    list_changed: Some(true),
                }),
                ..Default::default()
            },
            client_info: Implementation {
                name: "vtcode".to_string(),
                version: "1.0".to_string(),
            },
            protocol_version: mcp_types::LATEST_PROTOCOL_VERSION.to_string(),
        };

        let converted: rmcp::model::InitializeRequestParam =
            convert_to_rmcp(params.clone()).unwrap();
        let round_trip: InitializeRequestParams = convert_to_mcp(converted).unwrap();
        assert_eq!(round_trip.client_info.name, "vtcode");
        assert_eq!(round_trip.client_info.version, "1.0");
    }

    #[test]
    fn validate_elicitation_payload_rejects_invalid_content() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        let validator =
            build_elicitation_validator("test", &schema).expect("schema should compile");

        let result = validate_elicitation_payload(
            "test",
            Some(&validator),
            &ElicitationAction::Accept,
            Some(&json!({ "name": 42 })),
        );

        assert!(result.is_err());
    }

    #[test]
    fn validate_elicitation_payload_accepts_valid_content() {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": { "type": "string", "format": "email" }
            },
            "required": ["email"]
        });
        let validator =
            build_elicitation_validator("test", &schema).expect("schema should compile");

        let result = validate_elicitation_payload(
            "test",
            Some(&validator),
            &ElicitationAction::Accept,
            Some(&json!({ "email": "user@example.com" })),
        );

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn provider_max_concurrency_defaults_to_one() {
        use crate::client::providers::McpProvider;

        let config = McpProviderConfig {
            name: "test".into(),
            transport: McpTransportConfig::Stdio(McpStdioServerConfig {
                command: "cat".into(),
                args: vec![],
                working_directory: None,
            }),
            env: HashMap::new(),
            enabled: true,
            max_concurrent_requests: 0,
            startup_timeout_ms: None,
        };

        let provider = McpProvider::connect(config, None).await.unwrap();
        assert_eq!(provider.semaphore().available_permits(), 1);
    }

    #[test]
    fn directory_to_file_uri_generates_file_scheme() {
        use super::super::providers::handler::directory_to_file_uri;

        let temp_dir = std::env::temp_dir();
        let uri = directory_to_file_uri(temp_dir.as_path())
            .expect("should create file uri for temp directory");
        assert!(uri.starts_with("file://"));
    }
}
