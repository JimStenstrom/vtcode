//! Reusable sandbox environment abstractions for secure code execution.
//!
//! This module packages the sandbox coordination logic used by VTCode into a
//! standalone, well-documented API that can be embedded in other projects.
//! It focuses on ergonomic configuration of sandbox permissions, event
//! logging, and runtime profile creation without relying on the surrounding
//! application state.
//!
//! # Features
//!
//! - **Builder Pattern**: Ergonomic configuration with `SandboxEnvironment::builder()`
//! - **Permission Management**: Fine-grained control over file and network access
//! - **Workspace Boundaries**: Automatic enforcement of workspace restrictions
//! - **Runtime Integration**: Compatible with Anthropic's sandbox runtime (srt)
//! - **Audit Logging**: Structured event logging for security auditing
//!
//! # Architecture
//!
//! The sandbox module consists of three main components:
//!
//! - [`environment`] - Configuration builder and permission management
//! - [`profile`] - Runtime profile generation for execution
//! - [`settings`] - JSON settings for sandbox runtime configuration
//!
//! # Quick Start
//!
//! ## Basic Sandbox Setup
//!
//! ```rust,no_run
//! use vtcode_execution::sandbox::{SandboxEnvironment, SandboxRuntimeKind};
//! # use anyhow::Result;
//!
//! # fn main() -> Result<()> {
//! // Create sandbox environment with builder
//! let mut env = SandboxEnvironment::builder("./workspace")
//!     .sandbox_root("./.vtcode/sandbox")
//!     .runtime_kind(SandboxRuntimeKind::AnthropicSrt)
//!     .build();
//!
//! // Configure permissions
//! env.allow_domain("api.example.com")?;
//! env.allow_path("logs")?;
//! env.deny_path("secrets")?;
//!
//! // Write settings file
//! env.write_settings()?;
//!
//! // Create execution profile
//! let profile = env.create_profile("/usr/local/bin/srt");
//! println!("Sandbox configured at {}", env.settings_path().display());
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Configuration
//!
//! ```rust,no_run
//! use vtcode_execution::sandbox::{SandboxEnvironment, DEFAULT_DENY_RULES};
//! # use anyhow::Result;
//!
//! # fn main() -> Result<()> {
//! let mut env = SandboxEnvironment::builder("./workspace")
//!     .sandbox_root("./.vtcode/sandbox")
//!     .settings_filename("custom-settings.json")
//!     .persistent_dir_name("data")
//!     .build();
//!
//! // Allow multiple domains
//! for domain in &["api.github.com", "registry.npmjs.org"] {
//!     env.allow_domain(domain)?;
//! }
//!
//! // Allow specific paths with workspace validation
//! env.allow_path("dist")?;
//! env.allow_path("node_modules")?;
//!
//! // Review and apply default deny rules
//! println!("Default deny rules: {:?}", DEFAULT_DENY_RULES);
//!
//! env.write_settings()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! The sandbox module enforces several security guarantees:
//!
//! 1. **Workspace Isolation**: All paths are validated against workspace boundaries
//! 2. **Default Deny**: Sensitive paths are denied by default (SSH keys, /etc, /root)
//! 3. **Explicit Allow**: Permissions must be explicitly granted
//! 4. **Path Normalization**: Prevents traversal attacks (../, symlinks)
//! 5. **Audit Trail**: All permission changes are logged
//!
//! # Runtime Integration
//!
//! The sandbox settings are compatible with Anthropic's sandbox runtime (srt):
//!
//! ```bash
//! # Generated settings.json can be used with srt
//! srt --config .vtcode/sandbox/settings.json -- python script.py
//! ```
//!
//! # Use Cases
//!
//! ## AI Agent Sandboxing
//!
//! Restrict agent code execution to specific directories and networks:
//!
//! ```rust,ignore
//! let mut env = SandboxEnvironment::builder(workspace).build();
//! env.allow_path("src")?;
//! env.allow_path("tests")?;
//! env.deny_path("production-data")?;
//! ```
//!
//! ## Educational Platforms
//!
//! Run student code with strict isolation:
//!
//! ```rust,ignore
//! let mut env = SandboxEnvironment::builder(student_workspace)
//!     .deny_path("/etc")
//!     .deny_path("/root")
//!     .build();
//! ```
//!
//! ## Code Analysis Tools
//!
//! Analyze untrusted code safely:
//!
//! ```rust,ignore
//! let env = SandboxEnvironment::builder(temp_workspace)
//!     .build(); // No network access by default
//! ```

mod environment;
mod profile;
mod settings;

#[cfg(test)]
mod tests;

pub use environment::{
    DEFAULT_DENY_RULES, DomainAddition, DomainRemoval, PathAddition, PathRemoval,
    SandboxEnvironment, SandboxEnvironmentBuilder,
};
pub use profile::{SandboxProfile, SandboxRuntimeKind};
pub use settings::{
    SandboxNetworkPermissions, SandboxPermissions, SandboxRuntimeConfig, SandboxSettings,
};
