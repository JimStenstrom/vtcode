//! Cargo command validation
//!
//! Validates Rust package manager commands.
//!
//! Allowed operations:
//! - build, check, test, doc, clippy, fmt, run, bench, expand
//! - tree, metadata, search, cache
//!
//! Blocked operations:
//! - clean (destructive)
//! - install, uninstall (system modification)
//! - publish, yank (registry modification)

use std::path::Path;

use anyhow::{Result, anyhow};

use crate::policy::workspace::ensure_within_workspace;

/// Validate cargo command arguments.
///
/// Allows typical dev workflow operations within workspace.
/// Blocks destructive and system-modifying operations.
pub async fn validate_cargo(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("cargo requires a subcommand"));
    }

    let subcommand = args[0].as_str();
    match subcommand {
        // Safe read-only, build, and development operations
        "build" | "check" | "test" | "doc" | "clippy" | "fmt" | "run" | "bench" | "expand"
        | "tree" | "metadata" | "search" | "cache" => {
            // These are generally safe - check working directory is in workspace
            ensure_within_workspace(workspace_root, working_dir).await?;
            Ok(())
        }
        // Dangerous operations that modify system or registry
        "clean" | "install" | "uninstall" | "publish" | "yank" => Err(anyhow!(
            "cargo {} is not permitted by the execution policy",
            subcommand
        )),
        other => Err(anyhow!(
            "cargo subcommand '{}' is not permitted by the execution policy",
            other
        )),
    }
}
