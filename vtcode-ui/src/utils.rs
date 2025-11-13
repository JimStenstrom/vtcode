//! Utility functions for vtcode-ui

use anstyle::{AnsiColor, Style};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

// Style helper functions module
pub mod style_helpers {
    use super::*;

    /// Convert a color name string to an anstyle Style
    pub fn style_from_color_name(name: &str) -> Style {
        super::style_from_color_name(name)
    }
}

// Color utility functions
pub mod colors {
    use super::*;

    /// Create a style from various inputs
    pub fn style(input: &str) -> Style {
        super::style_from_color_name(input)
    }
}

// ANSI style utilities
pub mod anstyle_utils {
    use super::*;

    /// Parse ANSI codes
    pub fn parse_ansi(s: &str) -> Style {
        CachedStyleParser::default().parse(s)
    }

    /// Convert anstyle::Style to ratatui::style::Style
    pub fn ansi_style_to_ratatui_style(ansi_style: Style) -> ratatui::style::Style {
        let mut ratatui_style = ratatui::style::Style::default();

        // Convert foreground color
        if let Some(fg_color) = ansi_style.get_fg_color() {
            ratatui_style = ratatui_style.fg(ansi_color_to_ratatui_color(fg_color));
        }

        // Convert background color
        if let Some(bg_color) = ansi_style.get_bg_color() {
            ratatui_style = ratatui_style.bg(ansi_color_to_ratatui_color(bg_color));
        }

        // Convert text effects
        let effects = ansi_style.get_effects();
        if effects.contains(anstyle::Effects::BOLD) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        if effects.contains(anstyle::Effects::UNDERLINE) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::UNDERLINED);
        }
        if effects.contains(anstyle::Effects::ITALIC) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::ITALIC);
        }

        ratatui_style
    }

    fn ansi_color_to_ratatui_color(color: anstyle::Color) -> ratatui::style::Color {
        use anstyle::AnsiColor;
        match color {
            anstyle::Color::Ansi(ansi_color) => match ansi_color {
                AnsiColor::Black => ratatui::style::Color::Black,
                AnsiColor::Red => ratatui::style::Color::Red,
                AnsiColor::Green => ratatui::style::Color::Green,
                AnsiColor::Yellow => ratatui::style::Color::Yellow,
                AnsiColor::Blue => ratatui::style::Color::Blue,
                AnsiColor::Magenta => ratatui::style::Color::Magenta,
                AnsiColor::Cyan => ratatui::style::Color::Cyan,
                AnsiColor::White => ratatui::style::Color::White,
                AnsiColor::BrightBlack => ratatui::style::Color::DarkGray,
                AnsiColor::BrightRed => ratatui::style::Color::LightRed,
                AnsiColor::BrightGreen => ratatui::style::Color::LightGreen,
                AnsiColor::BrightYellow => ratatui::style::Color::LightYellow,
                AnsiColor::BrightBlue => ratatui::style::Color::LightBlue,
                AnsiColor::BrightMagenta => ratatui::style::Color::LightMagenta,
                AnsiColor::BrightCyan => ratatui::style::Color::LightCyan,
                AnsiColor::BrightWhite => ratatui::style::Color::Gray,
            },
            anstyle::Color::Ansi256(index) => ratatui::style::Color::Indexed(index.0),
            anstyle::Color::Rgb(rgb) => ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2),
        }
    }
}

/// Convert a color name string to an anstyle Style
pub fn style_from_color_name(name: &str) -> Style {
    let color = match name.to_lowercase().as_str() {
        "black" => AnsiColor::Black.into(),
        "red" => AnsiColor::Red.into(),
        "green" => AnsiColor::Green.into(),
        "yellow" => AnsiColor::Yellow.into(),
        "blue" => AnsiColor::Blue.into(),
        "magenta" => AnsiColor::Magenta.into(),
        "cyan" => AnsiColor::Cyan.into(),
        "white" => AnsiColor::White.into(),
        "bright-black" | "bright_black" => AnsiColor::BrightBlack.into(),
        "bright-red" | "bright_red" => AnsiColor::BrightRed.into(),
        "bright-green" | "bright_green" => AnsiColor::BrightGreen.into(),
        "bright-yellow" | "bright_yellow" => AnsiColor::BrightYellow.into(),
        "bright-blue" | "bright_blue" => AnsiColor::BrightBlue.into(),
        "bright-magenta" | "bright_magenta" => AnsiColor::BrightMagenta.into(),
        "bright-cyan" | "bright_cyan" => AnsiColor::BrightCyan.into(),
        "bright-white" | "bright_white" => AnsiColor::BrightWhite.into(),
        _ => AnsiColor::White.into(), // Default to white
    };
    Style::new().fg_color(Some(color))
}

/// Cached style parser for ANSI escape sequences
#[derive(Debug)]
pub struct CachedStyleParser {
    cache: std::sync::Arc<Mutex<HashMap<String, Style>>>,
}

impl Default for CachedStyleParser {
    fn default() -> Self {
        Self {
            cache: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Clone for CachedStyleParser {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl CachedStyleParser {
    /// Create a new cached style parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an ANSI escape sequence string into a Style
    pub fn parse(&self, ansi_string: &str) -> Style {
        let mut cache = self.cache.lock();

        if let Some(style) = cache.get(ansi_string) {
            return *style;
        }

        let style = Self::parse_ansi_to_style(ansi_string);
        cache.insert(ansi_string.to_string(), style);
        style
    }

    fn parse_ansi_to_style(s: &str) -> Style {
        // Simple parser for basic ANSI codes
        let mut style = Style::new();

        // Look for SGR (Select Graphic Rendition) sequences
        if let Some(start) = s.find("\x1b[") {
            if let Some(end) = s[start..].find('m') {
                let codes = &s[start + 2..start + end];
                for code in codes.split(';') {
                    if let Ok(num) = code.parse::<u8>() {
                        match num {
                            0 => style = Style::new(), // Reset
                            1 => style = style.bold(),
                            4 => style = style.underline(),
                            30..=37 => {
                                let color = match num {
                                    30 => AnsiColor::Black,
                                    31 => AnsiColor::Red,
                                    32 => AnsiColor::Green,
                                    33 => AnsiColor::Yellow,
                                    34 => AnsiColor::Blue,
                                    35 => AnsiColor::Magenta,
                                    36 => AnsiColor::Cyan,
                                    37 => AnsiColor::White,
                                    _ => AnsiColor::White,
                                };
                                style = style.fg_color(Some(color.into()));
                            }
                            90..=97 => {
                                let color = match num {
                                    90 => AnsiColor::BrightBlack,
                                    91 => AnsiColor::BrightRed,
                                    92 => AnsiColor::BrightGreen,
                                    93 => AnsiColor::BrightYellow,
                                    94 => AnsiColor::BrightBlue,
                                    95 => AnsiColor::BrightMagenta,
                                    96 => AnsiColor::BrightCyan,
                                    97 => AnsiColor::BrightWhite,
                                    _ => AnsiColor::BrightWhite,
                                };
                                style = style.fg_color(Some(color.into()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        style
    }

    /// Parse LS_COLORS style strings
    pub fn parse_ls_colors(&self, ls_colors_string: &str) -> Style {
        // Simple implementation - treat as ANSI for now
        self.parse(ls_colors_string)
    }

    /// Parse flexible style strings (ANSI, LS_COLORS, or color names)
    pub fn parse_flexible(&self, input: &str) -> Style {
        // Try color name first
        if !input.starts_with('\x1b') {
            return style_from_color_name(input);
        }
        // Otherwise parse as ANSI
        self.parse(input)
    }

    /// Parse Git-style color configuration
    pub fn parse_git_style(&self, git_style: &str) -> Style {
        // Git style is similar to ANSI, delegate to parse
        self.parse(git_style)
    }
}

// Extension trait for Style to add color methods
pub trait StyleExt {
    fn cyan(self) -> Self;
    fn yellow(self) -> Self;
    fn green(self) -> Self;
    fn blue(self) -> Self;
    fn with_context(self, context: &str) -> Self;
}

impl StyleExt for Style {
    fn cyan(self) -> Self {
        self.fg_color(Some(AnsiColor::Cyan.into()))
    }

    fn yellow(self) -> Self {
        self.fg_color(Some(AnsiColor::Yellow.into()))
    }

    fn green(self) -> Self {
        self.fg_color(Some(AnsiColor::Green.into()))
    }

    fn blue(self) -> Self {
        self.fg_color(Some(AnsiColor::Blue.into()))
    }

    fn with_context(self, _context: &str) -> Self {
        // Context is ignored for now
        self
    }
}
