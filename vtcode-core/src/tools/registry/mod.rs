//! Tool registry and function declarations

// Submodules
mod approval_recorder;
mod builtins;
mod cache;
mod core;
mod declarations;
mod dispatch;
mod error;
mod executors;
mod inventory;
mod justification;
mod justification_extractor;
mod legacy;
mod lifecycle;
mod lookup;
mod mcp;
mod policy;
mod pty;
mod registration;
mod risk_scorer;
mod telemetry;
mod timeout;
mod utils;
pub mod validation;

// Re-exports
pub use approval_recorder::ApprovalRecorder;
pub use core::ToolRegistry;
pub use declarations::{
    build_function_declarations, build_function_declarations_for_level,
    build_function_declarations_with_mode,
};
pub use dispatch::ToolPermissionDecision;
pub use error::{ToolErrorType, ToolExecutionError, classify_error};
pub use justification::{ApprovalPattern, JustificationManager, ToolJustification};
pub use justification_extractor::JustificationExtractor;
pub use registration::{ToolExecutorFn, ToolHandler, ToolRegistration};
pub use risk_scorer::{RiskLevel, ToolRiskContext, ToolRiskScorer, ToolSource, WorkspaceTrust};
pub use telemetry::ToolTelemetryEvent;
pub use timeout::{ToolTimeoutCategory, ToolTimeoutPolicy};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::constants::tools;
    use crate::config::types::CapabilityLevel;
    use crate::tools::traits::Tool;
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::time::Duration;
    use tempfile::TempDir;

    const CUSTOM_TOOL_NAME: &str = "custom_test_tool";

    struct CustomEchoTool;

    #[async_trait]
    impl Tool for CustomEchoTool {
        async fn execute(&self, args: Value) -> anyhow::Result<Value> {
            Ok(json!({
                "success": true,
                "args": args,
            }))
        }

        fn name(&self) -> &'static str {
            CUSTOM_TOOL_NAME
        }

        fn description(&self) -> &'static str {
            "Custom echo tool for testing"
        }
    }

    #[tokio::test]
    async fn registers_builtin_tools() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let registry = ToolRegistry::new(temp_dir.path().to_path_buf()).await;
        let available = registry.available_tools().await;

        assert!(available.contains(&tools::READ_FILE.to_string()));
        assert!(available.contains(&tools::RUN_COMMAND.to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn allows_registering_custom_tools() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let mut registry = ToolRegistry::new(temp_dir.path().to_path_buf()).await;

        registry.register_tool(ToolRegistration::from_tool_instance(
            CUSTOM_TOOL_NAME,
            CapabilityLevel::CodeSearch,
            CustomEchoTool,
        ))?;

        registry.allow_all_tools().await.ok();

        let available = registry.available_tools().await;
        assert!(available.contains(&CUSTOM_TOOL_NAME.to_string()));

        let response = registry
            .execute_tool(CUSTOM_TOOL_NAME, json!({"input": "value"}))
            .await?;
        assert!(response["success"].as_bool().unwrap_or(false));
        Ok(())
    }

    #[tokio::test]
    async fn full_auto_allowlist_enforced() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let mut registry = ToolRegistry::new(temp_dir.path().to_path_buf()).await;

        registry.enable_full_auto_mode(&vec![tools::READ_FILE.to_string()]);

        assert!(registry.preflight_tool_permission(tools::READ_FILE).await?);
        assert!(
            !registry
                .preflight_tool_permission(tools::RUN_COMMAND)
                .await?
        );

        Ok(())
    }
}
