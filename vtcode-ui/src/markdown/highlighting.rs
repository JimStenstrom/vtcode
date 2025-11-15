//! Syntax highlighting integration with syntect.
//!
//! This module handles code syntax highlighting using the syntect library,
//! converting highlighted output to ANSI-styled text segments.

use anstyle::Style;
use anstyle_syntect::to_anstyle;
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use vtcode_config::loader::SyntaxHighlightingConfig;

/// Global syntax set loaded once for all highlighting operations.
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Highlight code with syntax highlighting.
///
/// Returns a vector of lines, where each line is a vector of (style, text) segments.
/// Returns None if highlighting is disabled or fails.
///
/// # Arguments
///
/// * `code` - The code to highlight
/// * `language` - Optional language hint
/// * `config` - Syntax highlighting configuration
///
/// # Returns
///
/// Option containing highlighted lines, or None if highlighting should be skipped.
pub fn try_highlight(
    code: &str,
    language: Option<&str>,
    config: &SyntaxHighlightingConfig,
) -> Option<Vec<Vec<(Style, String)>>> {
    // Check file size limit
    let max_bytes = config.max_file_size_mb.saturating_mul(1024 * 1024);
    if max_bytes > 0 && code.len() > max_bytes {
        return None;
    }

    // Check if language is enabled
    if let Some(lang) = language {
        let enabled = config
            .enabled_languages
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(lang));
        if !enabled {
            return None;
        }
    }

    let syntax = select_syntax(language);
    let theme = super::themes::load_theme(&config.theme, config.cache_themes);

    highlight_with_theme(code, syntax, &theme)
}

/// Highlight code with a specific syntax and theme.
///
/// This is the core highlighting function that processes code line by line.
fn highlight_with_theme(
    code: &str,
    syntax: &SyntaxReference,
    theme: &Theme,
) -> Option<Vec<Vec<(Style, String)>>> {
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut rendered = Vec::new();

    let mut ends_with_newline = false;
    for line in LinesWithEndings::from(code) {
        ends_with_newline = line.ends_with('\n');
        let trimmed = line.trim_end_matches('\n');
        let ranges = highlighter.highlight_line(trimmed, &SYNTAX_SET).ok()?;

        let mut segments = Vec::new();
        for (style, part) in ranges {
            if part.is_empty() {
                continue;
            }
            segments.push((to_anstyle(style), part.to_string()));
        }
        rendered.push(segments);
    }

    // Add empty line if code ends with newline
    if ends_with_newline {
        rendered.push(Vec::new());
    }

    Some(rendered)
}

/// Select the appropriate syntax for highlighting.
///
/// Tries to find syntax by language hint (token), falling back to plain text.
fn select_syntax(language: Option<&str>) -> &'static SyntaxReference {
    language
        .and_then(|lang| SYNTAX_SET.find_syntax_by_token(lang))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
}

/// Get the global syntax set.
///
/// Useful for external operations that need direct access to syntaxes.
pub fn syntax_set() -> &'static SyntaxSet {
    &SYNTAX_SET
}

/// Find syntax by name or extension.
///
/// This is more flexible than token-based lookup and can handle
/// various ways of specifying a language.
pub fn find_syntax_by_name(name: &str) -> Option<&'static SyntaxReference> {
    // Try exact match
    if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(name) {
        return Some(syntax);
    }

    // Try as extension
    if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(name) {
        return Some(syntax);
    }

    // Try as token
    if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(name) {
        return Some(syntax);
    }

    None
}

/// Get list of all supported languages.
pub fn supported_languages() -> Vec<&'static str> {
    SYNTAX_SET
        .syntaxes()
        .iter()
        .map(|s| s.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_syntax() {
        let syntax = select_syntax(Some("rust"));
        assert_eq!(syntax.name, "Rust");
    }

    #[test]
    fn test_select_syntax_fallback() {
        let syntax = select_syntax(None);
        assert_eq!(syntax.name, "Plain Text");
    }

    #[test]
    fn test_find_syntax_by_name() {
        assert!(find_syntax_by_name("Rust").is_some());
        assert!(find_syntax_by_name("Python").is_some());
        assert!(find_syntax_by_name("nonexistent").is_none());
    }
}
