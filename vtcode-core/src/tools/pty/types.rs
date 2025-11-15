//! PTY types and request/response structures

use portable_pty::PtySize;
use std::path::PathBuf;
use std::time::Duration;

/// Request structure for running a PTY command
pub struct PtyCommandRequest {
    pub command: Vec<String>,
    pub working_dir: PathBuf,
    pub timeout: Duration,
    pub size: PtySize,
}

/// Result structure for a completed PTY command
pub struct PtyCommandResult {
    pub exit_code: i32,
    pub output: String,
    pub duration: Duration,
    pub size: PtySize,
}
