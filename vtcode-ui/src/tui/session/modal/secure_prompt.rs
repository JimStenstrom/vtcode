//! Secure prompt rendering for sensitive input

use crate::tui::types::SecurePromptConfig;
use ratatui::{layout::Rect, Frame};
use tui_prompts::{Prompt, State, TextPrompt, TextRenderStyle, TextState};

/// Render a secure password-style prompt
pub fn render_secure_prompt(
    frame: &mut Frame<'_>,
    area: Rect,
    config: &SecurePromptConfig,
    input: &str,
    cursor: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let grapheme_count = input.chars().count();
    let sanitized: String = std::iter::repeat('•').take(grapheme_count).collect();
    let cursor_chars = input[..cursor].chars().count();

    let mut state = TextState::new().with_value(sanitized);
    state.focus();
    *state.position_mut() = cursor_chars;

    let prompt =
        TextPrompt::from(config.label.clone()).with_render_style(TextRenderStyle::Password);
    prompt.draw(frame, area, &mut state);
}
