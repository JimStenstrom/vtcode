//! Command echo filtering using KMP pattern matching
//!
//! When commands are sent to a PTY session, they are often echoed back
//! in the output. This module provides filtering to remove the echoed
//! command, leaving only the actual command output.
//!
//! The implementation uses the Knuth-Morris-Pratt (KMP) string matching
//! algorithm for efficient pattern matching, with special handling for:
//! - ANSI escape sequences
//! - Zero-width spaces
//! - Carriage returns that reset the match
//! - Newline requirements

use super::ansi::parse_ansi_sequence;

/// State machine for filtering command echoes from PTY output
///
/// Uses the KMP algorithm to efficiently match and filter the echoed
/// command text from the output stream.
pub(crate) struct CommandEchoState {
    /// The command bytes we're looking for
    command_bytes: Vec<u8>,
    /// KMP failure function for pattern matching
    failure: Vec<usize>,
    /// Current number of matched bytes
    matched: usize,
    /// Whether we require a newline after the command
    require_newline: bool,
    /// Whether we're still waiting for the newline
    pending_newline: bool,
    /// Whether we've consumed the echo at least once
    consumed_once: bool,
}

impl CommandEchoState {
    /// Create a new command echo filter
    ///
    /// # Arguments
    ///
    /// * `command` - The command text to filter from output
    /// * `expect_newline` - Whether to expect a newline after the command
    ///
    /// # Returns
    ///
    /// Returns `None` if the command is empty after trimming
    pub fn new(command: &str, expect_newline: bool) -> Option<Self> {
        let trimmed = command.trim_matches(|ch| ch == '\n' || ch == '\r');
        if trimmed.is_empty() {
            return None;
        }

        let command_bytes = trimmed.as_bytes().to_vec();
        if command_bytes.is_empty() {
            return None;
        }

        let failure = build_failure(&command_bytes);

        Some(Self {
            command_bytes,
            failure,
            matched: 0,
            require_newline: expect_newline,
            pending_newline: expect_newline,
            consumed_once: false,
        })
    }

    /// Reset the matcher state
    fn reset(&mut self) {
        self.matched = 0;
        self.pending_newline = self.require_newline;
    }

    /// Consume a chunk of output text, returning how many bytes were consumed
    /// and whether the echo filtering is complete
    ///
    /// # Arguments
    ///
    /// * `text` - The output text to process
    ///
    /// # Returns
    ///
    /// A tuple of (bytes_consumed, is_complete)
    pub fn consume_chunk(&mut self, text: &str) -> (usize, bool) {
        let mut index = 0usize;
        let bytes = text.as_bytes();
        const ZERO_WIDTH_SPACE: &[u8] = "\u{200B}".as_bytes();

        while index < bytes.len() {
            let slice = &text[index..];

            // Skip ANSI escape sequences
            if let Some(len) = parse_ansi_sequence(slice) {
                index += len;
                continue;
            }

            // Skip zero-width spaces
            if slice.as_bytes().starts_with(ZERO_WIDTH_SPACE) {
                index += ZERO_WIDTH_SPACE.len();
                continue;
            }

            let byte = bytes[index];

            // Carriage return resets the match
            if byte == b'\r' {
                index += 1;
                self.reset();
                continue;
            }

            // Handle pending newline
            if self.pending_newline {
                if byte == b'\n' {
                    index += 1;
                    self.pending_newline = false;
                    continue;
                }
                self.pending_newline = false;
            }

            // KMP pattern matching
            let mut matched_byte = false;
            loop {
                if let Some(&expected) = self.command_bytes.get(self.matched) {
                    if byte == expected {
                        self.matched += 1;
                        index += 1;
                        if self.matched == self.command_bytes.len() {
                            self.consumed_once = true;
                            self.pending_newline = self.require_newline;
                            self.matched = if self.command_bytes.len() > 1 {
                                self.failure[self.matched - 1]
                            } else {
                                0
                            };
                        }
                        matched_byte = true;
                        break;
                    }
                }

                if self.matched == 0 {
                    break;
                }

                self.matched = self.failure[self.matched - 1];
            }

            if matched_byte {
                continue;
            }

            break;
        }

        let done = self.consumed_once && !self.pending_newline && self.matched == 0;
        (index, done)
    }
}

/// Build KMP failure function for pattern matching
///
/// The failure function is used by the KMP algorithm to efficiently
/// skip positions in the pattern when a mismatch occurs.
///
/// # Arguments
///
/// * `pattern` - The byte pattern to build the failure function for
///
/// # Returns
///
/// A vector where `failure[i]` indicates the length of the longest
/// proper prefix of `pattern[0..=i]` which is also a suffix
fn build_failure(pattern: &[u8]) -> Vec<usize> {
    let mut failure = vec![0usize; pattern.len()];
    let mut length = 0usize;
    let mut index = 1usize;

    while index < pattern.len() {
        if pattern[index] == pattern[length] {
            length += 1;
            failure[index] = length;
            index += 1;
        } else if length != 0 {
            length = failure[length - 1];
        } else {
            failure[index] = 0;
            index += 1;
        }
    }

    failure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_failure() {
        // Pattern: "ABABC"
        let pattern = b"ABABC";
        let failure = build_failure(pattern);
        assert_eq!(failure, vec![0, 0, 1, 2, 0]);
    }

    #[test]
    fn test_echo_filter_simple() {
        let mut filter = CommandEchoState::new("ls -la", true).unwrap();
        let (consumed, done) = filter.consume_chunk("ls -la\n");
        assert_eq!(consumed, 7);
        assert!(done);
    }

    #[test]
    fn test_echo_filter_with_ansi() {
        let mut filter = CommandEchoState::new("ls", true).unwrap();
        let text = "\x1b[31mls\x1b[0m\n";
        let (consumed, done) = filter.consume_chunk(text);
        assert!(done);
        assert!(consumed > 0);
    }
}
