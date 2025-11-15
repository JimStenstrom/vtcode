//! Markdown rendering for terminal output with syntax highlighting support.
//!
//! This module provides a complete markdown rendering pipeline that converts
//! markdown text into styled terminal output. The rendering is broken down into:
//!
//! - **Parsing**: Converting markdown text to events
//! - **Highlighting**: Syntax highlighting for code blocks
//! - **Themes**: Theme loading and caching
//! - **Blocks**: Block-level element rendering
//! - **Inline**: Inline element and text handling
//! - **Styles**: ANSI style generation
//!
//! # Example
//!
//! ```rust
//! use vtcode_ui::markdown::render_markdown;
//!
//! let markdown = "# Hello\n\nThis is **bold** text.";
//! let lines = render_markdown(markdown);
//! ```

pub mod blocks;
pub mod highlighting;
pub mod inline;
pub mod parser;
pub mod styles;
pub mod themes;

// Re-export key types for convenience
pub use blocks::{MarkdownLine, MarkdownSegment};

use blocks::{
    build_prefix_segments, highlight_code_block, CodeBlockState, ListState,
};
use inline::{append_text, ensure_prefix, flush_current_line, push_blank_line, trim_trailing_blank_lines};
use parser::{MarkdownEvent, MarkdownTag};
use vtcode_config::loader::SyntaxHighlightingConfig;

use crate::theme::{self, ThemeStyles};
use anstyle::Style;

/// Render markdown text to styled lines that can be written to the terminal renderer.
///
/// This is the main entry point for markdown rendering. It parses the markdown,
/// processes all events, and returns a vector of styled lines ready for output.
///
/// # Arguments
///
/// * `source` - The markdown text to render
/// * `base_style` - The base ANSI style to use
/// * `theme_styles` - Theme styles for various elements
/// * `highlight_config` - Optional syntax highlighting configuration
///
/// # Returns
///
/// A vector of `MarkdownLine`, each containing styled segments.
pub fn render_markdown_to_lines(
    source: &str,
    base_style: Style,
    theme_styles: &ThemeStyles,
    highlight_config: Option<&SyntaxHighlightingConfig>,
) -> Vec<MarkdownLine> {
    let events = parser::parse_markdown(source);

    let mut lines = Vec::new();
    let mut current_line = MarkdownLine::default();
    let mut style_stack = vec![base_style];
    let mut blockquote_depth = 0usize;
    let mut list_stack: Vec<ListState> = Vec::new();
    let mut pending_list_prefix: Option<String> = None;
    let mut code_block: Option<CodeBlockState> = None;

    for event in events {
        // Handle code block accumulation
        if let Some(state) = code_block.as_mut() {
            match event {
                MarkdownEvent::Text(text) => {
                    state.buffer.push_str(&text);
                    continue;
                }
                MarkdownEvent::End(MarkdownTag::CodeBlock(_)) => {
                    flush_current_line(
                        &mut lines,
                        &mut current_line,
                        blockquote_depth,
                        &list_stack,
                        &mut pending_list_prefix,
                        theme_styles,
                        base_style,
                    );
                    let prefix = build_prefix_segments(
                        blockquote_depth,
                        &list_stack,
                        theme_styles,
                        base_style,
                    );
                    let highlighted = highlight_code_block(
                        &state.buffer,
                        state.language.as_deref(),
                        highlight_config,
                        theme_styles,
                        base_style,
                        &prefix,
                    );
                    lines.extend(highlighted);
                    push_blank_line(&mut lines);
                    code_block = None;
                    continue;
                }
                _ => {}
            }
        }

        // Process non-code-block events
        match event {
            MarkdownEvent::Start(tag) => handle_start_tag(
                tag,
                &mut style_stack,
                &mut blockquote_depth,
                &mut list_stack,
                &mut pending_list_prefix,
                theme_styles,
                base_style,
                &mut code_block,
            ),
            MarkdownEvent::End(tag) => handle_end_tag(
                tag,
                &mut style_stack,
                &mut blockquote_depth,
                &mut list_stack,
                &mut pending_list_prefix,
                &mut lines,
                &mut current_line,
                theme_styles,
                base_style,
            ),
            MarkdownEvent::Text(text) => append_text(
                &text,
                &mut current_line,
                &mut lines,
                &style_stack,
                blockquote_depth,
                &list_stack,
                &mut pending_list_prefix,
                theme_styles,
                base_style,
            ),
            MarkdownEvent::Code(code_text) => {
                ensure_prefix(
                    &mut current_line,
                    blockquote_depth,
                    &list_stack,
                    &mut pending_list_prefix,
                    theme_styles,
                    base_style,
                );
                current_line.push_segment(
                    styles::inline_code_style(theme_styles, base_style),
                    &code_text,
                );
            }
            MarkdownEvent::SoftBreak => {
                append_text(
                    " ",
                    &mut current_line,
                    &mut lines,
                    &style_stack,
                    blockquote_depth,
                    &list_stack,
                    &mut pending_list_prefix,
                    theme_styles,
                    base_style,
                );
            }
            MarkdownEvent::HardBreak => {
                flush_current_line(
                    &mut lines,
                    &mut current_line,
                    blockquote_depth,
                    &list_stack,
                    &mut pending_list_prefix,
                    theme_styles,
                    base_style,
                );
            }
            MarkdownEvent::Rule => {
                flush_current_line(
                    &mut lines,
                    &mut current_line,
                    blockquote_depth,
                    &list_stack,
                    &mut pending_list_prefix,
                    theme_styles,
                    base_style,
                );
                let mut line = MarkdownLine::default();
                let rule_style = styles::rule_style(theme_styles);
                line.push_segment(rule_style, "―".repeat(32).as_str());
                lines.push(line);
                push_blank_line(&mut lines);
            }
            MarkdownEvent::TaskListMarker(checked) => {
                ensure_prefix(
                    &mut current_line,
                    blockquote_depth,
                    &list_stack,
                    &mut pending_list_prefix,
                    theme_styles,
                    base_style,
                );
                let marker = if checked { "[x] " } else { "[ ] " };
                current_line.push_segment(base_style, marker);
            }
            MarkdownEvent::Html(html) => append_text(
                &html,
                &mut current_line,
                &mut lines,
                &style_stack,
                blockquote_depth,
                &list_stack,
                &mut pending_list_prefix,
                theme_styles,
                base_style,
            ),
            MarkdownEvent::FootnoteReference(reference) => append_text(
                &format!("[^{}]", reference),
                &mut current_line,
                &mut lines,
                &style_stack,
                blockquote_depth,
                &list_stack,
                &mut pending_list_prefix,
                theme_styles,
                base_style,
            ),
        }
    }

    // Handle unclosed code block
    if let Some(state) = code_block {
        flush_current_line(
            &mut lines,
            &mut current_line,
            blockquote_depth,
            &list_stack,
            &mut pending_list_prefix,
            theme_styles,
            base_style,
        );
        let prefix = build_prefix_segments(blockquote_depth, &list_stack, theme_styles, base_style);
        let highlighted = highlight_code_block(
            &state.buffer,
            state.language.as_deref(),
            highlight_config,
            theme_styles,
            base_style,
            &prefix,
        );
        lines.extend(highlighted);
    }

    // Flush any remaining content
    if !current_line.segments.is_empty() {
        lines.push(current_line);
    }

    trim_trailing_blank_lines(&mut lines);
    lines
}

/// Convenience helper that renders markdown using the active theme.
///
/// Returns the styled lines so callers can perform custom handling or assertions in tests.
pub fn render_markdown(source: &str) -> Vec<MarkdownLine> {
    let styles = theme::active_styles();
    render_markdown_to_lines(source, Style::default(), &styles, None)
}

/// Handle start tag events, updating rendering state.
fn handle_start_tag(
    tag: MarkdownTag,
    style_stack: &mut Vec<Style>,
    blockquote_depth: &mut usize,
    list_stack: &mut Vec<ListState>,
    pending_list_prefix: &mut Option<String>,
    theme_styles: &ThemeStyles,
    base_style: Style,
    code_block: &mut Option<CodeBlockState>,
) {
    match tag {
        MarkdownTag::Paragraph => {}
        MarkdownTag::Heading(level) => {
            style_stack.push(styles::heading_style(level, theme_styles, base_style));
        }
        MarkdownTag::BlockQuote => {
            *blockquote_depth += 1;
        }
        MarkdownTag::List(start) => {
            let depth = list_stack.len();
            let state = ListState::new(start, depth);
            list_stack.push(state);
        }
        MarkdownTag::Item => {
            if let Some(state) = list_stack.last_mut() {
                let bullet = state.start_item();
                *pending_list_prefix = Some(bullet);
            }
        }
        MarkdownTag::Emphasis => {
            let style = style_stack.last().copied().unwrap_or(base_style).italic();
            style_stack.push(style);
        }
        MarkdownTag::Strong => {
            let style = style_stack.last().copied().unwrap_or(base_style).bold();
            style_stack.push(style);
        }
        MarkdownTag::Strikethrough => {
            let style = style_stack
                .last()
                .copied()
                .unwrap_or(base_style)
                .strikethrough();
            style_stack.push(style);
        }
        MarkdownTag::Link | MarkdownTag::Image => {
            let style = style_stack
                .last()
                .copied()
                .unwrap_or(base_style)
                .underline();
            style_stack.push(style);
        }
        MarkdownTag::CodeBlock(kind) => {
            *code_block = Some(CodeBlockState::new(kind));
        }
        MarkdownTag::Table
        | MarkdownTag::TableHead
        | MarkdownTag::TableRow
        | MarkdownTag::TableCell
        | MarkdownTag::FootnoteDefinition
        | MarkdownTag::HtmlBlock => {}
    }
}

/// Handle end tag events, updating rendering state.
fn handle_end_tag(
    tag: MarkdownTag,
    style_stack: &mut Vec<Style>,
    blockquote_depth: &mut usize,
    list_stack: &mut Vec<ListState>,
    pending_list_prefix: &mut Option<String>,
    lines: &mut Vec<MarkdownLine>,
    current_line: &mut MarkdownLine,
    theme_styles: &ThemeStyles,
    base_style: Style,
) {
    match tag {
        MarkdownTag::Paragraph => {
            flush_current_line(
                lines,
                current_line,
                *blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
            push_blank_line(lines);
        }
        MarkdownTag::Heading(..) => {
            flush_current_line(
                lines,
                current_line,
                *blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
            if !style_stack.is_empty() {
                style_stack.pop();
            }
            push_blank_line(lines);
        }
        MarkdownTag::BlockQuote => {
            flush_current_line(
                lines,
                current_line,
                *blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
            if *blockquote_depth > 0 {
                *blockquote_depth -= 1;
            }
        }
        MarkdownTag::List(_) => {
            flush_current_line(
                lines,
                current_line,
                *blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
            if list_stack.pop().is_some() {
                if let Some(state) = list_stack.last() {
                    pending_list_prefix.replace(state.continuation.clone());
                } else {
                    pending_list_prefix.take();
                }
            }
            push_blank_line(lines);
        }
        MarkdownTag::Item => {
            flush_current_line(
                lines,
                current_line,
                *blockquote_depth,
                list_stack,
                pending_list_prefix,
                theme_styles,
                base_style,
            );
            if let Some(state) = list_stack.last() {
                pending_list_prefix.replace(state.continuation.clone());
            }
        }
        MarkdownTag::Emphasis
        | MarkdownTag::Strong
        | MarkdownTag::Strikethrough
        | MarkdownTag::Link
        | MarkdownTag::Image => {
            style_stack.pop();
        }
        MarkdownTag::CodeBlock(_) => {}
        MarkdownTag::Table
        | MarkdownTag::TableHead
        | MarkdownTag::TableRow
        | MarkdownTag::TableCell
        | MarkdownTag::FootnoteDefinition
        | MarkdownTag::HtmlBlock => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_markdown() {
        let lines = render_markdown("# Hello\n\nWorld");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_code_block() {
        let markdown = "```rust\nfn main() {}\n```";
        let lines = render_markdown(markdown);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_list() {
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let lines = render_markdown(markdown);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_render_inline_code() {
        let markdown = "This is `code` inline.";
        let lines = render_markdown(markdown);
        assert!(!lines.is_empty());
    }
}
