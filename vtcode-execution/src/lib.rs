//! Code execution, sandbox management, and execution policy for VTCode
//!
//! This crate provides the execution layer for VTCode, enabling secure code execution,
//! sandbox management, and execution policy enforcement. It was extracted from `vtcode-core`
//! as part of Phase 2 of the architecture transformation to enable independent testing
//! and reusability.
//!
//! # Features
//!
//! - **Code Execution**: Execute Python/JavaScript code with MCP tool integration
//! - **Sandbox Management**: Configure secure execution environments with fine-grained permissions
//! - **Execution Policy**: Validate commands against security allow-lists and workspace boundaries
//! - **Async Commands**: Execute long-running processes with streaming output
//! - **Skills Management**: Save and reuse code functions across sessions
//! - **PII Detection**: Identify and tokenize personally identifiable information
//! - **Agent Optimization**: Analyze agent behavior patterns and suggest improvements
//! - **Tool Versioning**: Manage tool compatibility and migration paths
//!
//! # Architecture
//!
//! This crate depends only on foundation layers, maintaining a clean dependency hierarchy:
//!
//! ```text
//! vtcode-execution
//!     ├─→ vtcode-commons (utilities, safety validation)
//!     ├─→ vtcode-bash-runner (command execution primitives)
//!     ├─→ vtcode-tool-traits (MCP tool executor trait)
//!     └─→ vtcode-exec-events (execution event types)
//! ```
//!
//! This design ensures:
//! - No circular dependencies
//! - Independent compilation and testing
//! - Reusability in other projects
//! - Clear separation of concerns
//!
//! # Quick Start
//!
//! ## Code Execution with MCP Tools
//!
//! Execute code that can call MCP tools directly, reducing LLM round-trips:
//!
//! ```rust,ignore
//! use vtcode_execution::{CodeExecutor, Language};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! // Create executor with MCP client integration
//! let executor = CodeExecutor::new(
//!     Language::Python3,
//!     sandbox_profile,
//!     Arc::new(mcp_client),
//!     PathBuf::from("/workspace"),
//! );
//!
//! // Execute code that calls MCP tools
//! let code = r#"
//! # MCP tools are available as Python functions
//! files = list_files(path="/workspace", recursive=True)
//! filtered = [f for f in files if "test" in f]
//! result = {"count": len(filtered), "files": filtered[:10]}
//! "#;
//!
//! let result = executor.execute(code).await?;
//! println!("Found {} test files", result["count"]);
//! ```
//!
//! ## Sandbox Configuration
//!
//! Set up secure execution environments with precise control:
//!
//! ```rust,ignore
//! use vtcode_execution::sandbox::{SandboxEnvironment, SandboxRuntimeKind};
//!
//! // Build sandbox environment
//! let mut env = SandboxEnvironment::builder("./workspace")
//!     .sandbox_root("./.vtcode/sandbox")
//!     .runtime_kind(SandboxRuntimeKind::AnthropicSrt)
//!     .build();
//!
//! // Configure permissions
//! env.allow_domain("api.example.com")?;  // Allow network access
//! env.allow_path("logs")?;               // Allow file access
//! env.deny_path("secrets")?;             // Explicitly deny
//!
//! // Write settings and create profile
//! env.write_settings()?;
//! let profile = env.create_profile("/usr/local/bin/srt");
//! ```
//!
//! ## Command Validation
//!
//! Enforce security policies before executing commands:
//!
//! ```rust,ignore
//! use vtcode_execution::policy::{validate_command, sanitize_working_dir};
//! use std::path::Path;
//!
//! let workspace = Path::new("/workspace");
//! let working_dir = sanitize_working_dir(workspace, Some("./project")).await?;
//!
//! // Validate against allow-list
//! let command = vec!["git".to_string(), "status".to_string()];
//! validate_command(&command, workspace, &working_dir).await?;
//!
//! // Only approved commands with safe arguments pass validation
//! // Prevents: workspace breakout, destructive operations, unauthorized access
//! ```
//!
//! ## Async Process Execution
//!
//! Stream output from long-running commands:
//!
//! ```rust,ignore
//! use vtcode_execution::exec::async_command::{AsyncProcessRunner, ProcessOptions};
//! use std::time::Duration;
//!
//! let options = ProcessOptions {
//!     command: vec!["cargo".to_string(), "build".to_string()],
//!     working_dir: Some("/workspace".into()),
//!     timeout: Some(Duration::from_secs(300)),
//!     ..Default::default()
//! };
//!
//! let mut runner = AsyncProcessRunner::new(options);
//! let mut stream = runner.spawn().await?;
//!
//! // Stream output in real-time
//! while let Some(line) = stream.next().await {
//!     println!("Build: {}", line);
//! }
//! ```
//!
//! # Module Overview
//!
//! - [`exec`] - Code execution, async commands, skills, and agent optimization
//! - [`sandbox`] - Sandbox environment configuration and management
//! - [`policy`] - Command validation and security policy enforcement
//!
//! # Use Cases
//!
//! ## AI Agent Frameworks
//!
//! Add secure code execution to your AI agent:
//!
//! ```rust,ignore
//! // Agent generates code, executor runs it safely
//! let code = agent.generate_code(task)?;
//! let result = executor.execute(code).await?;
//! agent.process_result(result)?;
//! ```
//!
//! ## Code Analysis Tools
//!
//! Execute code analysis in sandboxed environments:
//!
//! ```rust,ignore
//! let env = SandboxEnvironment::builder(workspace)
//!     .deny_path("/etc")
//!     .deny_path("/root")
//!     .build();
//! ```
//!
//! ## Educational Platforms
//!
//! Run student code safely with resource limits:
//!
//! ```rust,ignore
//! let config = ExecutionConfig {
//!     timeout: Duration::from_secs(30),
//!     max_output_size: 10_000,
//!     ..Default::default()
//! };
//! ```
//!
//! ## Security Research
//!
//! Validate commands before execution:
//!
//! ```rust,ignore
//! // Ensure command is safe before running
//! validate_command(&cmd, workspace, working_dir).await?;
//! ```
//!
//! # Safety and Security
//!
//! This crate implements multiple layers of security:
//!
//! 1. **Command Allow-lists**: Only approved commands can execute
//! 2. **Argument Validation**: Prevent injection and breakout attempts
//! 3. **Workspace Boundaries**: Enforce path restrictions
//! 4. **Sandbox Integration**: Support for Anthropic's sandbox runtime
//! 5. **Resource Limits**: Timeout and output size constraints
//! 6. **PII Detection**: Identify sensitive data before processing
//!
//! # Performance
//!
//! - Async execution with tokio for efficient I/O
//! - Streaming output to avoid memory buildup
//! - Process pooling for repeated executions
//! - Minimal allocations in hot paths
//!
//! # Examples
//!
//! See the [`examples`](https://github.com/vinhnx/vtcode/tree/main/vtcode-execution/examples)
//! directory for complete working examples of:
//!
//! - Code execution with MCP tools
//! - Sandbox configuration
//! - Command validation
//! - Skills management
//! - Agent optimization
//!
//! # Testing
//!
//! Run tests with:
//!
//! ```bash
//! cargo test -p vtcode-execution
//! ```
//!
//! # Related Documentation
//!
//! - [Architecture Transformation Plan](https://github.com/vinhnx/vtcode/blob/main/ARCHITECTURE_TRANSFORMATION.md)
//! - [Phase 2 Completion Summary](https://github.com/vinhnx/vtcode/blob/main/vtcode-execution/PHASE2_VTCODE_EXECUTION_COMPLETE.md)
//! - [VTCode Main Documentation](https://docs.rs/vtcode-core)

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
