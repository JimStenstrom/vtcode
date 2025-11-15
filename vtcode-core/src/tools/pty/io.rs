//! I/O utilities for PTY operations
//!
//! This module provides helper functions for:
//! - Path normalization and validation
//! - Exit status conversion
//! - Timeout clamping
//! - Environment setup
//! - Program detection

use crate::sandbox::SandboxProfile;
use crate::tools::path_env;
use anyhow::anyhow;
use portable_pty::{CommandBuilder, PtySize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

/// Clamp timeout to maximum representable value
///
/// Converts a Duration to milliseconds, clamping to u64::MAX if necessary.
pub(crate) fn clamp_timeout(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

/// Extract exit code from portable_pty ExitStatus
pub(crate) fn exit_status_code(status: portable_pty::ExitStatus) -> i32 {
    status.exit_code() as i32
}

/// Normalize a path by resolving `.` and `..` components
///
/// This does NOT access the filesystem - it performs purely lexical
/// path normalization.
///
/// # Arguments
///
/// * `path` - The path to normalize
///
/// # Returns
///
/// The normalized path
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

/// Set up the environment for a PTY command
///
/// Configures environment variables including:
/// - Inheriting parent process environment (including PATH)
/// - Merging extra paths into PATH
/// - Setting terminal-related variables
/// - Disabling color output
/// - Setting sandbox-related variables if applicable
///
/// # Arguments
///
/// * `builder` - The command builder to configure
/// * `program` - The program being executed
/// * `size` - The PTY size
/// * `workspace_root` - The workspace root directory
/// * `sandbox_profile` - Optional sandbox profile
/// * `extra_paths` - Additional paths to add to PATH
pub(crate) fn set_command_environment(
    builder: &mut CommandBuilder,
    program: &str,
    size: PtySize,
    workspace_root: &Path,
    sandbox_profile: Option<&SandboxProfile>,
    extra_paths: &[PathBuf],
) {
    // Inherit environment from parent process to preserve PATH and other important variables
    let mut env_map: HashMap<OsString, OsString> = std::env::vars_os().collect();
    let path_key = OsString::from("PATH");
    let current_path = env_map.get(&path_key).map(|value| value.as_os_str());
    if let Some(merged) = path_env::merge_path_env(current_path, extra_paths) {
        env_map.insert(path_key, merged);
    }

    for (key, value) in env_map {
        builder.env(key, value);
    }

    // Override or set specific environment variables for TTY
    builder.env("TERM", "xterm-256color");
    builder.env("PAGER", "cat");
    builder.env("GIT_PAGER", "cat");
    builder.env("LESS", "R");
    builder.env("COLUMNS", size.cols.to_string());
    builder.env("LINES", size.rows.to_string());
    builder.env("WORKSPACE_DIR", workspace_root.as_os_str());

    // Disable automatic color output from ls and other commands
    builder.env("CLICOLOR", "0");
    builder.env("CLICOLOR_FORCE", "0");
    builder.env("LS_COLORS", "");
    builder.env("NO_COLOR", "1");

    if let Some(profile) = sandbox_profile {
        builder.env("VT_SANDBOX_RUNTIME", profile.runtime_kind().as_str());
        builder.env("VT_SANDBOX_SETTINGS", profile.settings().as_os_str());
        builder.env(
            "VT_SANDBOX_PERSISTENT_DIR",
            profile.persistent_storage().as_os_str(),
        );
        if profile.allowed_paths().is_empty() {
            builder.env("VT_SANDBOX_ALLOWED_PATHS", "");
        } else {
            match std::env::join_paths(profile.allowed_paths()) {
                Ok(joined) => builder.env("VT_SANDBOX_ALLOWED_PATHS", joined),
                Err(_) => builder.env("VT_SANDBOX_ALLOWED_PATHS", ""),
            };
        }
    }

    if is_shell_program(program) {
        builder.env("SHELL", program);
    }
}

/// Check if a program is a shell
///
/// # Arguments
///
/// * `program` - The program name or path
///
/// # Returns
///
/// True if the program is a recognized shell
pub(crate) fn is_shell_program(program: &str) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "bash" | "sh" | "zsh" | "fish" | "dash" | "ash" | "busybox"
    )
}

/// Check if a program is a development toolchain command
///
/// Used to identify commands that might be in user-specific directories
/// and require shell PATH resolution.
///
/// # Arguments
///
/// * `program` - The program name or path
///
/// # Returns
///
/// True if the program is a recognized development tool
pub fn is_development_toolchain_command(program: &str) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "cargo"
            | "rustc"
            | "rustup"
            | "rustfmt"
            | "clippy"
            | "npm"
            | "node"
            | "yarn"
            | "pnpm"
            | "bun"
            | "go"
            | "python"
            | "python3"
            | "pip"
            | "pip3"
            | "java"
            | "javac"
            | "mvn"
            | "gradle"
            | "make"
            | "cmake"
            | "gcc"
            | "g++"
            | "clang"
            | "clang++"
    )
}

/// Ensure a path is within the workspace
///
/// # Arguments
///
/// * `candidate` - The path to check
/// * `workspace_root` - The workspace root directory
///
/// # Returns
///
/// An error if the path escapes the workspace
pub(crate) fn ensure_within_workspace(candidate: &Path, workspace_root: &Path) -> anyhow::Result<()> {
    let normalized = normalize_path(candidate);
    if !normalized.starts_with(workspace_root) {
        return Err(anyhow!(
            "Path '{}' escapes workspace '{}'",
            candidate.display(),
            workspace_root.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let path = Path::new("/foo/bar/../baz/./qux");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("/foo/baz/qux"));
    }

    #[test]
    fn test_is_shell_program() {
        assert!(is_shell_program("bash"));
        assert!(is_shell_program("/bin/sh"));
        assert!(is_shell_program("zsh"));
        assert!(!is_shell_program("ls"));
        assert!(!is_shell_program("cargo"));
    }

    #[test]
    fn test_is_development_toolchain_command() {
        assert!(is_development_toolchain_command("cargo"));
        assert!(is_development_toolchain_command("npm"));
        assert!(is_development_toolchain_command("python"));
        assert!(!is_development_toolchain_command("ls"));
        assert!(!is_development_toolchain_command("bash"));
    }
}
