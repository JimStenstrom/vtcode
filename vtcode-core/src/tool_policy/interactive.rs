//! Interactive user prompting for policy decisions

use super::types::AUTO_ALLOW_TOOLS;
use anyhow::Result;
use dialoguer::{
    console::{style, Color as ConsoleColor, Style as ConsoleStyle},
    theme::ColorfulTheme,
    Confirm,
};
use std::io::IsTerminal;

use crate::ui::theme;
use crate::utils::ansi::{AnsiRenderer, MessageStyle};

/// Handles interactive user prompting for policy decisions
pub struct InteractivePolicyPrompt;

impl InteractivePolicyPrompt {
    /// Prompt user for tool execution permission
    ///
    /// WARNING: This function uses CLI dialoguer prompts and should NEVER be called
    /// when running in TUI mode, as it will corrupt the terminal display.
    ///
    /// In TUI mode, tool permissions should be handled by `ensure_tool_permission()`
    /// in `src/agent/runloop/unified/tool_routing.rs` which uses TUI modals.
    /// The preapproval system should prevent this function from being called.
    pub async fn prompt_user_for_tool(tool_name: &str) -> Result<bool> {
        // SAFETY CHECK: Detect if we're in TUI mode by checking for RATATUI environment
        // If this function is called during TUI mode, it's a bug in the preapproval system
        if std::env::var("VTCODE_TUI_MODE").is_ok() {
            tracing::error!(
                "BUG: prompt_user_for_tool() called in TUI mode for tool '{}'. \
                This should never happen - the preapproval system should prevent this. \
                Denying tool execution to prevent TUI corruption.",
                tool_name
            );
            anyhow::bail!(
                "Internal error: CLI prompt attempted in TUI mode for tool '{}'. \
                This is a bug - please report it. Tool execution denied to prevent display corruption.",
                tool_name
            );
        }

        let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
        let mut renderer = AnsiRenderer::stdout();
        let banner_style = theme::banner_style();

        if !interactive {
            let message = format!(
                "Non-interactive environment detected. Auto-approving '{}' tool.",
                tool_name
            );
            renderer.line_with_style(banner_style, &message)?;
            return Ok(true);
        }

        let header = format!("Tool Permission Request: {}", tool_name);
        renderer.line_with_style(banner_style, &header)?;
        renderer.line_with_style(
            banner_style,
            &format!("The agent wants to use the '{}' tool.", tool_name),
        )?;
        renderer.line_with_style(banner_style, "")?;
        renderer.line_with_style(
            banner_style,
            "This decision applies to the current request only.",
        )?;
        renderer.line_with_style(
            banner_style,
            "Update the policy file or use CLI flags to change the default.",
        )?;
        renderer.line_with_style(banner_style, "")?;

        if AUTO_ALLOW_TOOLS.contains(&tool_name) {
            renderer.line_with_style(
                banner_style,
                &format!(
                    "Auto-approving '{}' tool (default trusted tool).",
                    tool_name
                ),
            )?;
            return Ok(true);
        }

        let rgb = theme::banner_color();
        let to_ansi_256 = |value: u8| -> u8 {
            if value < 48 {
                0
            } else if value < 114 {
                1
            } else {
                ((value - 35) / 40).min(5)
            }
        };
        let rgb_to_index = |r: u8, g: u8, b: u8| -> u8 {
            let r_idx = to_ansi_256(r);
            let g_idx = to_ansi_256(g);
            let b_idx = to_ansi_256(b);
            16 + 36 * r_idx + 6 * g_idx + b_idx
        };
        let color_index = rgb_to_index(rgb.0, rgb.1, rgb.2);
        let dialog_color = ConsoleColor::Color256(color_index);
        let tinted_style = ConsoleStyle::new().for_stderr().fg(dialog_color);

        let mut dialog_theme = ColorfulTheme::default();
        dialog_theme.prompt_style = tinted_style;
        dialog_theme.prompt_prefix = style("—".to_string()).for_stderr().fg(dialog_color);
        dialog_theme.prompt_suffix = style("—".to_string()).for_stderr().fg(dialog_color);
        dialog_theme.hint_style = ConsoleStyle::new().for_stderr().fg(dialog_color);
        dialog_theme.defaults_style = dialog_theme.hint_style.clone();
        dialog_theme.success_prefix = style("✓".to_string()).for_stderr().fg(dialog_color);
        dialog_theme.success_suffix = style("·".to_string()).for_stderr().fg(dialog_color);
        dialog_theme.error_prefix = style("✗".to_string()).for_stderr().fg(dialog_color);
        dialog_theme.error_style = ConsoleStyle::new().for_stderr().fg(dialog_color);
        dialog_theme.values_style = ConsoleStyle::new().for_stderr().fg(dialog_color);

        let prompt_text = format!("Allow the agent to use '{}'?", tool_name);

        match Confirm::with_theme(&dialog_theme)
            .with_prompt(prompt_text)
            .default(false)
            .interact()
        {
            Ok(confirmed) => {
                let message = if confirmed {
                    format!("✓ Approved: '{}' tool will run now", tool_name)
                } else {
                    format!("✗ Denied: '{}' tool will not run", tool_name)
                };
                let style = if confirmed {
                    MessageStyle::Tool
                } else {
                    MessageStyle::Error
                };
                renderer.line(style, &message)?;
                Ok(confirmed)
            }
            Err(e) => {
                renderer.line(
                    MessageStyle::Error,
                    &format!("Failed to read confirmation: {}", e),
                )?;
                Ok(false)
            }
        }
    }
}
