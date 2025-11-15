//! NPM command validation
//!
//! Validates Node Package Manager commands.
//!
//! Allowed operations:
//! - Most NPM operations within workspace
//!
//! Blocked operations:
//! - publish (registry modification)
//! - unpublish (registry modification)

use std::path::Path;

use anyhow::{Result, anyhow};

use crate::policy::workspace::ensure_within_workspace;

/// Validate npm command arguments.
///
/// Allows typical dev operations within workspace.
/// Blocks registry publishing operations.
pub async fn validate_npm(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    ensure_within_workspace(workspace_root, working_dir).await?;
    if args.is_empty() {
        return Ok(());
    }

    let subcommand = args[0].as_str();
    match subcommand {
        // Dangerous operations
        "publish" | "unpublish" => Err(anyhow!(
            "npm {} is not permitted by the execution policy",
            subcommand
        )),
        // Allow safe and other commands by default, as npm is generally safe in workspace
        _ => Ok(()),
    }
}
