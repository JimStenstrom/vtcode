//! VTCode-core specific extensions for prompts
//!
//! This module provides integration between vtcode-prompts and vtcode-core
//! specific modules like gemini, instructions, and project_doc.

use crate::config::constants::{
    instructions as instruction_constants, project_doc as project_doc_constants,
};
use vtcode_llm_gemini::gemini::Content;
use crate::instructions::{InstructionBundle, InstructionScope, read_instruction_bundle};
use crate::project_doc::read_project_doc;
use dirs::home_dir;
use std::env;
use std::path::Path;
use tracing::warn;
use vtcode_prompts::{SystemPromptConfig, read_system_prompt_from_md, default_system_prompt};

/// Read AGENTS.md file if present and extract agent guidelines
pub async fn read_agent_guidelines(project_root: &Path) -> Option<String> {
    let max_bytes =
        project_doc_constants::DEFAULT_MAX_BYTES.min(instruction_constants::DEFAULT_MAX_BYTES);
    match read_project_doc(project_root, max_bytes).await {
        Ok(Some(bundle)) => Some(bundle.contents),
        Ok(None) => None,
        Err(err) => {
            warn!("failed to load project documentation: {err:#}");
            None
        }
    }
}

/// Compose system instruction text with vtcode-core specific configuration
pub async fn compose_system_instruction_text_with_config(
    project_root: &Path,
    vtcode_config: Option<&crate::config::VTCodeConfig>,
) -> String {
    let mut instruction = match read_system_prompt_from_md().await {
        Ok(content) => content,
        Err(_) => default_system_prompt().to_string(),
    };

    if let Some(cfg) = vtcode_config {
        instruction.push_str("\n\n## CONFIGURATION AWARENESS\n");
        instruction
            .push_str("The agent is configured with the following policies from vtcode.toml:\n\n");

        if cfg.security.human_in_the_loop {
            instruction.push_str("- **Human-in-the-loop**: Required for critical actions\n");
        }

        if !cfg.commands.allow_list.is_empty() {
            instruction.push_str(&format!(
                "- **Allowed commands**: {} commands in allow list\n",
                cfg.commands.allow_list.len()
            ));
        }
        if !cfg.commands.deny_list.is_empty() {
            instruction.push_str(&format!(
                "- **Denied commands**: {} commands in deny list\n",
                cfg.commands.deny_list.len()
            ));
        }

        if cfg.pty.enabled {
            instruction.push_str("- **PTY functionality**: Enabled\n");
            let (rows, cols) = (cfg.pty.default_rows, cfg.pty.default_cols);
            instruction.push_str(&format!(
                "- **Default terminal size**: {} rows × {} columns\n",
                rows, cols
            ));
            instruction.push_str(&format!(
                "- **PTY command timeout**: {} seconds\n",
                cfg.pty.command_timeout_seconds
            ));
        } else {
            instruction.push_str("- **PTY functionality**: Disabled\n");
        }

        instruction.push_str("\n**IMPORTANT**: Respect these configuration policies. Commands not in the allow list will require user confirmation. Always inform users when actions require confirmation due to security policies.\n");
    }

    let home_path = home_dir();

    if let Some(bundle) = read_instruction_hierarchy(project_root, vtcode_config).await {
        let home_ref = home_path.as_deref();
        instruction.push_str("\n\n## AGENTS.MD INSTRUCTION HIERARCHY\n");
        instruction.push_str(
            "Instructions are listed from lowest to highest precedence. When conflicts exist, defer to the later entries.\n\n",
        );

        for (index, segment) in bundle.segments.iter().enumerate() {
            let scope = match segment.source.scope {
                InstructionScope::Global => "global",
                InstructionScope::Workspace => "workspace",
                InstructionScope::Custom => "custom",
            };
            let display_path =
                format_instruction_path(&segment.source.path, project_root, home_ref);

            instruction.push_str(&format!(
                "### {}. {} ({})\n\n",
                index + 1,
                display_path,
                scope
            ));
            instruction.push_str(segment.contents.trim());
            instruction.push_str("\n");
        }

        if bundle.truncated {
            instruction.push_str(
                "\n_Note: instruction content was truncated due to size limits. Review the source files for full details._",
            );
        }
    }

    instruction
}

/// Generate system instruction with configuration and AGENTS.md guidelines incorporated
/// Returns a Content object for use with Gemini and other providers
pub async fn generate_system_instruction_content(
    _config: &SystemPromptConfig,
    project_root: &Path,
    vtcode_config: Option<&crate::config::VTCodeConfig>,
) -> Content {
    let instruction = compose_system_instruction_text_with_config(project_root, vtcode_config).await;
    Content::system_text(instruction)
}

/// Generate system instruction with configuration and guidelines (backward compatibility)
pub async fn generate_system_instruction_with_config_and_guidelines(
    config: &SystemPromptConfig,
    project_root: &Path,
    vtcode_config: Option<&crate::config::VTCodeConfig>,
) -> Content {
    generate_system_instruction_content(config, project_root, vtcode_config).await
}

/// Read instruction hierarchy from AGENTS.md and related files
pub async fn read_instruction_hierarchy(
    project_root: &Path,
    vtcode_config: Option<&crate::config::VTCodeConfig>,
) -> Option<InstructionBundle> {
    let (max_bytes, extra_sources) = match vtcode_config {
        Some(cfg) => (
            cfg.agent.instruction_max_bytes,
            cfg.agent.instruction_files.clone(),
        ),
        None => (instruction_constants::DEFAULT_MAX_BYTES, Vec::new()),
    };

    if max_bytes == 0 {
        return None;
    }

    let current_dir = env::current_dir().unwrap_or_else(|_| project_root.to_path_buf());
    let home = home_dir();
    match read_instruction_bundle(
        &current_dir,
        project_root,
        home.as_deref(),
        &extra_sources,
        max_bytes,
    )
    .await
    {
        Ok(Some(bundle)) => Some(bundle),
        Ok(None) => None,
        Err(err) => {
            warn!("failed to load instruction hierarchy: {err:#}");
            None
        }
    }
}

fn format_instruction_path(path: &Path, project_root: &Path, home_dir: Option<&Path>) -> String {
    if let Ok(relative) = path.strip_prefix(project_root) {
        let display = relative.display().to_string();
        if !display.is_empty() {
            return display;
        }

        if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            return name.to_string();
        }
    }

    if let Some(home) = home_dir {
        if let Ok(relative) = path.strip_prefix(home) {
            let display = relative.display().to_string();
            if display.is_empty() {
                return "~".to_string();
            }

            return format!("~/{display}");
        }
    }

    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_compose_system_instruction() {
        let temp = TempDir::new().unwrap();
        let instruction = compose_system_instruction_text_with_config(temp.path(), None).await;
        assert!(!instruction.is_empty());
    }

    #[tokio::test]
    async fn test_generate_system_instruction_content() {
        let temp = TempDir::new().unwrap();
        let config = SystemPromptConfig::default();
        let content = generate_system_instruction_content(&config, temp.path(), None).await;
        assert_eq!(content.role, "user"); // System content is converted to user for compatibility
    }
}
