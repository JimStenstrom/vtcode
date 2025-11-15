//! ANSI style generation helpers for markdown elements.
//!
//! This module provides styling functions for different markdown elements
//! like headings, code blocks, inline code, etc.

use super::parser::HeadingLevel;
use crate::theme::ThemeStyles;
use anstyle::Style;

/// Get style for inline code elements.
///
/// Inline code uses secondary color with bold and background.
pub fn inline_code_style(theme_styles: &ThemeStyles, base_style: Style) -> Style {
    let fg = theme_styles
        .secondary
        .get_fg_color()
        .or_else(|| base_style.get_fg_color());
    let bg = Some(theme_styles.background.into());

    let mut style = base_style;
    if let Some(fg_color) = fg {
        style = style.fg_color(Some(fg_color));
    }
    style.bg_color(bg).bold()
}

/// Get style for code blocks.
///
/// Code blocks use output color to differentiate from inline code.
pub fn code_block_style(theme_styles: &ThemeStyles, base_style: Style) -> Style {
    let fg = theme_styles
        .output
        .get_fg_color()
        .or_else(|| base_style.get_fg_color());

    let mut style = base_style;
    if let Some(color) = fg {
        style = style.fg_color(Some(color));
    }
    style
}

/// Get style for headings based on level.
///
/// Different heading levels get different styling:
/// - H1: Primary color, bold, underlined
/// - H2: Primary color, bold
/// - H3: Secondary color, bold
/// - H4-H6: Base style, bold
pub fn heading_style(level: HeadingLevel, theme_styles: &ThemeStyles, base_style: Style) -> Style {
    match level {
        HeadingLevel::H1 => theme_styles.primary.bold().underline(),
        HeadingLevel::H2 => theme_styles.primary.bold(),
        HeadingLevel::H3 => theme_styles.secondary.bold(),
        _ => base_style.bold(),
    }
}

/// Get style for block quotes.
///
/// Block quotes use secondary color with italic.
pub fn blockquote_style(theme_styles: &ThemeStyles) -> Style {
    theme_styles.secondary.italic()
}

/// Get style for horizontal rules.
///
/// Rules use secondary color with bold.
pub fn rule_style(theme_styles: &ThemeStyles) -> Style {
    theme_styles.secondary.bold()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_styles() {
        let theme_styles = crate::theme::active_styles();
        let base = Style::new();

        // Just verify they don't panic
        heading_style(HeadingLevel::H1, &theme_styles, base);
        heading_style(HeadingLevel::H2, &theme_styles, base);
        heading_style(HeadingLevel::H3, &theme_styles, base);
        heading_style(HeadingLevel::H4, &theme_styles, base);
    }

    #[test]
    fn test_code_styles() {
        let theme_styles = crate::theme::active_styles();
        let base = Style::new();

        inline_code_style(&theme_styles, base);
        code_block_style(&theme_styles, base);
    }
}
