//! Execution policy validation and security enforcement.
//!
//! This module provides command validation and security policy enforcement to prevent
//! workspace breakout, destructive operations, and unauthorized access. It implements
//! a curated allow-list of safe commands with argument validation.
//!
//! # Features
//!
//! - **Command Allow-list**: Only approved commands can execute
//! - **Argument Validation**: Prevent injection and dangerous arguments
//! - **Workspace Boundaries**: Enforce path restrictions
//! - **Path Normalization**: Prevent traversal attacks (../, symlinks)
//! - **File Validation**: Ensure files exist before operations
//!
//! # Security Model
//!
//! The policy follows a default-deny approach:
//!
//! 1. Only explicitly allowed commands can run
//! 2. All file paths must be within workspace boundaries
//! 3. Dangerous arguments are rejected (eval, arbitrary execution)
//! 4. Paths are normalized to prevent traversal
//! 5. File existence is validated before operations
//!
//! # Allowed Commands
//!
//! The following commands are permitted with validation:
//!
//! ## File Operations
//! - `cat` - Read file contents
//! - `head` - Read file beginning
//! - `tail` - Read file end
//! - `ls` - List directory contents
//! - `cp` - Copy files
//! - `wc` - Count words/lines
//!
//! ## Search
//! - `grep` - Text search
//! - `rg` (ripgrep) - Fast text search
//! - `sed` - Stream editor
//!
//! ## Development Tools
//! - `git` - Version control operations
//! - `cargo` - Rust package manager
//! - `npm` - Node package manager
//! - `python`/`python3` - Python interpreter
//! - `node` - Node.js runtime
//!
//! ## System Info
//! - `echo` - Print text
//! - `pwd` - Print working directory
//! - `printenv` - Print environment variables
//! - `which` - Locate programs
//! - `date` - Display date/time
//! - `whoami` - Current user
//! - `hostname` - System hostname
//! - `uname` - System information
//!
//! # Examples
//!
//! ## Basic Validation
//!
//! ```rust,ignore
//! use vtcode_execution::policy::validate_command;
//! use std::path::Path;
//!
//! let workspace = Path::new("/workspace");
//! let working_dir = Path::new("/workspace/project");
//!
//! // Safe command - allowed
//! let cmd = vec!["git".to_string(), "status".to_string()];
//! validate_command(&cmd, workspace, working_dir).await?;
//!
//! // Unsafe command - rejected
//! let cmd = vec!["rm".to_string(), "-rf".to_string(), "/".to_string()];
//! assert!(validate_command(&cmd, workspace, working_dir).await.is_err());
//! ```
//!
//! ## Working Directory Sanitization
//!
//! ```rust,ignore
//! use vtcode_execution::policy::sanitize_working_dir;
//! use std::path::Path;
//!
//! let workspace = Path::new("/workspace");
//!
//! // Valid subdirectory
//! let dir = sanitize_working_dir(workspace, Some("./project")).await?;
//! assert_eq!(dir, Path::new("/workspace/project"));
//!
//! // Attempt to escape - rejected
//! let result = sanitize_working_dir(workspace, Some("../../etc")).await;
//! assert!(result.is_err());
//! ```
//!
//! ## Path Validation
//!
//! ```rust,ignore
//! // All paths are validated against workspace boundaries
//! let cmd = vec!["cat".to_string(), "src/main.rs".to_string()];
//! validate_command(&cmd, workspace, working_dir).await?;
//!
//! // Paths outside workspace are rejected
//! let cmd = vec!["cat".to_string(), "/etc/passwd".to_string()];
//! assert!(validate_command(&cmd, workspace, working_dir).await.is_err());
//! ```
//!
//! # Use Cases
//!
//! ## AI Agent Execution
//!
//! Validate commands before executing agent-generated code:
//!
//! ```rust,ignore
//! let command = agent.generate_command()?;
//! validate_command(&command, workspace, working_dir).await?;
//! execute_command(&command).await?;
//! ```
//!
//! ## Educational Platforms
//!
//! Ensure student code only accesses approved resources:
//!
//! ```rust,ignore
//! // Students can only run approved commands
//! // within their workspace directory
//! validate_command(&student_command, student_workspace, working_dir).await?;
//! ```
//!
//! ## Code Analysis
//!
//! Safe execution of analysis tools:
//!
//! ```rust,ignore
//! let cmd = vec!["cargo".to_string(), "check".to_string()];
//! validate_command(&cmd, workspace, working_dir).await?;
//! ```
//!
//! # Security Considerations
//!
//! This policy prevents common attack vectors:
//!
//! - **Command Injection**: Only allowed commands execute
//! - **Path Traversal**: Paths are normalized and validated
//! - **Workspace Breakout**: All paths must be within workspace
//! - **Arbitrary Code Execution**: Dangerous flags are rejected (eval, -c)
//! - **Destructive Operations**: Commands like rm, chmod are not allowed
//!
//! # Extending the Policy
//!
//! To add new allowed commands:
//!
//! 1. Create a validator in the appropriate module (or new module)
//! 2. Add command to the match statement in `core::validate_command`
//! 3. Validate all arguments and file paths
//! 4. Ensure workspace boundary enforcement
//! 5. Document allowed flags and behavior
//! 6. Add tests to verify security properties

mod core;
mod paths;
mod workspace;
mod validators;

// Re-export public API
pub use core::validate_command;
pub use workspace::sanitize_working_dir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_echo() {
        assert!(validators::system::validate_echo(&[]).is_ok());
        assert!(validators::system::validate_echo(&["hello".to_string()]).is_ok());
        assert!(validators::system::validate_echo(&["-n".to_string(), "hello".to_string()]).is_ok());
        assert!(validators::system::validate_echo(&["-e".to_string(), "test".to_string()]).is_ok());
        assert!(validators::system::validate_echo(&["--invalid".to_string()]).is_err());
    }

    #[test]
    fn test_validate_pwd() {
        assert!(validators::system::validate_pwd(&[]).is_ok());
        assert!(validators::system::validate_pwd(&["arg".to_string()]).is_err());
    }

    #[test]
    fn test_validate_printenv() {
        assert!(validators::system::validate_printenv(&[]).is_ok());
        assert!(validators::system::validate_printenv(&["PATH".to_string()]).is_ok());
        assert!(validators::system::validate_printenv(&["MY_VAR_123".to_string()]).is_ok());
        assert!(validators::system::validate_printenv(&["MY-VAR".to_string()]).is_err());
        assert!(validators::system::validate_printenv(&["MY VAR".to_string()]).is_err());
        assert!(validators::system::validate_printenv(&["VAR1".to_string(), "VAR2".to_string()]).is_err());
    }

    #[tokio::test]
    async fn test_validate_git_read_only() {
        // Safe read-only operations
        assert!(validators::git::validate_git_read_only("status", &[]).is_ok());
        assert!(validators::git::validate_git_read_only("log", &["--oneline".to_string()]).is_ok());
        assert!(validators::git::validate_git_read_only("diff", &["-p".to_string()]).is_ok());
        assert!(validators::git::validate_git_read_only("show", &["HEAD".to_string()]).is_ok());
        assert!(validators::git::validate_git_read_only("branch", &["-a".to_string()]).is_ok());

        // Dangerous patterns blocked
        assert!(
            validators::git::validate_git_read_only("log", &["--format".to_string(), "test;cat".to_string()])
                .is_err()
        );
    }

    #[test]
    fn test_validate_git_commit() {
        // Valid commits
        assert!(validators::git::validate_git_commit(&["-m".to_string(), "fix: test".to_string()]).is_ok());
        assert!(validators::git::validate_git_commit(&["-a".to_string()]).is_ok());
        assert!(validators::git::validate_git_commit(&["--amend".to_string()]).is_ok());

        // Invalid commits
        assert!(validators::git::validate_git_commit(&["-m".to_string()]).is_err()); // Missing message
        assert!(validators::git::validate_git_commit(&["--invalid-flag".to_string()]).is_err());
    }

    #[test]
    fn test_validate_git_reset() {
        // Safe reset modes
        assert!(validators::git::validate_git_reset(&["--soft".to_string()]).is_ok());
        assert!(validators::git::validate_git_reset(&["--mixed".to_string()]).is_ok());
        assert!(validators::git::validate_git_reset(&["--unstage".to_string()]).is_ok());
        assert!(validators::git::validate_git_reset(&[]).is_ok());

        // Dangerous reset modes
        assert!(validators::git::validate_git_reset(&["--hard".to_string()]).is_err());
        assert!(validators::git::validate_git_reset(&["--merge".to_string()]).is_err());
        assert!(validators::git::validate_git_reset(&["--keep".to_string()]).is_err());
    }

    #[test]
    fn test_validate_git_stash() {
        // Safe stash operations
        assert!(validators::git::validate_git_stash(&["list".to_string()]).is_ok());
        assert!(validators::git::validate_git_stash(&["show".to_string()]).is_ok());
        assert!(validators::git::validate_git_stash(&["pop".to_string()]).is_ok());
        assert!(validators::git::validate_git_stash(&["apply".to_string()]).is_ok());
        assert!(validators::git::validate_git_stash(&["drop".to_string()]).is_ok());

        // Dangerous operations
        assert!(validators::git::validate_git_stash(&["push".to_string()]).is_err());
        assert!(validators::git::validate_git_stash(&["save".to_string()]).is_err());
    }

    #[tokio::test]
    async fn test_validate_git_safe_operations() {
        let workspace = std::path::PathBuf::from("/tmp");
        let working = std::path::PathBuf::from("/tmp");

        // Safe read-only operations should be allowed
        assert!(
            validators::git::validate_git(&["status".to_string()], &workspace, &working)
                .await
                .is_ok()
        );
        assert!(
            validators::git::validate_git(
                &["log".to_string(), "--oneline".to_string()],
                &workspace,
                &working
            )
            .await
            .is_ok()
        );
        assert!(
            validators::git::validate_git(&["diff".to_string()], &workspace, &working)
                .await
                .is_ok()
        );
        assert!(
            validators::git::validate_git(
                &["show".to_string(), "HEAD".to_string()],
                &workspace,
                &working
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validate_git_dangerous_operations_blocked() {
        let workspace = std::path::PathBuf::from("/tmp");
        let working = std::path::PathBuf::from("/tmp");

        // Dangerous operations should be blocked
        assert!(
            validators::git::validate_git(
                &["push".to_string(), "--force".to_string()],
                &workspace,
                &working
            )
            .await
            .is_err()
        );
        assert!(
            validators::git::validate_git(
                &["push".to_string(), "-f".to_string()],
                &workspace,
                &working
            )
            .await
            .is_err()
        );
        assert!(
            validators::git::validate_git(&["clean".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validators::git::validate_git(&["filter-branch".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validators::git::validate_git(&["rebase".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validators::git::validate_git(&["cherry-pick".to_string()], &workspace, &working)
                .await
                .is_err()
        );
    }

    #[test]
    fn test_validate_which() {
        assert!(validators::system::validate_which(&["ls".to_string()]).is_ok());
        assert!(validators::system::validate_which(&["git".to_string(), "-a".to_string()]).is_ok());
        assert!(validators::system::validate_which(&[]).is_err());
        assert!(validators::system::validate_which(&["/usr/bin/ls".to_string()]).is_err()); // Contains /
        assert!(validators::system::validate_which(&["ls git".to_string()]).is_err()); // Contains space
    }
}
