//! PTY session handle and lifecycle management
//!
//! This module manages individual PTY sessions, including:
//! - Terminal state (VT100 emulation)
//! - Output scrollback buffering
//! - Command echo filtering
//! - Reader thread coordination

use super::echo_filter::CommandEchoState;
use super::output::PtyScrollback;
use crate::tools::types::VTCodePtySession;
use parking_lot::Mutex;
use portable_pty::{Child, MasterPty};
use std::io::Write;
use std::sync::Arc;
use std::thread::JoinHandle;
use vt100::Parser;

/// Handle for an active PTY session
///
/// This structure manages the state and I/O for a single PTY session,
/// including the master PTY, child process, terminal emulation, and
/// output buffering.
pub(crate) struct PtySessionHandle {
    /// Master side of the PTY pair
    pub(super) master: Mutex<Box<dyn MasterPty + Send>>,
    /// Child process running in the PTY
    pub(super) child: Mutex<Box<dyn Child + Send>>,
    /// Writer for sending input to the PTY
    pub(super) writer: Mutex<Option<Box<dyn Write + Send>>>,
    /// VT100 terminal emulator for screen state
    pub(super) terminal: Arc<Mutex<Parser>>,
    /// Scrollback buffer for output history
    pub(super) scrollback: Arc<Mutex<PtyScrollback>>,
    /// Reader thread handle
    pub(super) reader_thread: Mutex<Option<JoinHandle<()>>>,
    /// Session metadata
    pub(super) metadata: VTCodePtySession,
    /// Current command echo filter state
    pub(super) last_input: Mutex<Option<CommandEchoState>>,
}

impl PtySessionHandle {
    /// Create a snapshot of the current session metadata
    ///
    /// Captures the current PTY size, screen contents, and scrollback.
    pub fn snapshot_metadata(&self) -> VTCodePtySession {
        let mut metadata = self.metadata.clone();
        {
            let master = self.master.lock();
            if let Ok(size) = master.get_size() {
                metadata.rows = size.rows;
                metadata.cols = size.cols;
            }
        }
        {
            let parser = self.terminal.lock();
            let contents = parser.screen().contents();
            metadata.screen_contents = Some(contents);
        }
        {
            let scrollback = self.scrollback.lock();
            let contents = scrollback.snapshot();
            if !contents.is_empty() {
                metadata.scrollback = Some(contents);
            }
        }
        metadata
    }

    /// Read output from the session
    ///
    /// # Arguments
    ///
    /// * `drain` - If true, clears the pending output buffer after reading
    ///
    /// # Returns
    ///
    /// The output text, or None if no output is available
    pub fn read_output(&self, drain: bool) -> Option<String> {
        let mut scrollback = self.scrollback.lock();
        let text = if drain {
            scrollback.take_pending()
        } else {
            scrollback.pending()
        };
        if text.is_empty() {
            return None;
        }

        let filtered = self.strip_command_echo(text);
        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }

    /// Strip command echo from output text
    ///
    /// Uses the current echo filter state to remove the echoed command
    /// from the output, leaving only the actual command result.
    ///
    /// # Arguments
    ///
    /// * `text` - The output text to filter
    ///
    /// # Returns
    ///
    /// The filtered text with command echo removed
    fn strip_command_echo(&self, text: String) -> String {
        let mut guard = self.last_input.lock();
        let Some(state) = guard.as_mut() else {
            return text;
        };

        let (consumed, done) = state.consume_chunk(&text);
        if done {
            *guard = None;
        }

        text.get(consumed..)
            .map(|tail| tail.to_string())
            .unwrap_or_default()
    }
}
