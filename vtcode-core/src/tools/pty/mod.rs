//! PTY (Pseudo-Terminal) management
//!
//! This module provides functionality for managing PTY sessions, which allow
//! running commands in an interactive terminal environment. Key features:
//!
//! - Session lifecycle management (create, read, write, close)
//! - VT100 terminal emulation for screen state
//! - Command echo filtering using KMP algorithm
//! - Output scrollback buffering
//! - Sandbox integration for secure command execution
//!
//! # Module Organization
//!
//! - `ansi` - ANSI escape sequence parsing
//! - `echo_filter` - Command echo filtering with KMP
//! - `io` - I/O utilities and helpers
//! - `output` - Output buffering and scrollback
//! - `session` - Session handle and state
//! - `types` - Public types for requests/responses

mod ansi;
mod echo_filter;
mod io;
mod output;
mod session;
pub mod types;

// Re-exports
pub use io::is_development_toolchain_command;
pub use types::{PtyCommandRequest, PtyCommandResult};

use crate::audit::PermissionAuditLog;
use crate::config::{CommandsConfig, PtyConfig};
use crate::sandbox::SandboxProfile;
use crate::tools::path_env;
use crate::tools::types::VTCodePtySession;
use anyhow::{Context, Result, anyhow};
use echo_filter::CommandEchoState;
use output::PtyScrollback;
use parking_lot::Mutex;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use session::PtySessionHandle;
use shell_words::join;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, warn};
use vt100::Parser;

/// PTY manager for creating and managing multiple PTY sessions
#[derive(Clone)]
pub struct PtyManager {
    workspace_root: PathBuf,
    config: PtyConfig,
    inner: Arc<PtyState>,
    sandbox_profile: Arc<Mutex<Option<SandboxProfile>>>,
    audit_log: Option<Arc<TokioMutex<PermissionAuditLog>>>,
    extra_paths: Arc<Mutex<Vec<PathBuf>>>,
}

/// Internal state for PTY manager
#[derive(Default)]
struct PtyState {
    sessions: Mutex<HashMap<String, Arc<PtySessionHandle>>>,
}

impl PtyManager {
    /// Create a new PTY manager
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - The workspace root directory
    /// * `config` - PTY configuration
    pub fn new(workspace_root: PathBuf, config: PtyConfig) -> Self {
        let resolved_root = workspace_root
            .canonicalize()
            .unwrap_or(workspace_root.clone());

        let default_paths = path_env::compute_extra_search_paths(
            &CommandsConfig::default().extra_path_entries,
            &resolved_root,
        );

        Self {
            workspace_root: resolved_root,
            config,
            inner: Arc::new(PtyState::default()),
            sandbox_profile: Arc::new(Mutex::new(None)),
            audit_log: None,
            extra_paths: Arc::new(Mutex::new(default_paths)),
        }
    }

    /// Add audit logging to the PTY manager
    pub fn with_audit_log(mut self, audit_log: Arc<TokioMutex<PermissionAuditLog>>) -> Self {
        self.audit_log = Some(audit_log);
        self
    }

    /// Get the current PTY configuration
    pub fn config(&self) -> &PtyConfig {
        &self.config
    }

    /// Set the sandbox profile for command execution
    pub fn set_sandbox_profile(&self, profile: Option<SandboxProfile>) {
        let mut slot = self.sandbox_profile.lock();
        *slot = profile;
    }

    /// Get the current sandbox profile
    pub fn sandbox_profile(&self) -> Option<SandboxProfile> {
        self.current_sandbox_profile()
    }

    /// Apply commands configuration
    pub fn apply_commands_config(&self, commands_config: &CommandsConfig) {
        let mut extra = self.extra_paths.lock();
        *extra = path_env::compute_extra_search_paths(
            &commands_config.extra_path_entries,
            &self.workspace_root,
        );
    }

    fn current_sandbox_profile(&self) -> Option<SandboxProfile> {
        self.sandbox_profile.lock().clone()
    }

    /// Format a working directory relative to the workspace
    pub fn describe_working_dir(&self, path: &Path) -> String {
        self.format_working_dir(path)
    }

    /// Run a single command in a PTY and wait for completion
    ///
    /// This creates a temporary PTY, runs the command, and returns the result.
    ///
    /// # Arguments
    ///
    /// * `request` - The command request with command, working directory, timeout, and size
    ///
    /// # Returns
    ///
    /// The command result including exit code, output, and duration
    pub async fn run_command(&self, request: PtyCommandRequest) -> Result<PtyCommandResult> {
        if request.command.is_empty() {
            return Err(anyhow!("PTY command cannot be empty"));
        }

        let mut command = request.command.clone();
        let program = command.remove(0);
        let args = command;
        let timeout = io::clamp_timeout(request.timeout);
        let work_dir = request.working_dir.clone();
        let size = request.size;
        let start = Instant::now();
        self.ensure_within_workspace(&work_dir)?;
        let workspace_root = self.workspace_root.clone();
        let extra_paths = self.extra_paths.lock().clone();

        let sandbox_profile = self.current_sandbox_profile();
        let result = tokio::task::spawn_blocking(move || -> Result<PtyCommandResult> {
            let timeout_duration = std::time::Duration::from_millis(timeout);

            // Try to resolve the program path. If not found, wrap in shell.
            let (exec_program, exec_args, display_program, env_profile, _use_shell_wrapper) =
                if let Some(profile) = sandbox_profile.clone() {
                    let command_string =
                        join(std::iter::once(program.clone()).chain(args.iter().cloned()));
                    (
                        profile.binary().display().to_string(),
                        vec![
                            "--settings".to_string(),
                            profile.settings().display().to_string(),
                            command_string,
                        ],
                        program.clone(),
                        Some(profile),
                        false,
                    )
                } else if let Some(resolved_path) =
                    path_env::resolve_program_path(&program, &extra_paths)
                {
                    // Program found in PATH, use resolved executable directly
                    (resolved_path, args.clone(), program.clone(), None, false)
                } else {
                    // Program not found in PATH, wrap in shell to leverage user's PATH
                    let shell = "/bin/sh";
                    let full_command =
                        join(std::iter::once(program.clone()).chain(args.iter().cloned()));
                    (
                        shell.to_string(),
                        vec!["-c".to_string(), full_command.clone()],
                        program.clone(),
                        None,
                        true,
                    )
                };
            let mut builder = CommandBuilder::new(exec_program.clone());
            for arg in &exec_args {
                builder.arg(arg);
            }
            builder.cwd(&work_dir);
            io::set_command_environment(
                &mut builder,
                &display_program,
                size,
                &workspace_root,
                env_profile.as_ref(),
                &extra_paths,
            );

            let pty_system = native_pty_system();
            let pair = pty_system
                .openpty(size)
                .context("failed to allocate PTY pair")?;

            let mut child = pair
                .slave
                .spawn_command(builder)
                .with_context(|| format!("failed to spawn PTY command '{display_program}'"))?;
            let mut killer = child.clone_killer();
            drop(pair.slave);

            let reader = pair
                .master
                .try_clone_reader()
                .context("failed to clone PTY reader")?;

            let (wait_tx, wait_rx) = mpsc::channel();
            let wait_thread = thread::spawn(move || {
                let status = child.wait();
                let _ = wait_tx.send(());
                status
            });

            let reader_thread = thread::spawn(move || -> Result<Vec<u8>> {
                let mut reader = reader;
                let mut buffer = [0u8; 4096];
                let mut collected = Vec::new();

                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            collected.extend_from_slice(&buffer[..bytes_read]);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(error) => {
                            return Err(error).context("failed to read PTY command output");
                        }
                    }
                }

                Ok(collected)
            });

            let wait_result = match wait_rx.recv_timeout(timeout_duration) {
                Ok(()) => wait_thread
                    .join()
                    .map_err(|panic| anyhow!("PTY command wait thread panicked: {:?}", panic))?,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    killer
                        .kill()
                        .context("failed to terminate PTY command after timeout")?;

                    let join_result = wait_thread.join().map_err(|panic| {
                        anyhow!("PTY command wait thread panicked: {:?}", panic)
                    })?;
                    if let Err(error) = join_result {
                        return Err(error)
                            .context("failed to wait for PTY command to exit after timeout");
                    }

                    reader_thread
                        .join()
                        .map_err(|panic| {
                            anyhow!("PTY command reader thread panicked: {:?}", panic)
                        })?
                        .context("failed to read PTY command output")?;

                    return Err(anyhow!(
                        "PTY command timed out after {} milliseconds",
                        timeout
                    ));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let join_result = wait_thread.join().map_err(|panic| {
                        anyhow!("PTY command wait thread panicked: {:?}", panic)
                    })?;
                    if let Err(error) = join_result {
                        return Err(error).context(
                            "failed to wait for PTY command after wait channel disconnected",
                        );
                    }

                    reader_thread
                        .join()
                        .map_err(|panic| {
                            anyhow!("PTY command reader thread panicked: {:?}", panic)
                        })?
                        .context("failed to read PTY command output")?;

                    return Err(anyhow!(
                        "PTY command wait channel disconnected unexpectedly"
                    ));
                }
            };

            let status = wait_result.context("failed to wait for PTY command to exit")?;

            let output_bytes = reader_thread
                .join()
                .map_err(|panic| anyhow!("PTY command reader thread panicked: {:?}", panic))?
                .context("failed to read PTY command output")?;
            let output = String::from_utf8_lossy(&output_bytes).to_string();
            let exit_code = io::exit_status_code(status);

            Ok(PtyCommandResult {
                exit_code,
                output,
                duration: start.elapsed(),
                size,
            })
        })
        .await
        .context("failed to join PTY command task")??;

        Ok(result)
    }

    /// Resolve a working directory path
    ///
    /// # Arguments
    ///
    /// * `requested` - Optional working directory (relative to workspace)
    ///
    /// # Returns
    ///
    /// The absolute path to the working directory
    pub async fn resolve_working_dir(&self, requested: Option<&str>) -> Result<PathBuf> {
        let requested = match requested {
            Some(dir) if !dir.trim().is_empty() => dir,
            _ => return Ok(self.workspace_root.clone()),
        };

        let candidate = self.workspace_root.join(requested);
        let normalized = io::normalize_path(&candidate);
        if !normalized.starts_with(&self.workspace_root) {
            return Err(anyhow!(
                "Working directory '{}' escapes the workspace root",
                candidate.display()
            ));
        }
        let metadata = tokio::fs::metadata(&normalized).await.with_context(|| {
            format!(
                "Working directory '{}' does not exist",
                normalized.display()
            )
        })?;
        if !metadata.is_dir() {
            return Err(anyhow!(
                "Working directory '{}' is not a directory",
                normalized.display()
            ));
        }
        Ok(normalized)
    }

    /// Create a new PTY session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the session
    /// * `command` - Command and arguments to execute
    /// * `working_dir` - Working directory for the session
    /// * `size` - PTY size (rows and columns)
    ///
    /// # Returns
    ///
    /// Session metadata
    pub fn create_session(
        &self,
        session_id: String,
        command: Vec<String>,
        working_dir: PathBuf,
        size: PtySize,
    ) -> Result<VTCodePtySession> {
        if command.is_empty() {
            return Err(anyhow!("PTY session command cannot be empty"));
        }

        let mut sessions = self.inner.sessions.lock();
        if sessions.contains_key(&session_id) {
            return Err(anyhow!("PTY session '{}' already exists", session_id));
        }

        let mut command_parts = command.clone();
        let program = command_parts.remove(0);
        let args = command_parts;
        let sandbox_profile = self.current_sandbox_profile();
        let extra_paths = self.extra_paths.lock().clone();

        let (exec_program, exec_args, display_program, env_profile) = if let Some(profile) =
            sandbox_profile.clone()
        {
            let command_string = join(std::iter::once(program.clone()).chain(args.iter().cloned()));
            (
                profile.binary().display().to_string(),
                vec![
                    "--settings".to_string(),
                    profile.settings().display().to_string(),
                    command_string,
                ],
                program.clone(),
                Some(profile),
            )
        } else if let Some(resolved_path) = path_env::resolve_program_path(&program, &extra_paths) {
            // Program found in PATH, use it directly
            (resolved_path, args.clone(), program.clone(), None)
        } else {
            // Program not found in PATH, wrap in shell to leverage user's PATH
            let shell = "/bin/sh";
            let full_command = join(std::iter::once(program.clone()).chain(args.iter().cloned()));
            (
                shell.to_string(),
                vec!["-c".to_string(), full_command.clone()],
                program.clone(),
                None,
            )
        };

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(size)
            .context("failed to allocate PTY pair")?;

        let mut builder = CommandBuilder::new(exec_program.clone());
        for arg in &exec_args {
            builder.arg(arg);
        }
        builder.cwd(&working_dir);
        self.ensure_within_workspace(&working_dir)?;
        io::set_command_environment(
            &mut builder,
            &display_program,
            size,
            &self.workspace_root,
            env_profile.as_ref(),
            &extra_paths,
        );

        let child = pair.slave.spawn_command(builder).with_context(|| {
            format!("failed to spawn PTY session command '{}'", display_program)
        })?;
        drop(pair.slave);

        let master = pair.master;
        let mut reader = master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let writer = master.take_writer().context("failed to take PTY writer")?;

        let parser = Arc::new(Mutex::new(Parser::new(
            size.rows,
            size.cols,
            self.config.scrollback_lines,
        )));
        let scrollback = Arc::new(Mutex::new(PtyScrollback::new(self.config.scrollback_lines)));
        let parser_clone = Arc::clone(&parser);
        let scrollback_clone = Arc::clone(&scrollback);
        let session_name = session_id.clone();
        let reader_thread = thread::Builder::new()
            .name(format!("vtcode-pty-reader-{session_name}"))
            .spawn(move || {
                let mut buffer = [0u8; 4096];
                let mut utf8_buffer: Vec<u8> = Vec::new();
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            if !utf8_buffer.is_empty() {
                                let mut scrollback = scrollback_clone.lock();
                                scrollback.push_utf8(&mut utf8_buffer, true);
                            }
                            debug!("PTY session '{}' reader reached EOF", session_name);
                            break;
                        }
                        Ok(bytes_read) => {
                            let chunk = &buffer[..bytes_read];
                            {
                                let mut parser = parser_clone.lock();
                                parser.process(chunk);
                            }

                            utf8_buffer.extend_from_slice(chunk);
                            {
                                let mut scrollback = scrollback_clone.lock();
                                scrollback.push_utf8(&mut utf8_buffer, false);
                            }
                        }
                        Err(error) => {
                            warn!("PTY session '{}' reader error: {}", session_name, error);
                            break;
                        }
                    }
                }
            })
            .context("failed to spawn PTY reader thread")?;

        let metadata = VTCodePtySession {
            id: session_id.clone(),
            command: program,
            args,
            working_dir: Some(self.format_working_dir(&working_dir)),
            rows: size.rows,
            cols: size.cols,
            screen_contents: None,
            scrollback: None,
        };

        sessions.insert(
            session_id.clone(),
            Arc::new(PtySessionHandle {
                master: Mutex::new(master),
                child: Mutex::new(child),
                writer: Mutex::new(Some(writer)),
                terminal: parser,
                scrollback,
                reader_thread: Mutex::new(Some(reader_thread)),
                metadata: metadata.clone(),
                last_input: Mutex::new(None),
            }),
        );

        Ok(metadata)
    }

    /// List all active PTY sessions
    pub fn list_sessions(&self) -> Vec<VTCodePtySession> {
        let sessions = self.inner.sessions.lock();
        sessions
            .values()
            .map(|handle| handle.snapshot_metadata())
            .collect()
    }

    /// Get a snapshot of a specific session
    pub fn snapshot_session(&self, session_id: &str) -> Result<VTCodePtySession> {
        let handle = self.session_handle(session_id)?;
        Ok(handle.snapshot_metadata())
    }

    /// Read output from a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session identifier
    /// * `drain` - If true, clear the output buffer after reading
    pub fn read_session_output(&self, session_id: &str, drain: bool) -> Result<Option<String>> {
        let handle = self.session_handle(session_id)?;
        Ok(handle.read_output(drain))
    }

    /// Send input to a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session identifier
    /// * `data` - The data to send
    /// * `append_newline` - Whether to append a newline after the data
    ///
    /// # Returns
    ///
    /// The number of bytes written
    pub fn send_input_to_session(
        &self,
        session_id: &str,
        data: &[u8],
        append_newline: bool,
    ) -> Result<usize> {
        let handle = self.session_handle(session_id)?;

        if let Ok(input_text) = std::str::from_utf8(data) {
            let mut last_input = handle.last_input.lock();
            *last_input = CommandEchoState::new(input_text, append_newline);
        } else {
            let mut last_input = handle.last_input.lock();
            *last_input = None;
        }

        let mut writer_guard = handle.writer.lock();
        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| anyhow!("PTY session '{}' is no longer writable", session_id))?;

        writer
            .write_all(data)
            .context("failed to write input to PTY session")?;
        let mut written = data.len();
        if append_newline {
            writer
                .write_all(b"\n")
                .context("failed to write newline to PTY session")?;
            written += 1;
        }
        writer
            .flush()
            .context("failed to flush PTY session input")?;

        Ok(written)
    }

    /// Resize a PTY session
    pub fn resize_session(&self, session_id: &str, size: PtySize) -> Result<VTCodePtySession> {
        let handle = self.session_handle(session_id)?;
        {
            let master = handle.master.lock();
            master
                .resize(size)
                .context("failed to resize PTY session")?;
        }
        let mut parser = handle.terminal.lock();
        parser.set_size(size.rows, size.cols);
        Ok(handle.snapshot_metadata())
    }

    /// Check if a session has completed
    ///
    /// # Returns
    ///
    /// - `Ok(Some(exit_code))` if the session has completed
    /// - `Ok(None)` if the session is still running
    pub fn is_session_completed(&self, session_id: &str) -> Result<Option<i32>> {
        let handle = self.session_handle(session_id)?;
        let mut child = handle.child.lock();
        Ok(
            if let Some(status) = child
                .try_wait()
                .context("failed to poll PTY session status")?
            {
                Some(io::exit_status_code(status))
            } else {
                None
            },
        )
    }

    /// Close a PTY session
    ///
    /// Sends an exit command, terminates the child process, and cleans up resources.
    pub fn close_session(&self, session_id: &str) -> Result<VTCodePtySession> {
        let handle = {
            let mut sessions = self.inner.sessions.lock();
            sessions
                .remove(session_id)
                .ok_or_else(|| anyhow!("PTY session '{}' not found", session_id))?
        };

        {
            let mut writer_guard = handle.writer.lock();
            if let Some(mut writer) = writer_guard.take() {
                let _ = writer.write_all(b"exit\n");
                let _ = writer.flush();
            }
        }

        let mut child = handle.child.lock();
        if child
            .try_wait()
            .context("failed to poll PTY session status")?
            .is_none()
        {
            child.kill().context("failed to terminate PTY session")?;
            let _ = child.wait();
        }

        {
            let mut thread_guard = handle.reader_thread.lock();
            if let Some(reader_thread) = thread_guard.take() {
                if let Err(panic) = reader_thread.join() {
                    warn!(
                        "PTY session '{}' reader thread panicked: {:?}",
                        session_id, panic
                    );
                }
            }
        }

        Ok(handle.snapshot_metadata())
    }

    fn format_working_dir(&self, path: &Path) -> String {
        match path.strip_prefix(&self.workspace_root) {
            Ok(relative) if relative.as_os_str().is_empty() => ".".to_string(),
            Ok(relative) => relative.to_string_lossy().replace("\\", "/"),
            Err(_) => path.to_string_lossy().to_string(),
        }
    }

    fn session_handle(&self, session_id: &str) -> Result<Arc<PtySessionHandle>> {
        let sessions = self.inner.sessions.lock();
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("PTY session '{}' not found", session_id))
    }

    fn ensure_within_workspace(&self, candidate: &Path) -> Result<()> {
        io::ensure_within_workspace(candidate, &self.workspace_root)
    }
}
