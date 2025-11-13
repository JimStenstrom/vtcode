//! Code execution, sandbox management, and execution policy for VTCode
//!
//! This crate provides the execution layer for VTCode, including:
//!
//! - **Code Execution**: Run code snippets in various languages with MCP tool integration
//! - **Sandbox Management**: Configure and manage sandboxed execution environments
//! - **Execution Policy**: Validate and enforce security policies for command execution
//! - **Async Commands**: Execute system commands asynchronously with streaming output
//! - **Skills & Versioning**: Manage agent skills and tool version compatibility
//!
//! # Architecture
//!
//! This crate was extracted from `vtcode-core` as part of Phase 2 of the architecture
//! transformation. It depends only on foundation crates:
//!
//! - `vtcode-commons` - Shared utilities and safety validation
//! - `vtcode-bash-runner` - Bash command execution
//! - `vtcode-tool-traits` - Tool system traits and MCP interfaces
//! - `vtcode-exec-events` - Execution event types
//!
//! # Example: Code Execution
//!
//! ```rust,ignore
//! use vtcode_execution::exec::{CodeExecutor, Language, ExecutionConfig};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! let executor = CodeExecutor::new(
//!     Language::Python3,
//!     sandbox_profile,
//!     Arc::new(mcp_client),
//!     PathBuf::from("/workspace"),
//! );
//!
//! let code = r#"
//! result = {"hello": "world"}
//! "#;
//!
//! let result = executor.execute(code).await?;
//! ```
//!
//! # Example: Sandbox Management
//!
//! ```rust,ignore
//! use vtcode_execution::sandbox::{SandboxEnvironment, SandboxRuntimeKind};
//!
//! let mut environment = SandboxEnvironment::builder("./workspace")
//!     .sandbox_root("./.vtcode/sandbox")
//!     .runtime_kind(SandboxRuntimeKind::AnthropicSrt)
//!     .build();
//!
//! environment.allow_domain("example.com")?;
//! environment.allow_path("logs")?;
//! environment.write_settings()?;
//! ```
//!
//! # Example: Execution Policy
//!
//! ```rust,ignore
//! use vtcode_execution::policy::validate_command;
//! use std::path::Path;
//!
//! let workspace = Path::new("/workspace");
//! let working_dir = Path::new("/workspace/project");
//! let command = vec!["git".to_string(), "status".to_string()];
//!
//! validate_command(&command, workspace, working_dir).await?;
//! ```

pub mod exec;
pub mod policy;
pub mod sandbox;

// Re-export commonly used types
pub use exec::{
    agent_optimization::{
        AgentBehaviorAnalyzer, CodePattern, FailurePatterns, RecoveryPattern, SkillStatistics,
        ToolStatistics,
    },
    async_command::{AsyncProcessRunner, ProcessOptions, StreamCaptureConfig},
    cancellation::CancellationToken,
    code_executor::{CodeExecutor, ExecutionConfig, ExecutionResult, Language},
    pii_tokenizer::{DetectedPii, PiiToken, PiiTokenizer, PiiType},
    sdk_ipc::{ToolIpcHandler, ToolRequest as SdkToolRequest, ToolResponse as SdkToolResponse},
    skill_manager::{Skill, SkillManager, SkillMetadata},
    tool_versioning::{
        BreakingChange, CompatibilityReport, Deprecation, Migration, SkillCompatibilityChecker,
        ToolDependency, ToolVersion, VersionCompatibility,
    },
};

pub use policy::{sanitize_working_dir, validate_command};

pub use sandbox::{
    environment::{
        DomainAddition, DomainRemoval, PathAddition, PathRemoval, SandboxEnvironment,
        SandboxEnvironmentBuilder, DEFAULT_DENY_RULES,
    },
    profile::{SandboxProfile, SandboxRuntimeKind},
    settings::{
        SandboxNetworkPermissions, SandboxPermissions, SandboxRuntimeConfig, SandboxSettings,
    },
};
