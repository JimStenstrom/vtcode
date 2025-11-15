//! Inline formatting and text handling.
//!
//! This module handles inline markdown elements like emphasis, strong, code,
//! and manages text flow with proper line wrapping.

use super::blocks::{ListState, MarkdownLine};
use crate::theme::ThemeStyles;
use anstyle::Style;

/// Append text to the current line, handling newlines and prefixes.
///
/// This function splits text on newlines and ensures each line has proper
/// prefixes for blockquotes and lists.
pub fn append_text(
    text: &str,
    current_line: &mut MarkdownLine,
    lines: &mut Vec<MarkdownLine>,
    style_stack: &[Style],
    blockquote_depth: usize,
    list_stack: &[ListState],
    pending_list_prefix: &mut Option<String>,
    theme_styles: &ThemeStyles,
    base_style: Style,
) {
    let style = style_stack.last().copied().unwrap_or(base_style);

    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if ch == '\n' {
            let segment = &text[start..idx];
            if !segment.is_empty() {
                ensure_prefix(
                    current_line,
                    blockquote_depth,
                    list_stack,
                    pending_list_prefix,
                    theme_styles,
                    base_style,
                );
                current_line.push_segment(style, segment);
            }
            lines.push(std::mem::take(current_line));
            start = idx + ch.len_utf8();
        }
    }

    if start < text.len() {
        let remaining = &text[start..];
        ensure_prefix(
            current_line,
            blockquote_depth,
            list_stack,
            pending_list_prefix,
            theme_styles,
            base_style,
        );
        current_line.push_segment(style, remaining);
    }
}

/// Ensure the current line has proper prefixes for blockquotes and lists.
pub fn ensure_prefix(
    current_line: &mut MarkdownLine,
    blockquote_depth: usize,
    list_stack: &[ListState],
    pending_list_prefix: &mut Option<String>,
    theme_styles: &ThemeStyles,
    base_style: Style,
) {
    // Only add prefix if line is empty
    if !current_line.segments.is_empty() {
        return;
    }

    // Add blockquote markers
    for _ in 0..blockquote_depth {
        current_line.push_segment(theme_styles.secondary.italic(), "│ ");
    }

    // Add list prefix (bullet/number or continuation)
    if let Some(prefix) = pending_list_prefix.take() {
        current_line.push_segment(base_style, &prefix);
    } else if !list_stack.is_empty() {
        let mut continuation = String::new();
        for state in list_stack {
            continuation.push_str(&state.continuation);
        }
        if !continuation.is_empty() {
            current_line.push_segment(base_style, &continuation);
        }
    }
}

/// Push a blank line, avoiding duplicates.
pub fn push_blank_line(lines: &mut Vec<MarkdownLine>) {
    if lines
        .last()
        .map(|line| line.segments.is_empty())
        .unwrap_or(false)
    {
        return;
    }
    lines.push(MarkdownLine::default());
}

/// Remove trailing blank lines from output.
pub fn trim_trailing_blank_lines(lines: &mut Vec<MarkdownLine>) {
    while lines
        .last()
        .map(|line| line.segments.is_empty())
        .unwrap_or(false)
    {
        lines.pop();
    }
}

/// Flush the current line to output.
///
/// If the line is empty but there's a pending list prefix, ensures the prefix
/// is added before flushing.
pub fn flush_current_line(
    lines: &mut Vec<MarkdownLine>,
    current_line: &mut MarkdownLine,
    blockquote_depth: usize,
    list_stack: &[ListState],
    pending_list_prefix: &mut Option<String>,
    theme_styles: &ThemeStyles,
    base_style: Style,
) {
    // If line is empty but we have a pending prefix, add it
    if current_line.segments.is_empty() {
        if pending_list_prefix.is_some() {
            ensure_prefix(
                current_line,
                blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
        }
    }

    // Only push non-empty lines
    if !current_line.segments.is_empty() {
        lines.push(std::mem::take(current_line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_blank_line() {
        let mut lines = Vec::new();
        push_blank_line(&mut lines);
        assert_eq!(lines.len(), 1);

        // Should not add duplicate blank line
        push_blank_line(&mut lines);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_trim_trailing_blank_lines() {
        let mut lines = vec![
            MarkdownLine::default(),
            MarkdownLine::default(),
            MarkdownLine::default(),
        ];
        trim_trailing_blank_lines(&mut lines);
        assert_eq!(lines.len(), 0);
    }
}
