//! Search command validators
//!
//! This module validates search commands:
//! - grep: Text search
//! - rg (ripgrep): Fast text search
//! - sed: Stream editor
//!
//! # SECURITY CRITICAL
//!
//! This module blocks dangerous features:
//! - Ripgrep preprocessor flags (--pre, --pre-glob) - enable arbitrary command execution
//! - Sed execution flags (e, E, F, f) - enable arbitrary command execution
//!
//! These blocks are ESSENTIAL to prevent command injection attacks.

use std::path::Path;
use tokio::fs;

use anyhow::{Context, Result, anyhow};

use crate::policy::paths::{ensure_is_file, parse_positive_int, resolve_path, resolve_path_allow_dir};
use crate::policy::workspace::ensure_within_workspace;

/// Validate ripgrep (rg) command arguments.
///
/// # SECURITY CRITICAL
///
/// Blocks --pre and --pre-glob flags which enable arbitrary command execution.
///
/// Requirements:
/// - Pattern required (unless using file listing flags)
/// - Search paths must be within workspace
/// - No preprocessor flags allowed
/// - Safe flags validated: -A, -B, -C, -d, --max-depth, -m, --max-count, -g, --glob, etc.
pub async fn validate_rg(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut index = 0;
    let mut allow_no_pattern = false;

    while index < args.len() {
        let current = &args[index];
        if current == "--" {
            index += 1;
            break;
        }

        match current.as_str() {
            // SECURITY: Block preprocessor flags that enable arbitrary command execution
            "--pre" | "--pre-glob" => {
                return Err(anyhow!(
                    "ripgrep preprocessor flag '{}' is not permitted for security reasons. \
                     This flag enables arbitrary command execution.",
                    current
                ));
            }
            "-A" | "-B" | "-C" | "-d" | "--max-depth" | "-m" | "--max-count" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("option '{}' requires a value", current))?;
                parse_positive_int(value)
                    .with_context(|| format!("invalid value '{}' for '{}'", value, current))?;
                index += 2;
            }
            "-g" | "--glob" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("option '{}' requires a value", current))?;
                if value.is_empty() {
                    return Err(anyhow!("glob value for '{}' cannot be empty", current));
                }
                index += 2;
            }
            "-n" | "-i" | "-l" | "--files" | "--files-with-matches" | "--files-without-match" => {
                if matches!(
                    current.as_str(),
                    "--files" | "--files-with-matches" | "--files-without-match"
                ) {
                    allow_no_pattern = true;
                }
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported ripgrep flag '{}'", value));
            }
            _ => break,
        }
    }

    let remaining = &args[index..];
    if remaining.is_empty() && !allow_no_pattern {
        return Err(anyhow!(
            "ripgrep requires a pattern unless file listing flags are used"
        ));
    }

    let mut rem_index = 0;
    if !remaining.is_empty() {
        let pattern = &remaining[0];
        if pattern.is_empty() {
            return Err(anyhow!("ripgrep pattern cannot be empty"));
        }
        rem_index = 1;
    }

    if remaining.len() > rem_index {
        let search_root = &remaining[rem_index];
        let path = resolve_path_allow_dir(workspace_root, working_dir, search_root).await?;
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Err(anyhow!("search path '{}' does not exist", search_root));
        }
        if remaining.len() > rem_index + 1 {
            return Err(anyhow!("ripgrep accepts at most one search path"));
        }
    }

    Ok(())
}

/// Validate sed command arguments.
///
/// # SECURITY CRITICAL
///
/// Blocks execution flags (e, E, F, f) which enable arbitrary command execution.
///
/// Requirements:
/// - At least one sed command required
/// - At least one file required
/// - All files must exist within workspace
/// - Only substitution commands allowed (starting with 's')
/// - No execution flags allowed
/// - Safe flags: -n, -u, -e
pub async fn validate_sed(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut commands = Vec::new();
    let mut files = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let current = &args[index];
        match current.as_str() {
            "-n" | "-u" => {
                index += 1;
            }
            "-e" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("-e requires a sed command"))?;
                ensure_safe_sed_command(value)?;
                commands.push(value.clone());
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported sed flag '{}'", value));
            }
            value => {
                if commands.is_empty() {
                    ensure_safe_sed_command(value)?;
                    commands.push(value.to_string());
                    index += 1;
                } else {
                    let path = resolve_path(workspace_root, working_dir, value).await?;
                    ensure_is_file(&path).await?;
                    files.push(path);
                    index += 1;
                }
            }
        }
    }

    if commands.is_empty() {
        return Err(anyhow!("sed requires at least one command"));
    }

    if files.is_empty() {
        return Err(anyhow!("sed requires at least one readable file"));
    }

    Ok(())
}

/// Validate grep command arguments.
///
/// Requirements:
/// - All files must be within workspace
/// - Pattern comes before files
pub async fn validate_grep(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    let mut pattern_seen = false;
    for arg in args {
        if !arg.starts_with('-') && pattern_seen {
            // Files come after pattern
            let path = super::super::paths::normalize_path(&working_dir.join(arg));
            ensure_within_workspace(workspace_root, &path).await?;
        } else if !arg.starts_with('-') {
            pattern_seen = true;
        }
    }
    Ok(())
}

/// Ensure a sed command is safe to execute.
///
/// # SECURITY CRITICAL
///
/// This function prevents arbitrary command execution via sed's execution flags.
///
/// Requirements:
/// - Command must be a substitution (starts with 's')
/// - Must have valid delimiter
/// - No execution flags (e, E, F, f) allowed
/// - No shell metacharacters (;, |, &, `) allowed
fn ensure_safe_sed_command(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("sed command cannot be empty"));
    }
    if value.contains([';', '|', '&', '`']) {
        return Err(anyhow!(
            "sed command contains unsupported control characters"
        ));
    }

    let mut chars = value.chars();
    if chars.next() != Some('s') {
        return Err(anyhow!("only sed substitution commands are supported"));
    }
    let delimiter = chars
        .next()
        .ok_or_else(|| anyhow!("sed substitution is missing a delimiter"))?;
    if delimiter.is_ascii_alphanumeric() || delimiter.is_ascii_whitespace() {
        return Err(anyhow!("invalid sed delimiter"));
    }

    let mut pattern = String::new();
    let mut replacement = String::new();
    let mut flags = String::new();

    parse_sed_section(&mut chars, delimiter, &mut pattern)?;
    parse_sed_section(&mut chars, delimiter, &mut replacement)?;
    collect_sed_flags(chars, &mut flags)?;

    // SECURITY: Block execution flags
    if flags.chars().any(|ch| matches!(ch, 'e' | 'E' | 'F' | 'f')) {
        return Err(anyhow!(
            "sed execution flags are not permitted in substitution"
        ));
    }

    Ok(())
}

/// Parse a section of a sed substitution command.
///
/// Sections are delimited by the chosen delimiter character.
/// Handles escaped delimiters within sections.
fn parse_sed_section(
    chars: &mut std::str::Chars<'_>,
    delimiter: char,
    target: &mut String,
) -> Result<()> {
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if escaped {
            target.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
            }
            value if value == delimiter => {
                return Ok(());
            }
            other => target.push(other),
        }
    }
    Err(anyhow!("sed command is missing a closing delimiter"))
}

/// Collect sed flags from the end of a substitution command.
///
/// Flags must be alphabetic characters only.
fn collect_sed_flags(chars: std::str::Chars<'_>, target: &mut String) -> Result<()> {
    for ch in chars {
        if ch.is_ascii_alphabetic() {
            target.push(ch);
        } else {
            return Err(anyhow!("sed flags contain unsupported characters"));
        }
    }
    Ok(())
}
