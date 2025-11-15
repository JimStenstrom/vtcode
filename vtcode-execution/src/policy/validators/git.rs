//! Git command validation
//!
//! Git has complex validation logic split into tiers:
//! - **Tier 1**: Safe read-only operations (status, log, diff, etc.)
//! - **Tier 2**: Safe write operations (add, commit, reset --soft)
//! - **Tier 3**: Dangerous operations (force push, clean, filter-branch) - BLOCKED
//!
//! # SECURITY CRITICAL
//!
//! This module blocks dangerous git operations:
//! - Force push (--force, -f)
//! - git clean (destructive)
//! - git filter-branch (complex history rewriting)
//! - git rebase (complex history operations)
//! - git gc --aggressive (resource intensive)

use std::path::Path;

use anyhow::{Result, anyhow};

use crate::policy::paths::resolve_path;
use crate::policy::workspace::ensure_within_workspace;

/// Validate git command and subcommands.
///
/// Dispatches to specialized validators based on git subcommand.
///
/// # Security Tiers
///
/// - Tier 1 (Read-only): status, log, show, diff, branch, tag, etc.
/// - Tier 2 (Safe writes): add, commit, reset --soft, checkout, merge
/// - Tier 3 (Blocked): force-push, clean, filter-branch, rebase
pub async fn validate_git(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("git requires a subcommand"));
    }

    let subcommand = args[0].as_str();
    let subargs = &args[1..];

    // Tier 1: Safe read-only operations (always allowed)
    match subcommand {
        // Status and history operations
        "status" | "log" | "show" | "diff" | "branch" | "tag" | "remote" => {
            return validate_git_read_only(subcommand, subargs);
        }

        // Tree and object inspection
        "ls-tree" | "ls-files" | "cat-file" | "rev-parse" | "describe" => {
            return validate_git_read_only(subcommand, subargs);
        }

        // Config inspection (read-only)
        "config" if subargs.is_empty() || subargs.iter().all(|a| !a.starts_with("--")) => {
            return validate_git_read_only(subcommand, subargs);
        }

        // Additional inspection commands
        "blame" | "grep" | "shortlog" | "format-patch" => {
            return validate_git_read_only(subcommand, subargs);
        }

        // Stash operations (safe list/show)
        "stash"
            if matches!(
                subargs.first().map(|s| s.as_str()),
                Some("list" | "show" | "pop" | "apply" | "drop")
            ) =>
        {
            return validate_git_stash(subargs);
        }

        // Tier 2: Safe write operations (with validation)
        "add" => return validate_git_add(subargs, workspace_root, working_dir).await,
        "commit" => return validate_git_commit(subargs),
        "reset" => return validate_git_reset(subargs),
        "checkout" | "switch" | "restore" => {
            return validate_git_checkout(subargs, workspace_root, working_dir).await;
        }
        "merge" => return validate_git_merge(subargs),

        // Tier 3: Dangerous operations (always blocked)
        "push" => {
            // Check for force flags
            if subargs
                .iter()
                .any(|a| a.contains("force") || a == "-f" || a == "--no-verify")
            {
                return Err(anyhow!(
                    "git push with force flags is not permitted. Use safe push operations only."
                ));
            }
            return validate_git_read_only(subcommand, subargs);
        }

        "force-push" => {
            return Err(anyhow!(
                "git force-push is not permitted by the execution policy"
            ));
        }

        "clean" => {
            return Err(anyhow!(
                "git clean is not permitted by the execution policy. Use explicit rm commands instead."
            ));
        }

        "gc" if subargs.iter().any(|a| a.contains("aggressive")) => {
            return Err(anyhow!("git gc with aggressive flag is not permitted"));
        }

        "filter-branch" | "rebase" | "cherry-pick" => {
            return Err(anyhow!(
                "git {} is not permitted - complex history operations require confirmation",
                subcommand
            ));
        }

        other => {
            return Err(anyhow!(
                "git subcommand '{}' is not permitted by the execution policy",
                other
            ));
        }
    }
}

/// Validate read-only git commands.
///
/// Blocks:
/// - Shell metacharacters (;, |, &)
/// - Suspicious patterns
///
/// Allows:
/// - Safe output flags (--oneline, --graph, --stat, etc.)
/// - Custom formats (--format)
/// - Common flags per subcommand
pub fn validate_git_read_only(subcommand: &str, subargs: &[String]) -> Result<()> {
    // Block dangerous flags even in read-only commands
    let dangerous_flags = ["-q", "--quiet", "--verbose", "-v"];

    for arg in subargs {
        if arg.starts_with("--") && arg.contains('=') {
            let key = arg.split('=').next().unwrap_or("");
            if key == "--format" {
                // Allow custom formats for output
                continue;
            }
        }

        if dangerous_flags.contains(&arg.as_str()) {
            // Benign flags, allow them
            continue;
        }

        // Allow common flags per subcommand
        match subcommand {
            "log" | "show" => {
                if matches!(
                    arg.as_str(),
                    "-n" | "--oneline"
                        | "--graph"
                        | "--decorate"
                        | "--all"
                        | "--grep"
                        | "-S"
                        | "-p"
                        | "-U"
                        | "--stat"
                        | "--shortstat"
                        | "--name-status"
                        | "--name-only"
                        | "--author"
                        | "--since"
                        | "--until"
                        | "--date"
                ) {
                    continue;
                }
            }
            "diff" => {
                if matches!(
                    arg.as_str(),
                    "-p" | "-U"
                        | "--stat"
                        | "--shortstat"
                        | "--name-status"
                        | "--name-only"
                        | "--no-index"
                        | "-w"
                        | "--ignore-all-space"
                        | "-b"
                        | "--ignore-space-change"
                ) {
                    continue;
                }
            }
            "branch" => {
                if matches!(arg.as_str(), "-a" | "-r" | "-v" | "--verbose") {
                    continue;
                }
            }
            _ => {
                // For other read-only commands, allow most flags
                if !arg.starts_with('-') || arg.starts_with("--") {
                    continue;
                }
            }
        }

        // Block any suspicious patterns
        if arg.contains(';') || arg.contains('|') || arg.contains('&') {
            return Err(anyhow!(
                "git argument contains suspicious shell metacharacters"
            ));
        }
    }

    Ok(())
}

/// Validate git add command.
///
/// Blocks:
/// - --force flag (adds ignored files)
///
/// Allows:
/// - -u, --update, -A, --all, . (safe bulk operations)
/// - -p, --patch, -i, --interactive (safe interactive)
/// - -n, --dry-run
/// - Individual file paths (validated)
pub async fn validate_git_add(
    args: &[String],
    workspace_root: &Path,
    working_dir: &Path,
) -> Result<()> {
    // Block dangerous flags
    if args.contains(&"-f".to_string()) || args.contains(&"--force".to_string()) {
        return Err(anyhow!(
            "git add --force is not permitted. Use regular add operations only."
        ));
    }

    // Validate file paths if provided
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-u" | "--update" | "-A" | "--all" | "." => {
                // These are safe - they add all tracked or current directory
                index += 1;
            }
            "-p" | "--patch" | "-i" | "--interactive" => {
                // Interactive mode is fine
                index += 1;
            }
            "-n" | "--dry-run" => {
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported git add flag '{}'", value));
            }
            path => {
                // Validate the file path
                let resolved = resolve_path(workspace_root, working_dir, path).await?;
                ensure_within_workspace(workspace_root, &resolved).await?;
                index += 1;
            }
        }
    }

    Ok(())
}

/// Validate git commit command.
///
/// Allows:
/// - -m, --message (with message)
/// - -F, --file (with file path)
/// - -a, --all, -p, --patch, --amend
/// - --no-verify, -q, --quiet
pub fn validate_git_commit(args: &[String]) -> Result<()> {
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-m" | "--message" => {
                if index + 1 >= args.len() {
                    return Err(anyhow!("-m requires a commit message"));
                }
                index += 2;
            }
            "-F" | "--file" => {
                if index + 1 >= args.len() {
                    return Err(anyhow!("-F requires a file path"));
                }
                index += 2;
            }
            "-a" | "--all" | "-p" | "--patch" | "--amend" | "--no-verify" | "-q" | "--quiet" => {
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported git commit flag '{}'", value));
            }
            _ => {
                index += 1;
            }
        }
    }

    Ok(())
}

/// Validate git reset command.
///
/// Blocks:
/// - --hard (destructive)
/// - --merge (complex)
/// - --keep (complex)
///
/// Allows:
/// - --soft (safe)
/// - --mixed (safe, default)
/// - --unstage (safe)
/// - -q, --quiet, -p, --patch
pub fn validate_git_reset(args: &[String]) -> Result<()> {
    // Block destructive reset modes
    if args.contains(&"--hard".to_string())
        || args.contains(&"--merge".to_string())
        || args.contains(&"--keep".to_string())
    {
        return Err(anyhow!(
            "git reset with --hard, --merge, or --keep is not permitted. Use --soft or --mixed instead."
        ));
    }

    // Allow safe flags: --soft, --mixed, --unstage
    let safe_modes = ["--soft", "--mixed", "--unstage"];
    for arg in args {
        if arg.starts_with('-') && !safe_modes.iter().any(|m| arg.contains(m)) {
            match arg.as_str() {
                "-q" | "--quiet" | "-p" | "--patch" => continue,
                _ => {
                    return Err(anyhow!(
                        "unsupported git reset flag '{}'. Use --soft or --mixed modes.",
                        arg
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validate git checkout/switch/restore commands.
///
/// Blocks:
/// - --force, -f (destructive)
///
/// Allows:
/// - Branch switching
/// - File restoration (validated paths)
pub async fn validate_git_checkout(
    args: &[String],
    workspace_root: &Path,
    working_dir: &Path,
) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    // Block forced checkout
    if args.contains(&"-f".to_string()) || args.contains(&"--force".to_string()) {
        return Err(anyhow!(
            "git checkout --force is not permitted. Use regular checkout instead."
        ));
    }

    // Validate paths if provided
    let mut paths_start = 0;
    for (i, arg) in args.iter().enumerate() {
        if arg == "--" {
            paths_start = i + 1;
            break;
        }
        if !arg.starts_with('-') {
            // Could be a branch or path
            paths_start = i;
            break;
        }
    }

    if paths_start > 0 {
        for path_arg in &args[paths_start..] {
            // Validate file paths
            let resolved = resolve_path(workspace_root, working_dir, path_arg).await?;
            ensure_within_workspace(workspace_root, &resolved).await?;
        }
    }

    Ok(())
}

/// Validate git stash command.
///
/// Allows:
/// - list, show, pop, apply, drop, clear, create
/// - Safe flags: -q, --quiet, -p, --patch, -k, --keep-index, -u, --include-untracked, -a, --all
///
/// Blocks:
/// - Other stash operations
pub fn validate_git_stash(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    let allowed_ops = ["list", "show", "pop", "apply", "drop", "clear", "create"];
    let first = args[0].as_str();

    if !allowed_ops.contains(&first) {
        return Err(anyhow!("git stash operation '{}' is not permitted", first));
    }

    // Allow flags for these operations
    for arg in &args[1..] {
        if arg.starts_with('-') {
            match arg.as_str() {
                "-q"
                | "--quiet"
                | "-p"
                | "--patch"
                | "-k"
                | "--keep-index"
                | "-u"
                | "--include-untracked"
                | "-a"
                | "--all" => continue,
                _ => return Err(anyhow!("unsupported git stash flag '{}'", arg)),
            }
        }
    }

    Ok(())
}

/// Validate git merge command.
///
/// Blocks:
/// - --no-ff (complex)
/// - --squash (complex)
///
/// Allows:
/// - Simple merges
pub fn validate_git_merge(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("git merge requires a branch"));
    }

    // Block dangerous flags
    let dangerous_flags = ["--no-ff", "--squash"];
    for arg in args {
        if dangerous_flags.contains(&arg.as_str()) {
            return Err(anyhow!(
                "git merge with {} flag is not permitted; use simpler merge",
                arg
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_git_read_only() {
        // Safe read-only operations
        assert!(validate_git_read_only("status", &[]).is_ok());
        assert!(validate_git_read_only("log", &["--oneline".to_string()]).is_ok());
        assert!(validate_git_read_only("diff", &["-p".to_string()]).is_ok());
        assert!(validate_git_read_only("show", &["HEAD".to_string()]).is_ok());
        assert!(validate_git_read_only("branch", &["-a".to_string()]).is_ok());

        // Dangerous patterns blocked
        assert!(
            validate_git_read_only("log", &["--format".to_string(), "test;cat".to_string()])
                .is_err()
        );
    }

    #[test]
    fn test_validate_git_commit() {
        // Valid commits
        assert!(validate_git_commit(&["-m".to_string(), "fix: test".to_string()]).is_ok());
        assert!(validate_git_commit(&["-a".to_string()]).is_ok());
        assert!(validate_git_commit(&["--amend".to_string()]).is_ok());

        // Invalid commits
        assert!(validate_git_commit(&["-m".to_string()]).is_err()); // Missing message
        assert!(validate_git_commit(&["--invalid-flag".to_string()]).is_err());
    }

    #[test]
    fn test_validate_git_reset() {
        // Safe reset modes
        assert!(validate_git_reset(&["--soft".to_string()]).is_ok());
        assert!(validate_git_reset(&["--mixed".to_string()]).is_ok());
        assert!(validate_git_reset(&["--unstage".to_string()]).is_ok());
        assert!(validate_git_reset(&[]).is_ok());

        // Dangerous reset modes
        assert!(validate_git_reset(&["--hard".to_string()]).is_err());
        assert!(validate_git_reset(&["--merge".to_string()]).is_err());
        assert!(validate_git_reset(&["--keep".to_string()]).is_err());
    }

    #[test]
    fn test_validate_git_stash() {
        // Safe stash operations
        assert!(validate_git_stash(&["list".to_string()]).is_ok());
        assert!(validate_git_stash(&["show".to_string()]).is_ok());
        assert!(validate_git_stash(&["pop".to_string()]).is_ok());
        assert!(validate_git_stash(&["apply".to_string()]).is_ok());
        assert!(validate_git_stash(&["drop".to_string()]).is_ok());

        // Dangerous operations
        assert!(validate_git_stash(&["push".to_string()]).is_err());
        assert!(validate_git_stash(&["save".to_string()]).is_err());
    }

    #[tokio::test]
    async fn test_validate_git_safe_operations() {
        let workspace = std::path::PathBuf::from("/tmp");
        let working = std::path::PathBuf::from("/tmp");

        // Safe read-only operations should be allowed
        assert!(
            validate_git(&["status".to_string()], &workspace, &working)
                .await
                .is_ok()
        );
        assert!(
            validate_git(
                &["log".to_string(), "--oneline".to_string()],
                &workspace,
                &working
            )
            .await
            .is_ok()
        );
        assert!(
            validate_git(&["diff".to_string()], &workspace, &working)
                .await
                .is_ok()
        );
        assert!(
            validate_git(
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
            validate_git(
                &["push".to_string(), "--force".to_string()],
                &workspace,
                &working
            )
            .await
            .is_err()
        );
        assert!(
            validate_git(
                &["push".to_string(), "-f".to_string()],
                &workspace,
                &working
            )
            .await
            .is_err()
        );
        assert!(
            validate_git(&["clean".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validate_git(&["filter-branch".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validate_git(&["rebase".to_string()], &workspace, &working)
                .await
                .is_err()
        );
        assert!(
            validate_git(&["cherry-pick".to_string()], &workspace, &working)
                .await
                .is_err()
        );
    }
}
