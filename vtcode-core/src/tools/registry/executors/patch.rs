//! Patch application executor
//!
//! Handles the apply_patch tool for applying unified diff patches to files.

use crate::tools::apply_patch::Patch;
use anyhow::{Context, Result, anyhow};
use futures::future::BoxFuture;
use serde_json::{Value, json};
use std::env;
use tracing::{debug, warn};

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn apply_patch_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_apply_patch(args).await })
    }

    pub(super) async fn execute_apply_patch(&self, args: Value) -> Result<Value> {
        let patch_source = args
            .get("input")
            .or_else(|| args.get("patch"))
            .or_else(|| args.get("diff"));

        let input = patch_source.and_then(|v| v.as_str()).ok_or_else(|| {
            anyhow!(
                "Error: Missing 'input' string with patch content (aliases: 'patch', 'diff'). Example: apply_patch({{ \"input\": '*** Begin Patch...*** End Patch' }})"
            )
        })?;
        let patch = Patch::parse(input)?;
        let delete_ops = patch
            .operations()
            .iter()
            .filter(|op| matches!(op, crate::tools::editing::PatchOperation::DeleteFile { .. }))
            .count();
        let add_ops = patch
            .operations()
            .iter()
            .filter(|op| matches!(op, crate::tools::editing::PatchOperation::AddFile { .. }))
            .count();

        if delete_ops > 0 && add_ops > 0 {
            warn!(
                delete_ops,
                add_ops,
                "apply_patch will delete and recreate files; ensure backups or incremental edits"
            );

            // Emit telemetry event for destructive operation detection
            // This addresses the Codex issue review recommendation to track
            // cascading delete/recreate sequences
            //
            // Reference: docs/research/codex_issue_review.md - apply_patch Tool Reliability
            let affected_files: Vec<String> = patch
                .operations()
                .iter()
                .filter_map(|op| match op {
                    crate::tools::editing::PatchOperation::DeleteFile { path } => {
                        Some(path.clone())
                    }
                    crate::tools::editing::PatchOperation::AddFile { path, .. } => {
                        Some(path.clone())
                    }
                    _ => None,
                })
                .collect();

            // Check if we're in a git repository (simple heuristic for backup detection)
            let has_git_backup = self.workspace_root().join(".git").exists();

            let event = crate::tools::registry::ToolTelemetryEvent::delete_and_recreate_warning(
                "apply_patch",
                affected_files.clone(),
                has_git_backup,
            );

            // Log the telemetry event (structured logging for observability)
            debug!(
                event = ?event,
                "Emitting destructive operation telemetry"
            );

            // Check if confirmation is needed (destructive operations without backup)
            let skip_confirmations = env::var("VTCODE_SKIP_CONFIRMATIONS")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false);

            // Always prompt for confirmation if no git backup and not skipping confirmations
            let requires_confirmation = !skip_confirmations && !has_git_backup;

            if requires_confirmation {
                let file_list = affected_files
                    .iter()
                    .take(10) // Show first 10 files; truncate if more
                    .map(|f| format!("  - {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");

                let file_count_suffix = if affected_files.len() > 10 {
                    format!("\n  ... and {} more file(s)", affected_files.len() - 10)
                } else {
                    String::new()
                };

                let backup_warning = if has_git_backup {
                    "\nGit backup detected - can be recovered if needed."
                } else {
                    "\n⚠️  No git backup detected - deletion is permanent!"
                };

                let prompt_msg = format!(
                    "apply_patch will delete and recreate {} file(s):{}{}{}\n\nContinue?",
                    affected_files.len(),
                    file_list,
                    file_count_suffix,
                    backup_warning
                );

                // Check if running in TUI mode
                let in_tui_mode = env::var("VTCODE_TUI_MODE").is_ok();

                if in_tui_mode {
                    // TUI mode: Return error for runloop to handle with modal confirmation
                    return Err(anyhow!("CONFIRMATION_REQUIRED: {}", prompt_msg));
                } else {
                    // CLI mode: Use dialoguer for confirmation prompt
                    let confirmed = dialoguer::Confirm::new()
                        .with_prompt(prompt_msg)
                        .default(false)
                        .interact()
                        .context("Failed to get user confirmation")?;

                    if !confirmed {
                        return Ok(json!({
                            "success": false,
                            "error": "Operation cancelled by user",
                            "affected_files": affected_files,
                            "cancelled_at": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .ok()
                                .map(|d| d.as_secs())
                        }));
                    }
                }
            }
        }

        // Generate diff preview
        let mut diff_lines = Vec::new();
        for op in patch.operations() {
            match op {
                crate::tools::editing::PatchOperation::AddFile { path, content } => {
                    diff_lines.push(format!("--- /dev/null"));
                    diff_lines.push(format!("+++ {}", path));
                    for line in content.lines() {
                        diff_lines.push(format!("+{}", line));
                    }
                }
                crate::tools::editing::PatchOperation::DeleteFile { path } => {
                    diff_lines.push(format!("--- {}", path));
                    diff_lines.push(format!("+++ /dev/null"));
                }
                crate::tools::editing::PatchOperation::UpdateFile { path, chunks, .. } => {
                    diff_lines.push(format!("--- {}", path));
                    diff_lines.push(format!("+++ {}", path));
                    for chunk in chunks {
                        if let Some(ctx) = &chunk.change_context {
                            diff_lines.push(format!("@@ {} @@", ctx));
                        }
                        for line in &chunk.lines {
                            let (prefix, text) = match line {
                                crate::tools::editing::PatchLine::Addition(t) => ("+", t),
                                crate::tools::editing::PatchLine::Removal(t) => ("-", t),
                                crate::tools::editing::PatchLine::Context(t) => (" ", t),
                            };
                            diff_lines.push(format!("{}{}", prefix, text));
                        }
                    }
                }
            }
        }

        let results = match patch.apply(self.workspace_root()).await {
            Ok(results) => results,
            Err(err) => {
                warn!(
                    error = %err,
                    "apply_patch failed; consider falling back to incremental edits"
                );
                return Err(err);
            }
        };
        Ok(json!({
            "success": true,
            "applied": results,
            "diff_preview": diff_lines.join("\n"),
        }))
    }
}
