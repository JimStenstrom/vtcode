use std::cmp::min;

use ansi_to_tui::IntoText;
use anstyle::{AnsiColor, Color as AnsiColorEnum, Effects};
use line_clipping::cohen_sutherland::clip_line;
use line_clipping::{LineSegment, Point, Window};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::tui::{
    style::ratatui_style_from_inline,
    types::{InlineMessageKind, InlineSegment, InlineTextStyle},
};
use vtcode_config::constants::ui;

use super::{message::MessageLine, Session};

// ============================================================================
// MESSAGE OPERATIONS
// ============================================================================

impl Session {
    /// Pushes a new message line with the specified segments.
    ///
    /// This creates a new line in the transcript with a new revision number
    /// and adjusts scrolling if needed.
    pub(super) fn push_line(&mut self, kind: InlineMessageKind, segments: Vec<InlineSegment>) {
        let previous_max_offset = self.current_max_scroll_offset();
        let revision = self.next_revision();
        self.display_state.lines.push(MessageLine {
            kind,
            segments,
            revision,
        });
        self.invalidate_scroll_metrics();
        self.adjust_scroll_after_change(previous_max_offset);
    }

    /// Appends a segment inline, handling newlines and carriage returns.
    ///
    /// This handles streaming text that may contain control characters.
    /// Newlines create new lines, while carriage returns reset the current line.
    pub(super) fn append_inline(&mut self, kind: InlineMessageKind, segment: InlineSegment) {
        let previous_max_offset = self.current_max_scroll_offset();
        let mut remaining = segment.text.as_str();
        let style = segment.style.clone();

        while !remaining.is_empty() {
            if let Some((index, control)) = remaining
                .char_indices()
                .find(|(_, ch)| matches!(ch, '\n' | '\r'))
            {
                let (text, _) = remaining.split_at(index);
                if !text.is_empty() {
                    self.append_text(kind, text, &style);
                }

                let control_char = control;
                let next_index = index + control_char.len_utf8();
                remaining = &remaining[next_index..];

                match control_char {
                    '\n' => self.start_line(kind),
                    '\r' => {
                        if remaining.starts_with('\n') {
                            remaining = &remaining[1..];
                            self.start_line(kind);
                        } else {
                            self.reset_line(kind);
                        }
                    }
                    _ => {}
                }
            } else {
                if !remaining.is_empty() {
                    self.append_text(kind, remaining, &style);
                }
                break;
            }
        }

        self.invalidate_scroll_metrics();
        self.adjust_scroll_after_change(previous_max_offset);
    }

    /// Replaces the last N lines with new content.
    ///
    /// Used for updating streaming content or correcting previous output.
    pub(super) fn replace_last(
        &mut self,
        count: usize,
        kind: InlineMessageKind,
        lines: Vec<Vec<InlineSegment>>,
    ) {
        let previous_max_offset = self.current_max_scroll_offset();
        let remove_count = min(count, self.display_state.lines.len());
        for _ in 0..remove_count {
            self.display_state.lines.pop();
        }
        for segments in lines {
            let revision = self.next_revision();
            self.display_state.lines.push(MessageLine {
                kind,
                segments,
                revision,
            });
        }
        self.invalidate_scroll_metrics();
        self.adjust_scroll_after_change(previous_max_offset);
    }

    /// Appends text to the current line, or creates a new line if needed.
    ///
    /// This is the core method for adding text to the transcript. It handles
    /// tool code fence markers and efficiently merges text with matching styles.
    pub(super) fn append_text(&mut self, kind: InlineMessageKind, text: &str, style: &InlineTextStyle) {
        if text.is_empty() {
            return;
        }

        if kind == InlineMessageKind::Tool && self.handle_tool_code_fence_marker(text) {
            return;
        }

        let mut appended = false;

        let mut mark_revision = false;
        {
            if let Some(line) = self.display_state.lines.last_mut() {
                if line.kind == kind {
                    if let Some(last) = line.segments.last_mut() {
                        if last.style == *style {
                            last.text.push_str(text);
                            appended = true;
                            mark_revision = true;
                        }
                    }
                    if !appended {
                        line.segments.push(InlineSegment {
                            text: text.to_string(),
                            style: style.clone(),
                        });
                        appended = true;
                        mark_revision = true;
                    }
                }
            }
        }

        if mark_revision {
            let revision = self.next_revision();
            if let Some(line) = self.display_state.lines.last_mut() {
                if line.kind == kind {
                    line.revision = revision;
                }
            }
        }

        if appended {
            self.invalidate_scroll_metrics();
            return;
        }

        let can_reuse_last = self
            .display_state.lines
            .last()
            .map(|line| line.kind == kind && line.segments.is_empty())
            .unwrap_or(false);
        if can_reuse_last {
            let revision = self.next_revision();
            if let Some(line) = self.display_state.lines.last_mut() {
                line.segments.push(InlineSegment {
                    text: text.to_string(),
                    style: style.clone(),
                });
                line.revision = revision;
            }
            self.invalidate_scroll_metrics();
            return;
        }

        let revision = self.next_revision();
        self.display_state.lines.push(MessageLine {
            kind,
            segments: vec![InlineSegment {
                text: text.to_string(),
                style: style.clone(),
            }],
            revision,
        });

        self.invalidate_scroll_metrics();
    }

    /// Starts a new empty line of the specified kind.
    pub(super) fn start_line(&mut self, kind: InlineMessageKind) {
        self.push_line(kind, Vec::new());
    }

    /// Resets the current line by clearing its segments.
    ///
    /// If the last line is of the specified kind, its segments are cleared.
    /// Otherwise, a new empty line is started.
    pub(super) fn reset_line(&mut self, kind: InlineMessageKind) {
        let mut cleared = false;
        {
            if let Some(line) = self.display_state.lines.last_mut() {
                if line.kind == kind {
                    line.segments.clear();
                    cleared = true;
                }
            }
        }
        if cleared {
            let revision = self.next_revision();
            if let Some(line) = self.display_state.lines.last_mut() {
                if line.kind == kind {
                    line.revision = revision;
                }
            }
            self.invalidate_scroll_metrics();
            return;
        }
        self.start_line(kind);
    }
}

// ============================================================================
// MESSAGE RENDERING
// ============================================================================

impl Session {
    /// Renders a message line as a vector of styled spans.
    ///
    /// Handles different message kinds (User, Agent, Tool, Pty) with
    /// appropriate prefixes and styling.
    pub(super) fn render_message_spans(&self, index: usize) -> Vec<Span<'static>> {
        let Some(line) = self.display_state.lines.get(index) else {
            return vec![Span::raw(String::new())];
        };
        let mut spans = Vec::new();
        if line.kind == InlineMessageKind::Agent {
            spans.extend(self.agent_prefix_spans(line));
        } else if let Some(prefix) = self.prefix_text(line.kind) {
            let style = self.prefix_style(line);
            spans.push(Span::styled(
                prefix,
                ratatui_style_from_inline(&style, self.display_state.theme.foreground),
            ));
        }

        if line.kind == InlineMessageKind::Agent {
            spans.push(Span::raw(ui::INLINE_AGENT_MESSAGE_LEFT_PADDING));
        }

        if line.segments.is_empty() {
            if spans.is_empty() {
                spans.push(Span::raw(String::new()));
            }
            return spans;
        }

        if line.kind == InlineMessageKind::Tool {
            let tool_spans = self.render_tool_segments(line);
            if tool_spans.is_empty() {
                spans.push(Span::raw(String::new()));
            } else {
                spans.extend(tool_spans);
            }
            return spans;
        }

        if line.kind == InlineMessageKind::Pty {
            let prev_is_pty = index
                .checked_sub(1)
                .and_then(|prev| self.display_state.lines.get(prev))
                .map(|prev| prev.kind == InlineMessageKind::Pty)
                .unwrap_or(false);
            if !prev_is_pty {
                let mut combined = String::new();
                for segment in &line.segments {
                    combined.push_str(segment.text.as_str());
                }
                let header_text = if combined.trim().is_empty() {
                    ui::INLINE_PTY_PLACEHOLDER.to_string()
                } else {
                    combined.trim().to_string()
                };
                let label_style = InlineTextStyle::default()
                    .with_color(self.display_state.theme.primary.or(self.display_state.theme.foreground))
                    .bold();
                spans.push(Span::styled(
                    format!("[{}]", ui::INLINE_PTY_HEADER_LABEL),
                    ratatui_style_from_inline(&label_style, self.display_state.theme.foreground),
                ));
                spans.push(Span::raw(" "));
                let body_style = InlineTextStyle::default()
                    .with_color(self.display_state.theme.foreground)
                    .bold();
                // Parse ANSI escape sequences in PTY output for color support
                // Limit to last 30 lines for performance and readability
                let output_text = if header_text.lines().count() > 30 {
                    let lines: Vec<&str> = header_text.lines().collect();
                    let start = lines.len().saturating_sub(30);
                    format!(
                        "[... {} lines truncated ...]\n{}",
                        lines.len() - 30,
                        lines[start..].join("\n")
                    )
                } else {
                    header_text.clone()
                };

                if let Ok(parsed) = output_text.as_bytes().into_text() {
                    let base_style = parsed.style;
                    for line in &parsed.lines {
                        let line_style = base_style.patch(line.style);
                        for span in &line.spans {
                            let content = span.content.clone().into_owned();
                            if content.is_empty() {
                                continue;
                            }
                            let span_style = line_style.patch(span.style);
                            spans.push(Span::styled(content, span_style));
                        }
                        // Add newline between lines
                        spans.push(Span::raw("\n"));
                    }
                    // Remove trailing newline
                    if spans.last().map(|s| s.content.as_ref()) == Some("\n") {
                        spans.pop();
                    }
                } else {
                    // Fallback to plain text if ANSI parsing fails
                    spans.push(Span::styled(
                        output_text,
                        ratatui_style_from_inline(&body_style, self.display_state.theme.foreground),
                    ));
                }
                return spans;
            }
        }

        let fallback = self.text_fallback(line.kind).or(self.display_state.theme.foreground);
        for segment in &line.segments {
            let style = ratatui_style_from_inline(&segment.style, fallback);
            spans.push(Span::styled(segment.text.clone(), style));
        }

        if spans.is_empty() {
            spans.push(Span::raw(String::new()));
        }

        spans
    }

    /// Renders the agent prefix with optional quote marker and label.
    pub(super) fn agent_prefix_spans(&self, line: &MessageLine) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let prefix_style =
            ratatui_style_from_inline(&self.prefix_style(line), self.display_state.theme.foreground);
        if !ui::INLINE_AGENT_QUOTE_PREFIX.is_empty() {
            spans.push(Span::styled(
                ui::INLINE_AGENT_QUOTE_PREFIX.to_string(),
                prefix_style,
            ));
        }

        if let Some(label) = self.display_state.labels.agent.clone() {
            if !label.is_empty() {
                let label_style =
                    ratatui_style_from_inline(&self.prefix_style(line), self.display_state.theme.foreground);
                spans.push(Span::styled(label, label_style));
            }
        }

        spans
    }

    /// Renders tool segments as styled spans.
    ///
    /// Combines all segments into a single tool header line.
    pub(super) fn render_tool_segments(&self, line: &MessageLine) -> Vec<Span<'static>> {
        let mut combined = String::new();
        for segment in &line.segments {
            combined.push_str(segment.text.as_str());
        }

        if combined.is_empty() {
            return Vec::new();
        }

        // Always render tool calls as a single combined line
        self.render_tool_header_line(&combined)
    }

    /// Renders a tool header line with formatted tool name and parameters.
    ///
    /// Parses the tool line to extract:
    /// - Indentation
    /// - Status prefix (checkmarks, etc.)
    /// - Tool name with distinctive coloring
    /// - Action text with syntax highlighting
    /// - Parameters with name/value distinction
    pub(super) fn render_tool_header_line(&self, text: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let indent_len = text.chars().take_while(|ch| ch.is_whitespace()).count();
        let indent: String = text.chars().take(indent_len).collect();
        let mut remaining = if indent_len < text.len() {
            &text[indent_len..]
        } else {
            ""
        };

        if !indent.is_empty() {
            let mut indent_style = InlineTextStyle::default();
            indent_style.color = self.display_state.theme.tool_body.or(self.display_state.theme.foreground);
            spans.push(Span::styled(
                indent,
                ratatui_style_from_inline(&indent_style, self.display_state.theme.foreground),
            ));
        }

        if remaining.is_empty() {
            return spans;
        }

        remaining = self.strip_tool_status_prefix(remaining);
        if remaining.is_empty() {
            return spans;
        }

        let (name, tail) = if remaining.starts_with('[') {
            if let Some(end) = remaining.find(']') {
                let name = &remaining[1..end];
                let tail = &remaining[end + 1..];
                (name, tail)
            } else {
                (remaining, "")
            }
        } else {
            let mut name_end = remaining.len();
            for (index, character) in remaining.char_indices() {
                if character.is_whitespace() {
                    name_end = index;
                    break;
                }
            }
            remaining.split_at(name_end)
        };
        if !name.is_empty() {
            // Add bracket wrapper with different styling
            spans.push(Span::styled(
                "[",
                self.accent_style().add_modifier(Modifier::BOLD),
            ));

            // Get distinctive color based on the tool name for better differentiation
            let tool_name_style = self.tool_inline_style(name);

            spans.push(Span::styled(
                name.to_string(),
                ratatui_style_from_inline(&tool_name_style, self.display_state.theme.foreground),
            ));

            spans.push(Span::styled(
                "] ",
                self.accent_style().add_modifier(Modifier::BOLD),
            ));
        }

        let trimmed_tail = tail.trim_start();
        if !trimmed_tail.is_empty() {
            // Parse the tail to extract tool action and parameters for better formatting
            let parts: Vec<&str> = trimmed_tail.split(" · ").collect();
            if parts.len() > 1 {
                // Format as "action → description · parameter1 · parameter2"
                let action = parts[0];
                let body_style = InlineTextStyle::default()
                    .with_color(self.display_state.theme.tool_body.or(self.display_state.theme.foreground));

                // Parse and style the action text with special highlighting
                self.render_styled_action_text(&mut spans, action, &body_style);

                // Format additional parameters (limit to avoid multi-line)
                let max_parts = 3; // Limit parameters to keep on one line
                for (i, part) in parts[1..].iter().enumerate() {
                    if i >= max_parts {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            "· ...",
                            self.accent_style().add_modifier(Modifier::DIM),
                        ));
                        break;
                    }
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        "·",
                        self.accent_style().add_modifier(Modifier::DIM),
                    ));
                    spans.push(Span::raw(" "));

                    // Differentiate between parameter names and values
                    let param_parts: Vec<&str> = part.split(": ").collect();
                    if param_parts.len() > 1 {
                        // Parameter name (before colon) - in accent color with bold
                        spans.push(Span::styled(
                            format!("{}: ", param_parts[0]),
                            self.accent_style().add_modifier(Modifier::BOLD),
                        ));

                        // Parameter value (after colon) - highlighted with different color
                        let value_style = InlineTextStyle::default()
                            .with_color(Some(AnsiColor::Green.into())) // Green for argument values
                            .bold();
                        spans.push(Span::styled(
                            param_parts[1].to_string(),
                            ratatui_style_from_inline(&value_style, self.display_state.theme.foreground),
                        ));
                    } else {
                        spans.push(Span::styled(
                            part.to_string(),
                            ratatui_style_from_inline(&body_style, self.display_state.theme.foreground),
                        ));
                    }
                }
            } else {
                // Fallback for original formatting
                let body_style = InlineTextStyle::default()
                    .with_color(self.display_state.theme.tool_body.or(self.display_state.theme.foreground));

                // Simplify common tool call patterns for human readability
                let mut simplified_text = self.simplify_tool_display(trimmed_tail);
                // Truncate to fit in one line (approximately 100 characters for readability)
                if simplified_text.len() > 100 {
                    simplified_text = simplified_text.chars().take(97).collect::<String>() + "...";
                }
                spans.push(Span::styled(
                    simplified_text,
                    ratatui_style_from_inline(&body_style, self.display_state.theme.foreground),
                ));
            }
        }

        spans
    }

    /// Renders action text with special styling for verbs and keywords.
    ///
    /// Highlights action verbs (List, Read, Write, etc.) and contextual words (in).
    pub(super) fn render_styled_action_text(
        &self,
        spans: &mut Vec<Span<'static>>,
        action: &str,
        body_style: &InlineTextStyle,
    ) {
        let words: Vec<&str> = action.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }

            if *word == "in" {
                // Style "in" with italic and different color (cyan)
                let in_style = InlineTextStyle::default()
                    .with_color(Some(AnsiColor::Cyan.into()))
                    .italic();
                spans.push(Span::styled(
                    word.to_string(),
                    ratatui_style_from_inline(&in_style, self.display_state.theme.foreground),
                ));
            } else if i < 2
                && (word.starts_with("List")
                    || word.starts_with("Read")
                    || word.starts_with("Write")
                    || word.starts_with("Find")
                    || word.starts_with("Search")
                    || word.starts_with("Run"))
            {
                // Highlight the main action verb (first 1-2 words) with bold and accent color
                let action_style = InlineTextStyle::default()
                    .with_color(self.display_state.theme.tool_accent.or(Some(AnsiColor::Yellow.into())))
                    .bold();
                spans.push(Span::styled(
                    word.to_string(),
                    ratatui_style_from_inline(&action_style, self.display_state.theme.foreground),
                ));
            } else {
                // Normal styling for other words
                spans.push(Span::styled(
                    word.to_string(),
                    ratatui_style_from_inline(body_style, self.display_state.theme.foreground),
                ));
            }
        }
    }

    /// Strips status prefix icons (✓, ✗, ~, ✕) from tool output.
    pub(super) fn strip_tool_status_prefix<'a>(&self, text: &'a str) -> &'a str {
        let trimmed = text.trim_start();
        const STATUS_ICONS: [&str; 4] = ["✓", "✗", "~", "✕"];
        for icon in STATUS_ICONS {
            if trimmed.starts_with(icon) {
                return trimmed[icon.len()..].trim_start();
            }
        }
        text
    }

    /// Simplifies tool call display text for better human readability.
    ///
    /// Converts technical patterns like "file path/to/file" to more readable forms.
    pub(super) fn simplify_tool_display(&self, text: &str) -> String {
        // Common patterns to simplify for human readability
        let simplified = if text.starts_with("file ") {
            // Convert "file path/to/file" to "accessing path/to/file"
            text.replacen("file ", "accessing ", 1)
        } else if text.starts_with("path: ") {
            // Convert "path: path/to/file" to "file: path/to/file"
            text.replacen("path: ", "file: ", 1)
        } else if text.contains(" → file ") {
            // Convert complex patterns to simpler ones
            text.replace(" → file ", " → ")
        } else if text.starts_with("grep ") {
            // Simplify grep patterns for better readability
            text.replacen("grep ", "searching for ", 1)
        } else if text.starts_with("find ") {
            // Simplify find patterns
            text.replacen("find ", "finding ", 1)
        } else if text.starts_with("list ") {
            // Simplify list patterns
            text.replacen("list ", "listing ", 1)
        } else {
            // Return original text if no simplification needed
            text.to_string()
        };

        // Further simplify parameter displays
        self.format_tool_parameters(&simplified)
    }

    /// Formats tool parameters for better readability.
    ///
    /// Converts "pattern: xyz" to "matching 'xyz'" and "path: xyz" to "in 'xyz'".
    pub(super) fn format_tool_parameters(&self, text: &str) -> String {
        // Convert common parameter patterns to more readable formats
        let mut formatted = text.to_string();

        // Convert "pattern: xyz" to "matching 'xyz'"
        if formatted.contains("pattern: ") {
            formatted = formatted.replace("pattern: ", "matching '");
            // Close the quote if there's a parameter separator
            if formatted.contains(" · ") {
                formatted = formatted.replacen(" · ", "' · ", 1);
            } else if formatted.contains("  ") {
                formatted = formatted.replacen("  ", "' ", 1);
            } else {
                formatted.push('\'');
            }
        }

        // Convert "path: xyz" to "in 'xyz'"
        if formatted.contains("path: ") {
            formatted = formatted.replace("path: ", "in '");
            // Close the quote if there's a parameter separator
            if formatted.contains(" · ") {
                formatted = formatted.replacen(" · ", "' · ", 1);
            } else if formatted.contains("  ") {
                formatted = formatted.replacen("  ", "' ", 1);
            } else {
                formatted.push('\'');
            }
        }

        formatted
    }

    /// Normalizes tool names to group similar tools together.
    ///
    /// Groups variants like "grep", "rg", "ripgrep" under "search".
    pub(super) fn normalize_tool_name(&self, tool_name: &str) -> String {
        // Group similar tools under common names for consistent styling
        match tool_name.to_lowercase().as_str() {
            "grep" | "rg" | "ripgrep" | "grep_file" | "search" | "find" | "ag" => {
                "search".to_string()
            }
            "list" | "ls" | "dir" | "list_files" => "list".to_string(),
            "read" | "cat" | "file" | "read_file" => "read".to_string(),
            "write" | "edit" | "save" | "insert" | "edit_file" => "write".to_string(),
            "run" | "command" | "bash" | "sh" => "run".to_string(),
            _ => tool_name.to_string(),
        }
    }
}

// ============================================================================
// TOOL HANDLING
// ============================================================================

impl Session {
    /// Handles tool code fence markers (``` or ~~~).
    ///
    /// Tracks whether we're inside a tool code fence to prevent
    /// special rendering of code blocks in tool output.
    /// Returns true if the text was a fence marker and was handled.
    pub(super) fn handle_tool_code_fence_marker(&mut self, text: &str) -> bool {
        let trimmed = text.trim();
        let stripped = trimmed
            .strip_prefix("```")
            .or_else(|| trimmed.strip_prefix("~~~"));

        let Some(rest) = stripped else {
            return false;
        };

        if rest.contains("```") || rest.contains("~~~") {
            return false;
        }

        if self.display_state.in_tool_code_fence {
            self.display_state.in_tool_code_fence = false;
            self.remove_trailing_empty_tool_line();
        } else {
            self.display_state.in_tool_code_fence = true;
        }

        true
    }

    /// Removes the last line if it's an empty tool line.
    ///
    /// Called when closing a code fence to clean up trailing empty lines.
    pub(super) fn remove_trailing_empty_tool_line(&mut self) {
        let should_remove = self
            .display_state.lines
            .last()
            .map(|line| line.kind == InlineMessageKind::Tool && line.segments.is_empty())
            .unwrap_or(false);
        if should_remove {
            self.display_state.lines.pop();
            self.invalidate_scroll_metrics();
        }
    }
}

// ============================================================================
// MESSAGE REFLOW AND WRAPPING
// ============================================================================

impl Session {
    /// Reflows message lines to fit within the specified width.
    ///
    /// Handles different message kinds with appropriate wrapping and borders.
    pub(super) fn reflow_message_lines(&self, index: usize, width: u16) -> Vec<Line<'static>> {
        let Some(message) = self.display_state.lines.get(index) else {
            return vec![Line::default()];
        };

        if message.kind == InlineMessageKind::Tool {
            return self.reflow_tool_lines(index, width);
        }

        if message.kind == InlineMessageKind::Pty {
            return self.reflow_pty_lines(index, width);
        }

        let spans = self.render_message_spans(index);
        let base_line = Line::from(spans);
        if width == 0 {
            return vec![base_line];
        }

        let mut wrapped = Vec::new();
        let max_width = width as usize;

        if message.kind == InlineMessageKind::User && max_width > 0 {
            wrapped.push(self.message_divider_line(max_width, message.kind));
        }

        let mut lines = self.wrap_line(base_line, max_width);
        if !lines.is_empty() {
            lines = self.justify_wrapped_lines(lines, max_width, message.kind);
        }
        if lines.is_empty() {
            lines.push(Line::default());
        }
        wrapped.extend(lines);

        if message.kind == InlineMessageKind::User && max_width > 0 {
            wrapped.push(self.message_divider_line(max_width, message.kind));
        }

        if wrapped.is_empty() {
            wrapped.push(Line::default());
        }

        wrapped
    }

    /// Reflows tool lines with optional borders.
    ///
    /// Tool detail lines (italic) are wrapped in borders, while simple
    /// tool output is rendered without borders.
    pub(super) fn reflow_tool_lines(&self, index: usize, width: u16) -> Vec<Line<'static>> {
        let Some(line) = self.display_state.lines.get(index) else {
            return vec![Line::default()];
        };

        let max_width = if width == 0 {
            usize::MAX
        } else {
            width as usize
        };

        let mut border_style =
            ratatui_style_from_inline(&self.tool_border_style(), self.display_state.theme.foreground);
        border_style = border_style.add_modifier(Modifier::DIM);

        let is_detail = line
            .segments
            .iter()
            .any(|segment| segment.style.effects.contains(Effects::ITALIC));
        let next_is_tool = self
            .display_state.lines
            .get(index + 1)
            .map(|next| next.kind == InlineMessageKind::Tool)
            .unwrap_or(false);

        let is_end = !next_is_tool;

        let mut lines = Vec::new();
        if is_detail {
            let body_prefix = format!("{} ", ui::INLINE_BLOCK_BODY_LEFT);
            let content = self.render_tool_segments(line);
            lines.extend(self.wrap_block_lines(
                &body_prefix,
                &body_prefix,
                content,
                max_width,
                border_style.clone(),
            ));
        } else {
            // For simple tool output, render without borders
            let content = self.render_tool_segments(line);
            for segment in content {
                lines.push(Line::from(vec![segment]));
            }
        }

        if is_end {
            // Don't add bottom border for simple tool output
            // lines.push(self.block_footer_line(width, border_style));
        }

        if lines.is_empty() {
            lines.push(Line::default());
        }

        lines
    }

    /// Reflows PTY (terminal) output with borders and headers.
    ///
    /// PTY blocks have a header showing the command and status (LIVE/DONE).
    pub(super) fn reflow_pty_lines(&self, index: usize, width: u16) -> Vec<Line<'static>> {
        let Some(line) = self.display_state.lines.get(index) else {
            return vec![Line::default()];
        };

        let max_width = if width == 0 {
            usize::MAX
        } else {
            width as usize
        };

        if !self.pty_block_has_content(index) {
            return Vec::new();
        }

        let mut border_inline = InlineTextStyle::default();
        border_inline.color = self.display_state.theme.secondary.or(self.display_state.theme.foreground);
        let mut border_style = ratatui_style_from_inline(&border_inline, self.display_state.theme.foreground);
        border_style = border_style.add_modifier(Modifier::DIM);

        let header_inline = InlineTextStyle::default()
            .with_color(self.display_state.theme.primary.or(self.display_state.theme.foreground))
            .bold();
        let header_style = ratatui_style_from_inline(&header_inline, self.display_state.theme.foreground);

        let mut body_inline = InlineTextStyle::default();
        body_inline.color = self.display_state.theme.foreground;
        let mut body_style = ratatui_style_from_inline(&body_inline, self.display_state.theme.foreground);
        body_style = body_style.add_modifier(Modifier::BOLD);

        let prev_is_pty = index
            .checked_sub(1)
            .and_then(|prev| self.display_state.lines.get(prev))
            .map(|prev| prev.kind == InlineMessageKind::Pty)
            .unwrap_or(false);
        let next_is_pty = self
            .display_state.lines
            .get(index + 1)
            .map(|next| next.kind == InlineMessageKind::Pty)
            .unwrap_or(false);

        let is_start = !prev_is_pty;
        let is_end = !next_is_pty;

        let mut lines = Vec::new();

        let mut combined = String::new();
        for segment in &line.segments {
            combined.push_str(segment.text.as_str());
        }
        if is_start && is_end && combined.trim().is_empty() {
            return Vec::new();
        }
        let header_text = combined
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| ui::INLINE_PTY_PLACEHOLDER.to_string());

        if is_start {
            // Add top border line
            if max_width > 2 {
                let top_border_content = format!(
                    "{}{}{}",
                    ui::INLINE_BLOCK_TOP_LEFT,
                    ui::INLINE_BLOCK_HORIZONTAL.repeat(max_width.saturating_sub(2)),
                    ui::INLINE_BLOCK_TOP_RIGHT
                );
                lines.push(Line::from(vec![Span::styled(
                    top_border_content,
                    border_style.clone(),
                )]));
            }

            let mut header_spans = Vec::new();
            header_spans.push(Span::styled(
                format!("[{}]", ui::INLINE_PTY_HEADER_LABEL),
                header_style.clone(),
            ));
            header_spans.push(Span::raw(" "));
            let running_style = InlineTextStyle::default()
                .with_color(self.display_state.theme.secondary.or(self.display_state.theme.foreground))
                .italic();
            header_spans.push(Span::styled(
                ui::INLINE_PTY_RUNNING_LABEL.to_string(),
                ratatui_style_from_inline(&running_style, self.display_state.theme.foreground),
            ));
            if !header_text.is_empty() {
                header_spans.push(Span::raw(" "));
                header_spans.push(Span::styled(header_text.clone(), body_style.clone()));
            }
            let status_label = if is_end {
                ui::INLINE_PTY_STATUS_DONE
            } else {
                ui::INLINE_PTY_STATUS_LIVE
            };
            header_spans.push(Span::raw(" "));
            header_spans.push(Span::styled(
                format!("[{}]", status_label),
                self.accent_style()
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD),
            ));

            let first_prefix = format!("{} ", ui::INLINE_BLOCK_BODY_LEFT);
            let continuation_prefix = format!("{} ", ui::INLINE_BLOCK_BODY_LEFT);
            lines.extend(self.wrap_block_lines(
                &first_prefix,
                &continuation_prefix,
                header_spans,
                max_width,
                border_style.clone(),
            ));
        } else {
            let fallback = self
                .text_fallback(InlineMessageKind::Pty)
                .or(self.display_state.theme.foreground);
            let mut body_spans = Vec::new();
            for segment in &line.segments {
                let style = ratatui_style_from_inline(&segment.style, fallback);
                body_spans.push(Span::styled(segment.text.clone(), style));
            }
            let body_prefix = format!("{} ", ui::INLINE_BLOCK_BODY_LEFT);
            lines.extend(self.wrap_block_lines(
                &body_prefix,
                &body_prefix,
                body_spans,
                max_width,
                border_style.clone(),
            ));
        }

        if is_end {
            // Don't add bottom border for PTY output either
            // lines.push(self.block_footer_line(width, border_style));
        }

        if lines.is_empty() {
            lines.push(Line::default());
        }

        lines
    }

    /// Wraps block content with left and right borders.
    ///
    /// Used for tool and PTY output to create bordered blocks.
    pub(super) fn wrap_block_lines(
        &self,
        first_prefix: &str,
        _continuation_prefix: &str,
        content: Vec<Span<'static>>,
        max_width: usize,
        border_style: Style,
    ) -> Vec<Line<'static>> {
        if max_width < 2 {
            return vec![Line::from(vec![Span::styled(
                format!("{}││", first_prefix),
                border_style,
            )])];
        }

        let right_border = ui::INLINE_BLOCK_BODY_RIGHT;
        let prefix_width = first_prefix.chars().count();
        let border_width = right_border.chars().count();
        let consumed_width = prefix_width.saturating_add(border_width);
        let content_width = max_width.saturating_sub(consumed_width);

        if max_width == usize::MAX {
            let mut spans = vec![Span::styled(first_prefix.to_string(), border_style)];
            spans.extend(content);
            spans.push(Span::styled(right_border.to_string(), border_style));
            return vec![Line::from(spans)];
        }

        let mut wrapped = self.wrap_line(Line::from(content), content_width);
        if wrapped.is_empty() {
            wrapped.push(Line::default());
        }

        // Add borders to each wrapped line
        for line in wrapped.iter_mut() {
            let line_width = line.spans.iter().map(|s| s.width()).sum::<usize>();
            let padding = content_width.saturating_sub(line_width);

            let mut new_spans = vec![Span::styled(first_prefix.to_string(), border_style)];
            new_spans.extend(line.spans.drain(..));
            if padding > 0 {
                new_spans.push(Span::styled(" ".repeat(padding), Style::default()));
            }
            new_spans.push(Span::styled(right_border.to_string(), border_style));
            line.spans = new_spans;
        }

        wrapped
    }

    /// Wraps a line to fit within max_width, handling grapheme clusters properly.
    ///
    /// This is the core wrapping algorithm that respects Unicode grapheme boundaries
    /// and uses line clipping for proper handling of wide characters.
    pub(super) fn wrap_line(&self, line: Line<'static>, max_width: usize) -> Vec<Line<'static>> {
        use std::mem;

        if max_width == 0 {
            return vec![Line::default()];
        }

        fn push_span(spans: &mut Vec<Span<'static>>, style: &Style, text: &str) {
            if text.is_empty() {
                return;
            }

            if let Some(last) = spans.last_mut().filter(|last| last.style == *style) {
                last.content.to_mut().push_str(text);
                return;
            }

            spans.push(Span::styled(text.to_string(), *style));
        }

        let mut rows = Vec::new();
        let mut current_spans: Vec<Span<'static>> = Vec::new();
        let mut current_width = 0usize;
        let window = Window::new(0.0, max_width as f64, -1.0, 1.0);

        let flush_current = |spans: &mut Vec<Span<'static>>, rows: &mut Vec<Line<'static>>| {
            if spans.is_empty() {
                rows.push(Line::default());
            } else {
                rows.push(Line::from(mem::take(spans)));
            }
        };

        for span in line.spans.into_iter() {
            let style = span.style;
            let content = span.content.into_owned();
            if content.is_empty() {
                continue;
            }

            for piece in content.split_inclusive('\n') {
                let mut text = piece;
                let mut had_newline = false;
                if let Some(stripped) = text.strip_suffix('\n') {
                    text = stripped;
                    had_newline = true;
                    if let Some(without_carriage) = text.strip_suffix('\r') {
                        text = without_carriage;
                    }
                }

                if !text.is_empty() {
                    for grapheme in UnicodeSegmentation::graphemes(text, true) {
                        if grapheme.is_empty() {
                            continue;
                        }

                        let width = UnicodeWidthStr::width(grapheme);
                        if width == 0 {
                            push_span(&mut current_spans, &style, grapheme);
                            continue;
                        }

                        let mut attempts = 0usize;
                        loop {
                            let line_segment = LineSegment::new(
                                Point::new(current_width as f64, 0.0),
                                Point::new((current_width + width) as f64, 0.0),
                            );

                            match clip_line(line_segment, window) {
                                Some(clipped) => {
                                    let visible = (clipped.p2.x - clipped.p1.x).round() as usize;
                                    if visible == width {
                                        push_span(&mut current_spans, &style, grapheme);
                                        current_width += width;
                                        break;
                                    }

                                    if current_width == 0 {
                                        push_span(&mut current_spans, &style, grapheme);
                                        current_width += width;
                                        break;
                                    }

                                    flush_current(&mut current_spans, &mut rows);
                                    current_width = 0;
                                }
                                None => {
                                    if current_width == 0 {
                                        push_span(&mut current_spans, &style, grapheme);
                                        current_width += width;
                                        break;
                                    }

                                    flush_current(&mut current_spans, &mut rows);
                                    current_width = 0;
                                }
                            }

                            attempts += 1;
                            if attempts > 4 {
                                push_span(&mut current_spans, &style, grapheme);
                                current_width += width;
                                break;
                            }
                        }

                        if current_width >= max_width {
                            flush_current(&mut current_spans, &mut rows);
                            current_width = 0;
                        }
                    }
                }

                if had_newline {
                    flush_current(&mut current_spans, &mut rows);
                    current_width = 0;
                }
            }
        }

        if !current_spans.is_empty() {
            flush_current(&mut current_spans, &mut rows);
        } else if rows.is_empty() {
            rows.push(Line::default());
        }

        rows
    }

    /// Justifies wrapped lines for agent messages.
    ///
    /// Applies text justification to agent messages (excluding code blocks)
    /// to create a more polished appearance.
    pub(super) fn justify_wrapped_lines(
        &self,
        lines: Vec<Line<'static>>,
        max_width: usize,
        kind: InlineMessageKind,
    ) -> Vec<Line<'static>> {
        if max_width == 0 || kind != InlineMessageKind::Agent {
            return lines;
        }

        let total = lines.len();
        let mut justified = Vec::with_capacity(total);
        let mut in_fenced_block = false;
        for (index, line) in lines.into_iter().enumerate() {
            let is_last = index + 1 == total;
            let mut next_in_fenced_block = in_fenced_block;
            let is_fence_line = {
                let line_text_storage: std::borrow::Cow<'_, str> = if line.spans.len() == 1 {
                    std::borrow::Cow::Borrowed(line.spans[0].content.as_ref())
                } else {
                    std::borrow::Cow::Owned(
                        line.spans
                            .iter()
                            .map(|span| span.content.as_ref())
                            .collect::<String>(),
                    )
                };
                let line_text = line_text_storage.as_ref();
                let trimmed_start = line_text.trim_start();
                trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")
            };
            if is_fence_line {
                next_in_fenced_block = !in_fenced_block;
            }

            if !in_fenced_block
                && !is_fence_line
                && self.should_justify_message_line(&line, max_width, is_last)
            {
                justified.push(self.justify_message_line(&line, max_width));
            } else {
                justified.push(line);
            }

            in_fenced_block = next_in_fenced_block;
        }

        justified
    }

    /// Determines if a line should be justified.
    ///
    /// Only justifies plain text lines that are not too short or too long,
    /// and excludes lines starting with special characters (lists, headers, etc.).
    fn should_justify_message_line(
        &self,
        line: &Line<'static>,
        max_width: usize,
        is_last: bool,
    ) -> bool {
        if is_last || max_width == 0 {
            return false;
        }
        if line.spans.len() != 1 {
            return false;
        }
        let text = line.spans[0].content.as_ref();
        if text.trim().is_empty() {
            return false;
        }
        if text.starts_with(char::is_whitespace) {
            return false;
        }
        let trimmed = text.trim();
        if trimmed.starts_with(|ch: char| matches!(ch, '-' | '*' | '`' | '>' | '#')) {
            return false;
        }
        if trimmed.contains("```") {
            return false;
        }
        let width = UnicodeWidthStr::width(trimmed);
        if width >= max_width || width < max_width / 2 {
            return false;
        }

        justify_plain_text(text, max_width).is_some()
    }

    /// Justifies a single message line.
    fn justify_message_line(&self, line: &Line<'static>, max_width: usize) -> Line<'static> {
        let span = &line.spans[0];
        if let Some(justified) = justify_plain_text(span.content.as_ref(), max_width) {
            Line::from(vec![Span::styled(justified, span.style)])
        } else {
            line.clone()
        }
    }
}

// ============================================================================
// STYLING HELPERS
// ============================================================================

impl Session {
    /// Returns the distinctive inline style for a tool based on its name.
    ///
    /// Assigns different colors to different tool categories:
    /// - Blue: read operations
    /// - Green: list operations
    /// - Yellow: search operations
    /// - Magenta: write operations
    /// - Red: execution operations
    /// - Cyan: version control
    pub(super) fn tool_inline_style(&self, tool_name: &str) -> InlineTextStyle {
        let normalized_name = self.normalize_tool_name(tool_name);
        let mut style = InlineTextStyle::default().bold();

        // Assign distinctive colors based on normalized tool type
        style.color = match normalized_name.to_lowercase().as_str() {
            "read" => {
                // Blue for file reading operations
                Some(AnsiColor::Blue.into())
            }
            "list" => {
                // Green for listing operations
                Some(AnsiColor::Green.into())
            }
            "search" => {
                // Yellow for search operations
                Some(AnsiColor::Yellow.into())
            }
            "write" => {
                // Magenta for write/edit operations
                Some(AnsiColor::Magenta.into())
            }
            "run" => {
                // Red for execution operations
                Some(AnsiColor::Red.into())
            }
            "git" | "version_control" => {
                // Cyan for version control
                Some(AnsiColor::Cyan.into())
            }
            _ => {
                // Use the default tool accent color for other tools
                self.display_state.theme
                    .tool_accent
                    .or(self.display_state.theme.primary)
                    .or(self.display_state.theme.foreground)
            }
        };

        style
    }

    /// Returns the border style for tool output.
    pub(super) fn tool_border_style(&self) -> InlineTextStyle {
        self.border_inline_style()
    }

    /// Returns the prefix text for a message kind.
    ///
    /// User messages get a user label/prefix, other kinds have no prefix.
    pub(super) fn prefix_text(&self, kind: InlineMessageKind) -> Option<String> {
        match kind {
            InlineMessageKind::User => Some(
                self.display_state.labels
                    .user
                    .clone()
                    .unwrap_or_else(|| super::USER_PREFIX.to_string()),
            ),
            InlineMessageKind::Agent => None,
            InlineMessageKind::Policy => self.display_state.labels.agent.clone(),
            InlineMessageKind::Tool | InlineMessageKind::Pty | InlineMessageKind::Error => None,
            InlineMessageKind::Info => None,
        }
    }

    /// Returns the prefix style for a message line.
    ///
    /// Uses the first segment's color if available, otherwise falls back
    /// to the message kind's default color.
    pub(super) fn prefix_style(&self, line: &MessageLine) -> InlineTextStyle {
        let fallback = self.text_fallback(line.kind).or(self.display_state.theme.foreground);

        let color = line
            .segments
            .iter()
            .find_map(|segment| segment.style.color)
            .or(fallback);

        InlineTextStyle {
            color,
            ..InlineTextStyle::default()
        }
    }

    /// Returns the fallback text color for a message kind.
    pub(super) fn text_fallback(&self, kind: InlineMessageKind) -> Option<AnsiColorEnum> {
        match kind {
            InlineMessageKind::Agent | InlineMessageKind::Policy => {
                self.display_state.theme.primary.or(self.display_state.theme.foreground)
            }
            InlineMessageKind::User => self.display_state.theme.secondary.or(self.display_state.theme.foreground),
            InlineMessageKind::Tool | InlineMessageKind::Pty | InlineMessageKind::Error => {
                self.display_state.theme.primary.or(self.display_state.theme.foreground)
            }
            InlineMessageKind::Info => self.display_state.theme.foreground,
        }
    }
}

// ============================================================================
// STYLE UTILITIES
// ============================================================================

impl Session {
    /// Returns the default ratatui style with foreground color.
    pub(super) fn default_style(&self) -> Style {
        use crate::tui::style::ratatui_color_from_ansi;

        let mut style = Style::default();
        if let Some(foreground) = self.display_state.theme.foreground.map(ratatui_color_from_ansi) {
            style = style.fg(foreground);
        }
        style
    }

    /// Returns the accent inline style (primary color).
    pub(super) fn accent_inline_style(&self) -> InlineTextStyle {
        InlineTextStyle {
            color: self.display_state.theme.primary.or(self.display_state.theme.foreground),
            ..InlineTextStyle::default()
        }
    }

    /// Returns the accent style as a ratatui Style.
    pub(super) fn accent_style(&self) -> Style {
        ratatui_style_from_inline(&self.accent_inline_style(), self.display_state.theme.foreground)
    }

    /// Returns the border inline style (secondary color).
    pub(super) fn border_inline_style(&self) -> InlineTextStyle {
        InlineTextStyle {
            color: self.display_state.theme.secondary.or(self.display_state.theme.foreground),
            ..InlineTextStyle::default()
        }
    }

    /// Returns the border style as a ratatui Style with DIM modifier.
    pub(super) fn border_style(&self) -> Style {
        ratatui_style_from_inline(&self.border_inline_style(), self.display_state.theme.foreground)
            .add_modifier(Modifier::DIM)
    }
}

// ============================================================================
// ADDITIONAL RENDERING HELPERS
// ============================================================================

impl Session {
    /// Creates a divider line for user messages.
    pub(super) fn message_divider_line(&self, width: usize, kind: InlineMessageKind) -> Line<'static> {
        if width == 0 {
            return Line::default();
        }

        let content = ui::INLINE_USER_MESSAGE_DIVIDER_SYMBOL.repeat(width);
        let style = self.message_divider_style(kind);
        Line::from(vec![Span::styled(content, style)])
    }

    /// Returns the style for message divider lines.
    fn message_divider_style(&self, kind: InlineMessageKind) -> Style {
        let mut style = InlineTextStyle::default();
        if kind == InlineMessageKind::User {
            style.color = self.display_state.theme.primary.or(self.display_state.theme.foreground);
        } else {
            style.color = self.text_fallback(kind).or(self.display_state.theme.foreground);
        }
        let resolved = ratatui_style_from_inline(&style, self.display_state.theme.foreground);
        if kind == InlineMessageKind::User {
            resolved
        } else {
            resolved.add_modifier(Modifier::DIM)
        }
    }

    /// Checks if a PTY block has any non-empty content.
    ///
    /// Scans the entire PTY block (from start to end) to determine
    /// if there's any meaningful content to display.
    pub(super) fn pty_block_has_content(&self, index: usize) -> bool {
        if self.display_state.lines.is_empty() {
            return false;
        }

        let mut start = index;
        while start > 0 {
            let Some(previous) = self.display_state.lines.get(start - 1) else {
                break;
            };
            if previous.kind != InlineMessageKind::Pty {
                break;
            }
            start -= 1;
        }

        let mut end = index;
        while end + 1 < self.display_state.lines.len() {
            let Some(next) = self.display_state.lines.get(end + 1) else {
                break;
            };
            if next.kind != InlineMessageKind::Pty {
                break;
            }
            end += 1;
        }

        for line in &self.display_state.lines[start..=end] {
            if line
                .segments
                .iter()
                .any(|segment| !segment.text.trim().is_empty())
            {
                return true;
            }
        }

        false
    }
}

// ============================================================================
// FREE FUNCTIONS
// ============================================================================

/// Justifies plain text by distributing spaces evenly.
///
/// Returns None if the text cannot or should not be justified.
fn justify_plain_text(text: &str, max_width: usize) -> Option<String> {
    let trimmed = text.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() <= 1 {
        return None;
    }

    let total_word_width: usize = words.iter().map(|word| UnicodeWidthStr::width(*word)).sum();
    if total_word_width >= max_width {
        return None;
    }

    let gaps = words.len() - 1;
    let spaces_needed = max_width.saturating_sub(total_word_width);
    if spaces_needed <= gaps {
        return None;
    }

    let base_space = spaces_needed / gaps;
    if base_space == 0 {
        return None;
    }
    let extra = spaces_needed % gaps;

    let mut output = String::with_capacity(max_width + gaps);
    for (index, word) in words.iter().enumerate() {
        output.push_str(word);
        if index < gaps {
            let mut count = base_space;
            if index < extra {
                count += 1;
            }
            for _ in 0..count {
                output.push(' ');
            }
        }
    }

    Some(output)
}
