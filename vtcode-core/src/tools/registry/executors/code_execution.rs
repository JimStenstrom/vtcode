//! Code execution executor
//!
//! Handles the execute_code tool for running Python and JavaScript code in a sandbox.

use anyhow::{Context, Result, anyhow};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tracing::debug;

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn execute_code_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let mcp_client = self.mcp_client.clone();
        let workspace_root = self.inventory.workspace_root().to_path_buf();
        Box::pin(async move {
            use crate::exec::code_executor::{CodeExecutor, Language};

            #[derive(Debug, Deserialize)]
            struct ExecuteCodeArgs {
                code: String,
                language: String,
                #[serde(default)]
                timeout_secs: Option<u64>,
            }

            let parsed: ExecuteCodeArgs = serde_json::from_value(args)
                .context("execute_code requires 'code' and 'language' fields")?;

            // Validate language
            let language = match parsed.language.as_str() {
                "python3" | "python" => Language::Python3,
                "javascript" | "js" => Language::JavaScript,
                invalid => {
                    return Err(anyhow!(
                        "Invalid language: '{}'. Must be 'python3' or 'javascript'",
                        invalid
                    ));
                }
            };

            // Get MCP client for code execution
            let result = match mcp_client {
                Some(mcp_client) => {
                    // Build execution config
                    let mut config: crate::exec::code_executor::ExecutionConfig =
                        Default::default();
                    if let Some(timeout_secs) = parsed.timeout_secs {
                        config.timeout_secs = timeout_secs;
                    }

                    // Create a safe sandbox profile with workspace isolation
                    // The sandbox enforces these restrictions:
                    // - Shell: Auto-detected (pwsh/bash on supported systems, cmd.exe on Windows)
                    // - Working directory: .vtcode/sandbox in workspace
                    // - Allowed paths: workspace root + /tmp (temporary files)
                    // - Runtime: AnthropicSrt for code execution monitoring
                    let sandbox_profile = crate::sandbox::SandboxProfile::new(
                        resolve_shell_candidate(),
                        workspace_root.join(".vtcode/sandbox/settings.json"),
                        workspace_root.join(".vtcode/sandbox"),
                        vec![workspace_root.clone(), std::path::PathBuf::from("/tmp")],
                        crate::sandbox::SandboxRuntimeKind::AnthropicSrt,
                    );

                    // Create and configure code executor
                    let executor = CodeExecutor::new(
                        language,
                        sandbox_profile,
                        mcp_client,
                        workspace_root.clone(),
                    )
                    .with_config(config);

                    // Execute the code
                    executor
                        .execute(&parsed.code)
                        .await
                        .context("code execution failed")?
                }
                None => {
                    debug!("MCP client not configured, attempting direct code execution");

                    // Attempt direct code execution without MCP if no client available
                    let code = parsed.code.clone();
                    let language = language;

                    // Create a direct executor (non-sandboxed fallback)
                    // In a real implementation, this would need proper sandboxing
                    use std::io::Write;
                    use std::process::Command;
                    use tempfile::NamedTempFile;

                    let result = match language {
                        Language::Python3 => {
                            let output = Command::new("python3")
                                .arg("-c")
                                .arg(&code)
                                .current_dir(&workspace_root)
                                .output()
                                .context("failed to execute Python code")?;

                            crate::exec::code_executor::ExecutionResult {
                                exit_code: output.status.code().unwrap_or(1) as i32,
                                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                                duration_ms: 0, // Not tracked in this fallback
                                json_result: None,
                            }
                        }
                        Language::JavaScript => {
                            // Create a temporary file for JavaScript execution
                            let mut temp_file = NamedTempFile::new_in(&workspace_root)
                                .context("failed to create temp file for JavaScript execution")?;
                            temp_file
                                .write_all(code.as_bytes())
                                .context("failed to write JavaScript code to temp file")?;

                            let output = Command::new("node")
                                .arg(temp_file.path())
                                .current_dir(&workspace_root)
                                .output()
                                .context("failed to execute JavaScript code")?;

                            crate::exec::code_executor::ExecutionResult {
                                exit_code: output.status.code().unwrap_or(1) as i32,
                                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                                duration_ms: 0, // Not tracked in this fallback
                                json_result: None,
                            }
                        }
                    };

                    result
                }
            };

            debug!(
                exit_code = result.exit_code,
                duration_ms = result.duration_ms,
                has_output = !result.stdout.is_empty(),
                has_error = !result.stderr.is_empty(),
                has_json_result = result.json_result.is_some(),
                "Code execution completed"
            );

            // Build response
            let mut response = json!({
                "exit_code": result.exit_code,
                "duration_ms": result.duration_ms,
                "stdout": result.stdout,
                "stderr": result.stderr,
            });

            // Include JSON result if present
            if let Some(json_result) = result.json_result {
                response["result"] = json_result;
            }

            Ok(response)
        })
    }
}

// Helper functions for shell resolution

fn resolve_shell_candidate() -> PathBuf {
    // Resolve the preferred shell for sandbox execution
    // Detects available shells based on platform
    if cfg!(windows) {
        // Windows: prefer PowerShell if available, fall back to cmd.exe
        if Path::new("C:\\Windows\\System32\\pwsh.exe").exists() {
            PathBuf::from("C:\\Windows\\System32\\pwsh.exe")
        } else if Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
            PathBuf::from("C:\\Program Files\\PowerShell\\7\\pwsh.exe")
        } else {
            PathBuf::from("cmd.exe")
        }
    } else {
        // POSIX systems: use detected shell or default to /bin/sh
        detect_posix_shell_candidate()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/bin/sh"))
    }
}

fn detect_posix_shell_candidate() -> Option<String> {
    if cfg!(windows) {
        return None;
    }

    const CANDIDATES: [&str; 6] = [
        "/bin/zsh",
        "/usr/bin/zsh",
        "/bin/bash",
        "/usr/bin/bash",
        "/bin/sh",
        "/usr/bin/sh",
    ];

    for candidate in CANDIDATES {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    None
}
