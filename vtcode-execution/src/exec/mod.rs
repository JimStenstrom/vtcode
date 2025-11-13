//! Execution module providing code execution, async commands, and agent optimization.
//!
//! This module contains the core execution functionality including:
//!
//! # Submodules
//!
//! - [`code_executor`] - Execute Python/JavaScript code with MCP tool integration
//! - [`async_command`] - Async process execution with streaming output
//! - [`skill_manager`] - Persistent skill storage and retrieval
//! - [`pii_tokenizer`] - PII detection and tokenization
//! - [`agent_optimization`] - Agent behavior analysis and optimization
//! - [`tool_versioning`] - Tool compatibility and version management
//! - [`sdk_ipc`] - Inter-process communication for tool SDKs
//! - [`cancellation`] - Cancellation token support for async operations
//!
//! # Examples
//!
//! ## Code Execution
//!
//! ```rust,ignore
//! use vtcode_execution::exec::{CodeExecutor, Language};
//!
//! let executor = CodeExecutor::new(
//!     Language::Python3,
//!     sandbox,
//!     mcp_client,
//!     workspace,
//! );
//!
//! let result = executor.execute("result = {'status': 'ok'}").await?;
//! ```
//!
//! ## Agent Behavior Analysis
//!
//! ```rust,ignore
//! use vtcode_execution::exec::AgentBehaviorAnalyzer;
//!
//! let mut analyzer = AgentBehaviorAnalyzer::new();
//! analyzer.record_tool_call("read_file", true, duration);
//!
//! let stats = analyzer.get_tool_statistics();
//! let recommendations = analyzer.suggest_optimizations();
//! ```

pub mod agent_optimization;
pub mod async_command;
pub mod cancellation;
pub mod code_executor;
pub mod events;
pub mod integration_tests;
pub mod pii_tokenizer;
pub mod sdk_ipc;
pub mod skill_manager;
pub mod tool_versioning;

pub use agent_optimization::{
    AgentBehaviorAnalyzer, CodePattern, FailurePatterns, RecoveryPattern, SkillStatistics,
    ToolStatistics,
};
pub use async_command::{AsyncProcessRunner, ProcessOptions, StreamCaptureConfig};
pub use cancellation::{current_tool_cancellation, with_tool_cancellation, CancellationToken};
pub use code_executor::{CodeExecutor, ExecutionConfig, ExecutionResult, Language};
pub use pii_tokenizer::{DetectedPii, PiiToken, PiiTokenizer, PiiType};
pub use sdk_ipc::{ToolIpcHandler, ToolRequest, ToolResponse};
pub use skill_manager::{Skill, SkillManager, SkillMetadata};
pub use tool_versioning::{
    BreakingChange, CompatibilityReport, Deprecation, Migration, SkillCompatibilityChecker,
    ToolDependency, ToolVersion, VersionCompatibility,
};
