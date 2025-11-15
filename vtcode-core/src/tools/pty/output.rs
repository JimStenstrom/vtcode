//! PTY output buffering and scrollback management
//!
//! This module provides buffering for PTY output, managing both complete
//! lines and partial output, with configurable scrollback history.

use std::collections::VecDeque;

/// Scrollback buffer for PTY output
///
/// Maintains a circular buffer of output lines with separate tracking of:
/// - All lines (for snapshot/history)
/// - Pending lines (new output since last read)
/// - Partial lines (incomplete lines without newline)
pub(crate) struct PtyScrollback {
    /// All complete lines in the scrollback buffer
    lines: VecDeque<String>,
    /// Pending lines that haven't been read yet
    pending_lines: VecDeque<String>,
    /// Partial line that hasn't received a newline yet
    partial: String,
    /// Pending partial line
    pending_partial: String,
    /// Maximum number of lines to retain
    capacity: usize,
}

impl PtyScrollback {
    /// Create a new scrollback buffer with the given capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of lines to retain (minimum 1)
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            pending_lines: VecDeque::new(),
            partial: String::new(),
            pending_partial: String::new(),
            capacity: capacity.max(1),
        }
    }

    /// Push text into the scrollback buffer
    ///
    /// Text is split on newlines, with complete lines added to the buffer
    /// and partial lines accumulated until a newline is received.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to add to the buffer
    pub fn push_text(&mut self, text: &str) {
        for part in text.split_inclusive('\n') {
            self.partial.push_str(part);
            self.pending_partial.push_str(part);
            if part.ends_with('\n') {
                let complete = std::mem::take(&mut self.partial);
                let _ = std::mem::take(&mut self.pending_partial);
                self.lines.push_back(complete.clone());
                self.pending_lines.push_back(complete);
                while self.lines.len() > self.capacity {
                    self.lines.pop_front();
                }
                while self.pending_lines.len() > self.capacity {
                    self.pending_lines.pop_front();
                }
            }
        }
    }

    /// Push UTF-8 bytes into the scrollback buffer
    ///
    /// Handles partial UTF-8 sequences by accumulating bytes until a valid
    /// UTF-8 sequence can be decoded.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Mutable buffer of bytes to process
    /// * `eof` - Whether this is the end of the stream
    pub fn push_utf8(&mut self, buffer: &mut Vec<u8>, eof: bool) {
        loop {
            match std::str::from_utf8(buffer) {
                Ok(valid) => {
                    if !valid.is_empty() {
                        self.push_text(valid);
                    }
                    buffer.clear();
                    break;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    if valid_up_to > 0 {
                        if let Ok(valid) = std::str::from_utf8(&buffer[..valid_up_to]) {
                            if !valid.is_empty() {
                                self.push_text(valid);
                            }
                        }
                        buffer.drain(..valid_up_to);
                        continue;
                    }

                    if let Some(error_len) = error.error_len() {
                        // Invalid UTF-8 sequence - replace with replacement character
                        self.push_text("\u{FFFD}");
                        buffer.drain(..error_len);
                        continue;
                    }

                    // Incomplete UTF-8 sequence at end
                    if eof && !buffer.is_empty() {
                        self.push_text("\u{FFFD}");
                        buffer.clear();
                    }

                    break;
                }
            }
        }
    }

    /// Get a snapshot of all buffered output
    ///
    /// Returns all complete lines plus any partial line.
    pub fn snapshot(&self) -> String {
        let mut output = String::new();
        for line in &self.lines {
            output.push_str(line);
        }
        output.push_str(&self.partial);
        output
    }

    /// Get pending output without draining it
    ///
    /// Returns pending lines plus any pending partial line.
    pub fn pending(&self) -> String {
        let mut output = String::new();
        for line in &self.pending_lines {
            output.push_str(line);
        }
        output.push_str(&self.pending_partial);
        output
    }

    /// Take pending output, clearing the pending buffers
    ///
    /// Returns all pending lines and partial line, then clears the
    /// pending state. The output remains in the main scrollback buffer.
    pub fn take_pending(&mut self) -> String {
        let mut output = String::new();
        while let Some(line) = self.pending_lines.pop_front() {
            output.push_str(&line);
        }
        if !self.pending_partial.is_empty() {
            output.push_str(&self.pending_partial);
            self.pending_partial.clear();
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollback_basic() {
        let mut sb = PtyScrollback::new(10);
        sb.push_text("line 1\n");
        sb.push_text("line 2\n");

        let snapshot = sb.snapshot();
        assert!(snapshot.contains("line 1"));
        assert!(snapshot.contains("line 2"));
    }

    #[test]
    fn test_scrollback_partial() {
        let mut sb = PtyScrollback::new(10);
        sb.push_text("partial");

        let snapshot = sb.snapshot();
        assert_eq!(snapshot, "partial");
    }

    #[test]
    fn test_scrollback_capacity() {
        let mut sb = PtyScrollback::new(2);
        sb.push_text("line 1\n");
        sb.push_text("line 2\n");
        sb.push_text("line 3\n");

        let snapshot = sb.snapshot();
        assert!(!snapshot.contains("line 1"));
        assert!(snapshot.contains("line 2"));
        assert!(snapshot.contains("line 3"));
    }

    #[test]
    fn test_pending_take() {
        let mut sb = PtyScrollback::new(10);
        sb.push_text("line 1\n");

        let pending1 = sb.take_pending();
        assert_eq!(pending1, "line 1\n");

        let pending2 = sb.take_pending();
        assert_eq!(pending2, "");
    }
}
