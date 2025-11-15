//! Block-level element rendering for markdown.
//!
//! This module handles rendering of block-level markdown elements like
//! code blocks, lists, blockquotes, and headings.

use super::parser::CodeBlockKind;
use super::styles;
use crate::theme::ThemeStyles;
use anstyle::Style;
use std::cmp::max;
use syntect::util::LinesWithEndings;
use vtcode_config::loader::SyntaxHighlightingConfig;

pub const LIST_INDENT_WIDTH: usize = 2;
const CODE_EXTRA_INDENT: &str = "    ";

/// A styled text segment.
#[derive(Clone, Debug)]
pub struct MarkdownSegment {
    pub style: Style,
    pub text: String,
}

impl MarkdownSegment {
    pub fn new(style: Style, text: impl Into<String>) -> Self {
        Self {
            style,
            text: text.into(),
        }
    }
}

/// A rendered line composed of styled segments.
#[derive(Clone, Debug, Default)]
pub struct MarkdownLine {
    pub segments: Vec<MarkdownSegment>,
}

impl MarkdownLine {
    pub fn push_segment(&mut self, style: Style, text: &str) {
        if text.is_empty() {
            return;
        }
        // Merge with last segment if styles match
        if let Some(last) = self.segments.last_mut() {
            if last.style == style {
                last.text.push_str(text);
                return;
            }
        }
        self.segments.push(MarkdownSegment::new(style, text));
    }

    pub fn prepend_segments(&mut self, segments: &[PrefixSegment]) {
        if segments.is_empty() {
            return;
        }
        let mut prefixed = Vec::with_capacity(segments.len() + self.segments.len());
        for segment in segments {
            prefixed.push(MarkdownSegment::new(segment.style, segment.text.clone()));
        }
        prefixed.append(&mut self.segments);
        self.segments = prefixed;
    }

    pub fn is_empty(&self) -> bool {
        self.segments
            .iter()
            .all(|segment| segment.text.trim().is_empty())
    }
}

/// Prefix segment for blockquotes and list continuations.
#[derive(Clone, Debug)]
pub struct PrefixSegment {
    pub style: Style,
    pub text: String,
}

impl PrefixSegment {
    pub fn new(style: Style, text: impl Into<String>) -> Self {
        Self {
            style,
            text: text.into(),
        }
    }
}

/// State for code block rendering.
#[derive(Clone, Debug)]
pub struct CodeBlockState {
    pub language: Option<String>,
    pub buffer: String,
}

impl CodeBlockState {
    pub fn new(kind: CodeBlockKind) -> Self {
        let language = match kind {
            CodeBlockKind::Fenced(info) => info
                .split_whitespace()
                .next()
                .filter(|lang| !lang.is_empty())
                .map(|lang| lang.to_string()),
            CodeBlockKind::Indented => None,
        };
        Self {
            language,
            buffer: String::new(),
        }
    }
}

/// List state tracking for nested lists.
#[derive(Clone, Debug)]
pub struct ListState {
    pub kind: ListKind,
    pub depth: usize,
    pub continuation: String,
}

impl ListState {
    pub fn new(start: Option<usize>, depth: usize) -> Self {
        let kind = start
            .map(|value| ListKind::Ordered {
                next: max(1, value),
            })
            .unwrap_or(ListKind::Unordered);
        Self {
            kind,
            depth,
            continuation: String::new(),
        }
    }

    /// Start a new list item and return the bullet/number prefix.
    pub fn start_item(&mut self) -> String {
        let indent = " ".repeat(self.depth * LIST_INDENT_WIDTH);
        match &mut self.kind {
            ListKind::Unordered => {
                let bullet = format!("{}- ", indent);
                self.continuation = format!("{}  ", indent);
                bullet
            }
            ListKind::Ordered { next } => {
                let bullet = format!("{}{}. ", indent, *next);
                let width = bullet.len().saturating_sub(indent.len());
                self.continuation = format!("{}{}", indent, " ".repeat(width));
                *next += 1;
                bullet
            }
        }
    }
}

/// List kind: ordered or unordered.
#[derive(Clone, Debug)]
pub enum ListKind {
    Unordered,
    Ordered { next: usize },
}

/// Build prefix segments for blockquotes and lists.
pub fn build_prefix_segments(
    blockquote_depth: usize,
    list_stack: &[ListState],
    theme_styles: &ThemeStyles,
    base_style: Style,
) -> Vec<PrefixSegment> {
    let mut segments = Vec::new();

    // Add blockquote markers
    for _ in 0..blockquote_depth {
        segments.push(PrefixSegment::new(theme_styles.secondary.italic(), "│ "));
    }

    // Add list continuation
    if !list_stack.is_empty() {
        let mut continuation = String::new();
        for state in list_stack {
            continuation.push_str(&state.continuation);
        }
        if !continuation.is_empty() {
            segments.push(PrefixSegment::new(base_style, continuation));
        }
    }

    segments
}

/// Highlight and render a code block.
///
/// Attempts syntax highlighting if enabled and available, otherwise
/// renders as plain text with code block styling.
pub fn highlight_code_block(
    code: &str,
    language: Option<&str>,
    highlight_config: Option<&SyntaxHighlightingConfig>,
    theme_styles: &ThemeStyles,
    base_style: Style,
    prefix_segments: &[PrefixSegment],
) -> Vec<MarkdownLine> {
    let mut lines = Vec::new();
    let mut augmented_prefix = prefix_segments.to_vec();
    augmented_prefix.push(PrefixSegment::new(base_style, CODE_EXTRA_INDENT));

    // Try syntax highlighting if enabled
    if let Some(config) = highlight_config.filter(|cfg| cfg.enabled) {
        if let Some(highlighted) = super::highlighting::try_highlight(code, language, config) {
            for segments in highlighted {
                let mut line = MarkdownLine::default();
                line.prepend_segments(&augmented_prefix);
                for (style, text) in segments {
                    line.push_segment(style, &text);
                }
                lines.push(line);
            }
            return lines;
        }
    }

    // Fallback: render as plain text with code style
    for raw_line in LinesWithEndings::from(code) {
        let trimmed = raw_line.trim_end_matches('\n');
        let mut line = MarkdownLine::default();
        line.prepend_segments(&augmented_prefix);
        if !trimmed.is_empty() {
            line.push_segment(styles::code_block_style(theme_styles, base_style), trimmed);
        }
        lines.push(line);
    }

    // Add final empty line if code ends with newline
    if code.ends_with('\n') {
        let mut line = MarkdownLine::default();
        line.prepend_segments(&augmented_prefix);
        lines.push(line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_line_push_segment() {
        let mut line = MarkdownLine::default();
        let style = Style::new().bold();

        line.push_segment(style, "Hello");
        assert_eq!(line.segments.len(), 1);

        // Should merge with same style
        line.push_segment(style, " World");
        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].text, "Hello World");

        // Should create new segment with different style
        line.push_segment(Style::new().italic(), "!");
        assert_eq!(line.segments.len(), 2);
    }

    #[test]
    fn test_list_state() {
        let mut state = ListState::new(None, 0);
        let bullet = state.start_item();
        assert_eq!(bullet, "- ");

        let mut state = ListState::new(Some(1), 0);
        let bullet = state.start_item();
        assert_eq!(bullet, "1. ");

        let bullet = state.start_item();
        assert_eq!(bullet, "2. ");
    }

    #[test]
    fn test_code_block_state() {
        let state = CodeBlockState::new(CodeBlockKind::Fenced("rust".to_string()));
        assert_eq!(state.language, Some("rust".to_string()));

        let state = CodeBlockState::new(CodeBlockKind::Indented);
        assert_eq!(state.language, None);
    }
}
