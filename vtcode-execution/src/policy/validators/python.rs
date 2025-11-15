//! Python and Node.js command validation
//!
//! Validates Python and Node.js interpreter commands.
//!
//! Allowed operations:
//! - Running scripts within workspace
//! - Module execution (-m)
//! - Interactive REPL
//!
//! Requirements:
//! - Script paths must be within workspace
//! - Working directory must be within workspace

use std::path::Path;

use anyhow::{Result};

use crate::policy::workspace::ensure_within_workspace;

/// Validate python/python3 command arguments.
///
/// Allows running scripts and modules within workspace.
/// Interactive mode allowed.
///
/// Validates:
/// - Script files are within workspace
/// - Module execution (-m)
/// - Warning flags (-W)
pub async fn validate_python(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    ensure_within_workspace(workspace_root, working_dir).await?;

    if args.is_empty() {
        return Ok(()); // python interactive is allowed
    }

    let first_arg = &args[0];
    if first_arg == "-c" || first_arg == "-m" || first_arg == "-W" {
        // Allow -m (module), -W (warnings), but validate any file paths
        if first_arg != "-m" && args.len() > 1 {
            let path = super::super::paths::normalize_path(&working_dir.join(&args[1]));
            ensure_within_workspace(workspace_root, &path).await?;
        }
    } else if !first_arg.starts_with('-') {
        // It's a script file - validate it exists in workspace
        let path = super::super::paths::normalize_path(&working_dir.join(first_arg));
        ensure_within_workspace(workspace_root, &path).await?;
    }
    Ok(())
}

/// Validate node command arguments.
///
/// Allows running scripts within workspace.
/// Interactive REPL allowed.
///
/// Validates:
/// - Script files are within workspace
pub async fn validate_node(args: &[String], workspace_root: &Path, working_dir: &Path) -> Result<()> {
    ensure_within_workspace(workspace_root, working_dir).await?;

    if args.is_empty() {
        return Ok(()); // node interactive/REPL
    }

    let first_arg = &args[0];
    if !first_arg.starts_with('-') {
        // It's a script file - validate it exists in workspace
        let path = super::super::paths::normalize_path(&working_dir.join(first_arg));
        ensure_within_workspace(workspace_root, &path).await?;
    }
    Ok(())
}
