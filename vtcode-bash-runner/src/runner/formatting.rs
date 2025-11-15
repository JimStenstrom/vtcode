//! Shell-specific command formatting

use shell_escape::escape;
use std::path::Path;

use crate::executor::ShellKind;

/// Format path for shell
pub fn format_path(shell: ShellKind, path: &Path) -> String {
    match shell {
        ShellKind::Unix => escape(path.to_string_lossy()).to_string(),
        ShellKind::Windows => format!("'{}'", path.to_string_lossy().replace('\'', "''")),
    }
}

/// Format pattern for shell
pub fn format_pattern(shell: ShellKind, pattern: &str) -> String {
    match shell {
        ShellKind::Unix => escape(pattern.into()).to_string(),
        ShellKind::Windows => format!("'{}'", pattern.replace('\'', "''")),
    }
}

/// Join command parts
pub fn join_command(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
