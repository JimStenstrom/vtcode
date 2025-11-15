//! ANSI escape sequence parsing
//!
//! This module provides functionality for parsing ANSI escape sequences
//! commonly found in terminal output.

/// Parse ANSI escape sequence from the beginning of text
///
/// Returns the length of the ANSI sequence if found, or None if the text
/// doesn't start with an ANSI sequence.
///
/// # Examples
///
/// ```ignore
/// use vtcode_core::tools::pty::ansi::parse_ansi_sequence;
///
/// assert_eq!(parse_ansi_sequence("\x1b[31mRed"), Some(5));
/// assert_eq!(parse_ansi_sequence("normal text"), None);
/// ```
pub fn parse_ansi_sequence(text: &str) -> Option<usize> {
    crate::utils::ansi_parser::parse_ansi_sequence(text)
}
