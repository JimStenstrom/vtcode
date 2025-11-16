//! Input area rendering logic for the session TUI
//!
//! This module handles the rendering of the user input area, including:
//! - Main input block rendering with borders and styling
//! - Multi-line input layout and text wrapping
//! - Cursor position calculation and display
//! - Input status line rendering (git status, etc.)
//! - Trust mode indicators (Full Auto Trust, Tools Policy Trust)
//! - Placeholder text when input is empty
//! - Secure prompt masking (password input)
//!
//! The input area is rendered at the bottom of the viewport and adapts its height
//! based on the content, up to a maximum number of lines defined in the UI constants.

use super::super::{PLACEHOLDER_COLOR, Session, measure_text_width, ratatui_style_from_inline};
use vtcode_config::constants::ui;
use crate::tui::types::InlineTextStyle;
use anstyle::{Color as AnsiColorEnum, Effects};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Represents the rendered input with text and cursor position
struct InputRender {
    text: Text<'static>,
    cursor_x: u16,
    cursor_y: u16,
}

/// Buffer for a single line of input text with its prefix
#[derive(Default)]
struct InputLineBuffer {
    prefix: String,
    text: String,
    prefix_width: u16,
    text_width: u16,
}

impl InputLineBuffer {
    fn new(prefix: String, prefix_width: u16) -> Self {
        Self {
            prefix,
            text: String::new(),
            prefix_width,
            text_width: 0,
        }
    }
}

/// Layout information for multi-line input rendering
struct InputLayout {
    buffers: Vec<InputLineBuffer>,
    cursor_line_idx: usize,
    cursor_column: u16,
}

impl Session {
    /// Renders the user input area at the bottom of the screen.
    ///
    /// This is the main entry point for input area rendering. It handles:
    /// - Rendering the input block with borders and trust mode styling
    /// - Laying out multi-line input text with proper wrapping
    /// - Positioning the cursor at the correct location
    /// - Rendering the optional status line (git status, etc.)
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `area` - The rectangular area to render the input in
    pub(in crate::tui::session) fn render_input(&mut self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        if area.height == 0 {
            return;
        }

        let mut input_area = area;
        let mut status_area = None;
        let mut status_line = None;

        if area.height >= 2 {
            if let Some(line) = self.render_input_status_line(area.width) {
                let block_height = area.height.saturating_sub(1).max(1);
                input_area.height = block_height;
                status_area = Some(Rect::new(area.x, area.y + block_height, area.width, 1));
                status_line = Some(line);
            }
        }

        // Determine if we're in full auto trust mode and adjust styling accordingly
        let is_full_auto_trust = self.is_full_auto_trust();
        let border_style = if is_full_auto_trust {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            self.accent_style()
        };

        // Determine the trust mode title for the border
        let trust_title = if is_full_auto_trust {
            "Full Auto Trust"
        } else if self.is_tools_policy_trust() {
            "Tools Policy Trust"
        } else {
            ""
        };

        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(border_style)
            .title(trust_title);
        let inner = block.inner(input_area);
        let input_render = self.build_input_render(inner.width, inner.height);
        let paragraph = Paragraph::new(input_render.text)
            .style(self.default_style())
            .wrap(Wrap { trim: false })
            .block(block);
        frame.render_widget(paragraph, input_area);

        if self.cursor_should_be_visible() && inner.width > 0 && inner.height > 0 {
            let cursor_x = input_render
                .cursor_x
                .min(inner.width.saturating_sub(1))
                .saturating_add(inner.x);
            let cursor_y = input_render
                .cursor_y
                .min(inner.height.saturating_sub(1))
                .saturating_add(inner.y);
            frame.set_cursor_position((cursor_x, cursor_y));
        }

        if let (Some(status_rect), Some(line)) = (status_area, status_line) {
            let paragraph = Paragraph::new(line)
                .style(self.default_style())
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, status_rect);
        }
    }

    /// Calculates the desired number of lines for the input area.
    ///
    /// This determines how many lines the input needs to display all content,
    /// up to the maximum allowed by UI constants.
    ///
    /// # Arguments
    ///
    /// * `inner_width` - The available width inside the input block
    ///
    /// # Returns
    ///
    /// The number of lines needed, capped at the maximum
    pub(in crate::tui::session) fn desired_input_lines(&self, inner_width: u16) -> u16 {
        if inner_width == 0 || self.input_manager.content().is_empty() {
            return 1;
        }

        let prompt_width = UnicodeWidthStr::width(self.prompt_state.prompt_prefix.as_str()) as u16;
        let prompt_display_width = prompt_width.min(inner_width);
        let layout = self.input_layout(inner_width, prompt_display_width);
        let line_count = layout.buffers.len().max(1);
        let capped = line_count.min(ui::INLINE_INPUT_MAX_LINES.max(1));
        capped as u16
    }

    /// Updates the cached input height if it has changed.
    ///
    /// This triggers a recalculation of transcript rows when the input height changes,
    /// ensuring the layout remains consistent.
    ///
    /// # Arguments
    ///
    /// * `height` - The new input height in rows
    pub(in crate::tui::session) fn apply_input_height(&mut self, height: u16) {
        let resolved = height.max(Self::input_block_height_for_lines(1));
        if self.ui_state.input_height != resolved {
            self.ui_state.input_height = resolved;
            self.recalculate_transcript_rows();
        }
    }

    /// Calculates the total block height needed for a given number of input lines.
    ///
    /// This includes the borders (top and bottom) in addition to the content lines.
    ///
    /// # Arguments
    ///
    /// * `lines` - The number of content lines
    ///
    /// # Returns
    ///
    /// The total height including borders
    pub(in crate::tui::session) fn input_block_height_for_lines(lines: u16) -> u16 {
        lines.max(1).saturating_add(2)
    }

    /// Checks if the input status line should be displayed.
    ///
    /// The status line is shown if either left or right status text is non-empty.
    ///
    /// # Returns
    ///
    /// `true` if a status line should be displayed
    pub(in crate::tui::session) fn has_input_status(&self) -> bool {
        let left_present = self
            .prompt_state.input_status_left
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        if left_present {
            return true;
        }
        self.prompt_state.input_status_right
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
    }

    /// Calculates the layout for multi-line input text.
    ///
    /// This breaks the input into lines based on:
    /// - Explicit newlines in the input
    /// - Automatic wrapping when text exceeds available width
    /// - Proper indentation for continuation lines
    ///
    /// # Arguments
    ///
    /// * `width` - The total available width
    /// * `prompt_display_width` - The width of the prompt prefix
    ///
    /// # Returns
    ///
    /// An `InputLayout` containing line buffers and cursor position
    fn input_layout(&self, width: u16, prompt_display_width: u16) -> InputLayout {
        let indent_prefix = " ".repeat(prompt_display_width as usize);
        let mut buffers = vec![InputLineBuffer::new(
            self.prompt_state.prompt_prefix.clone(),
            prompt_display_width,
        )];
        let secure_prompt_active = self.secure_prompt_active();
        let mut cursor_line_idx = 0usize;
        let mut cursor_column = prompt_display_width;
        let input_content = self.input_manager.content();
        let cursor_pos = self.input_manager.cursor();
        let mut cursor_set = cursor_pos == 0;

        for (idx, ch) in input_content.char_indices() {
            if !cursor_set && cursor_pos == idx {
                if let Some(current) = buffers.last() {
                    cursor_line_idx = buffers.len() - 1;
                    cursor_column = current.prefix_width + current.text_width;
                    cursor_set = true;
                }
            }

            if ch == '\n' {
                let end = idx + ch.len_utf8();
                buffers.push(InputLineBuffer::new(
                    indent_prefix.clone(),
                    prompt_display_width,
                ));
                if !cursor_set && cursor_pos == end {
                    cursor_line_idx = buffers.len() - 1;
                    cursor_column = prompt_display_width;
                    cursor_set = true;
                }
                continue;
            }

            let display_ch = if secure_prompt_active { '•' } else { ch };
            let char_width = UnicodeWidthChar::width(display_ch).unwrap_or(0) as u16;

            if let Some(current) = buffers.last_mut() {
                let capacity = width.saturating_sub(current.prefix_width);
                if capacity > 0
                    && current.text_width + char_width > capacity
                    && !current.text.is_empty()
                {
                    buffers.push(InputLineBuffer::new(
                        indent_prefix.clone(),
                        prompt_display_width,
                    ));
                }
            }

            if let Some(current) = buffers.last_mut() {
                current.text.push(display_ch);
                current.text_width = current.text_width.saturating_add(char_width);
            }

            let end = idx + ch.len_utf8();
            if !cursor_set && cursor_pos == end {
                if let Some(current) = buffers.last() {
                    cursor_line_idx = buffers.len() - 1;
                    cursor_column = current.prefix_width + current.text_width;
                    cursor_set = true;
                }
            }
        }

        if !cursor_set {
            if let Some(current) = buffers.last() {
                cursor_line_idx = buffers.len() - 1;
                cursor_column = current.prefix_width + current.text_width;
            }
        }

        InputLayout {
            buffers,
            cursor_line_idx,
            cursor_column,
        }
    }

    /// Builds the final rendered input with styling and cursor position.
    ///
    /// This converts the input layout into styled text ready for rendering,
    /// handling:
    /// - Prompt prefix styling
    /// - Placeholder text when input is empty
    /// - Visible line windowing (showing bottom lines when content overflows)
    /// - Cursor position tracking
    ///
    /// # Arguments
    ///
    /// * `width` - The available width
    /// * `height` - The available height
    ///
    /// # Returns
    ///
    /// An `InputRender` with styled text and cursor position
    fn build_input_render(&self, width: u16, height: u16) -> InputRender {
        if width == 0 || height == 0 {
            return InputRender {
                text: Text::default(),
                cursor_x: 0,
                cursor_y: 0,
            };
        }

        let max_visible_lines = height.max(1).min(ui::INLINE_INPUT_MAX_LINES as u16) as usize;

        let mut prompt_style = self.prompt_state.prompt_style.clone();
        if prompt_style.color.is_none() {
            prompt_style.color = self.display_state.theme.primary.or(self.display_state.theme.foreground);
        }
        let prompt_style = ratatui_style_from_inline(&prompt_style, self.display_state.theme.foreground);
        let prompt_width = UnicodeWidthStr::width(self.prompt_state.prompt_prefix.as_str()) as u16;
        let prompt_display_width = prompt_width.min(width);

        if self.input_manager.content().is_empty() {
            let mut spans = Vec::new();
            spans.push(Span::styled(self.prompt_state.prompt_prefix.clone(), prompt_style));

            if let Some(placeholder) = &self.prompt_state.placeholder {
                let placeholder_style =
                    self.prompt_state.placeholder_style
                        .clone()
                        .unwrap_or_else(|| InlineTextStyle {
                            color: Some(AnsiColorEnum::Rgb(PLACEHOLDER_COLOR)),
                            bg_color: None,
                            effects: Effects::ITALIC,
                        });
                let style = ratatui_style_from_inline(
                    &placeholder_style,
                    Some(AnsiColorEnum::Rgb(PLACEHOLDER_COLOR)),
                );
                spans.push(Span::styled(placeholder.clone(), style));
            }

            return InputRender {
                text: Text::from(vec![Line::from(spans)]),
                cursor_x: prompt_display_width,
                cursor_y: 0,
            };
        }

        let accent_style =
            ratatui_style_from_inline(&self.accent_inline_style(), self.display_state.theme.foreground);
        let layout = self.input_layout(width, prompt_display_width);
        let total_lines = layout.buffers.len();
        let visible_limit = max_visible_lines.max(1);
        let mut start = total_lines.saturating_sub(visible_limit);
        if layout.cursor_line_idx < start {
            start = layout.cursor_line_idx.saturating_sub(visible_limit - 1);
        }
        let end = (start + visible_limit).min(total_lines);
        let cursor_y = layout.cursor_line_idx.saturating_sub(start) as u16;

        let mut lines = Vec::new();
        for buffer in &layout.buffers[start..end] {
            let mut spans = Vec::new();
            spans.push(Span::styled(buffer.prefix.clone(), prompt_style));
            if !buffer.text.is_empty() {
                spans.push(Span::styled(buffer.text.clone(), accent_style));
            }
            lines.push(Line::from(spans));
        }

        if lines.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                self.prompt_state.prompt_prefix.clone(),
                prompt_style,
            )]));
        }

        InputRender {
            text: Text::from(lines),
            cursor_x: layout.cursor_column,
            cursor_y,
        }
    }

    /// Renders the optional status line below the input.
    ///
    /// The status line can show:
    /// - Left-aligned status (e.g., git branch and status)
    /// - Right-aligned status (e.g., line/column info)
    /// - Both, with padding between them
    ///
    /// Git status indicators are styled with color:
    /// - Red/bold for dirty status
    /// - Green/bold for clean status
    ///
    /// # Arguments
    ///
    /// * `width` - The available width for the status line
    ///
    /// # Returns
    ///
    /// A styled `Line` if there is status to display, `None` otherwise
    fn render_input_status_line(&self, width: u16) -> Option<Line<'static>> {
        if width == 0 {
            return None;
        }

        let left = self
            .prompt_state.input_status_left
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let right = self
            .prompt_state.input_status_right
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        if left.is_none() && right.is_none() {
            return None;
        }

        let style = self.default_style().add_modifier(Modifier::DIM);
        let mut spans = Vec::new();

        match (left, right) {
            (Some(left_value), Some(right_value)) => {
                let left_width = measure_text_width(&left_value);
                let right_width = measure_text_width(&right_value);
                let padding = width.saturating_sub(left_width + right_width);

                spans.extend(self.create_git_status_spans(&left_value, style));

                if padding > 0 {
                    spans.push(Span::raw(" ".repeat(padding as usize)));
                } else {
                    spans.push(Span::raw(" ".to_string()));
                }
                spans.push(Span::styled(right_value, style));
            }
            (Some(left_value), None) => {
                spans.extend(self.create_git_status_spans(&left_value, style));
            }
            (None, Some(right_value)) => {
                let right_width = measure_text_width(&right_value);
                if width > right_width {
                    spans.push(Span::raw(" ".repeat((width - right_width) as usize)));
                }
                spans.push(Span::styled(right_value, style));
            }
            (None, None) => return None,
        }

        Some(Line::from(spans))
    }

    /// Creates styled spans for git status text.
    ///
    /// Parses git status text in the format "branch | status" and applies
    /// appropriate styling to the status indicator:
    /// - Red/bold for dirty status
    /// - Green/bold for clean status
    /// - Accent/bold for other indicators
    ///
    /// # Arguments
    ///
    /// * `text` - The status text to parse and style
    /// * `default_style` - The default style to use for non-indicator text
    ///
    /// # Returns
    ///
    /// A vector of styled spans
    #[allow(dead_code)]
    fn create_git_status_spans(&self, text: &str, default_style: Style) -> Vec<Span<'static>> {
        if let Some((branch_part, indicator_part)) = text.rsplit_once(" | ") {
            let mut spans = Vec::new();
            let branch_trim = branch_part.trim_end();
            if !branch_trim.is_empty() {
                spans.push(Span::styled(branch_trim.to_string(), default_style));
            }
            spans.push(Span::raw(" "));

            let indicator_trim = indicator_part.trim();
            let indicator_style = if indicator_trim == ui::HEADER_GIT_DIRTY_SUFFIX {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if indicator_trim == ui::HEADER_GIT_CLEAN_SUFFIX {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                self.accent_style().add_modifier(Modifier::BOLD)
            };

            spans.push(Span::styled(indicator_trim.to_string(), indicator_style));
            spans
        } else {
            vec![Span::styled(text.to_string(), default_style)]
        }
    }

    /// Checks if the cursor should be visible.
    ///
    /// The cursor is visible when both:
    /// - The cursor_visible flag is set
    /// - Input is currently enabled
    ///
    /// # Returns
    ///
    /// `true` if the cursor should be displayed
    fn cursor_should_be_visible(&self) -> bool {
        self.ui_state.cursor_visible && self.ui_state.input_enabled
    }

    /// Checks if a secure prompt (password input) is active.
    ///
    /// When active, input characters are masked with bullets.
    ///
    /// # Returns
    ///
    /// `true` if secure prompt is active
    fn secure_prompt_active(&self) -> bool {
        self.render_state.modal
            .as_ref()
            .and_then(|modal| modal.secure_prompt.as_ref())
            .is_some()
    }

    /// Checks if the workspace is in Full Auto Trust mode.
    ///
    /// This affects the input border styling (red/bold).
    ///
    /// # Returns
    ///
    /// `true` if Full Auto Trust mode is active
    fn is_full_auto_trust(&self) -> bool {
        self.render_state.header_context.workspace_trust.contains("full auto")
    }

    /// Checks if the workspace is in Tools Policy Trust mode.
    ///
    /// This affects the input border title.
    ///
    /// # Returns
    ///
    /// `true` if Tools Policy Trust mode is active
    fn is_tools_policy_trust(&self) -> bool {
        self.render_state.header_context.workspace_trust.contains("tools policy")
    }
}
