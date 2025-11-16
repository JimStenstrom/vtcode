//! PTY session management module.
//!
//! This module handles persistent PTY (pseudo-terminal) sessions that allow
//! for interactive command execution, input/output streaming, and session lifecycle management.

use crate::config::PtyConfig;
use crate::tools::types::VTCodePtySession;
use crate::tools::PtyManager;
use anyhow::{Context, Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use futures::future::BoxFuture;
use portable_pty::PtySize;
use serde_json::{Map, Value, json};
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, trace};

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn create_pty_session_executor(
        &mut self,
        args: Value,
    ) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_create_pty_session(args).await })
    }

    pub(in crate::tools::registry) fn list_pty_sessions_executor(
        &mut self,
        _args: Value,
    ) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_list_pty_sessions().await })
    }

    pub(in crate::tools::registry) fn close_pty_session_executor(
        &mut self,
        args: Value,
    ) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_close_pty_session(args).await })
    }

    pub(in crate::tools::registry) fn send_pty_input_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_send_pty_input(args).await })
    }

    pub(in crate::tools::registry) fn read_pty_session_executor(
        &mut self,
        args: Value,
    ) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_read_pty_session(args).await })
    }

    pub(in crate::tools::registry) fn resize_pty_session_executor(
        &mut self,
        args: Value,
    ) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_resize_pty_session(args).await })
    }

    async fn execute_create_pty_session(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "create_pty_session expects an object payload")?;
        let session_id =
            parse_session_id(payload, "create_pty_session requires a 'session_id' string")?;

        let mut command_parts = parse_command_parts(
            payload,
            "create_pty_session requires a 'command' value",
            "PTY session command cannot be empty",
        )?;

        let login_shell = payload
            .get("login")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        let shell_program = resolve_shell_preference(
            payload.get("shell").and_then(|value| value.as_str()),
            self.pty_config(),
        );
        let should_replace = payload.get("shell").is_some()
            || (command_parts.len() == 1 && is_default_shell_placeholder(&command_parts[0]));
        if should_replace {
            command_parts = vec![shell_program];
        }

        if login_shell
            && !command_parts.is_empty()
            && !should_use_windows_command_tokenizer(Some(&command_parts[0]))
            && !command_parts.iter().skip(1).any(|arg| arg == "-l")
        {
            command_parts.push("-l".to_string());
        }

        // Check if this is a development toolchain command in sandbox mode
        if !command_parts.is_empty() {
            let program = &command_parts[0];
            if crate::tools::pty::is_development_toolchain_command(program) {
                if let Some(_profile) = self.pty_manager().sandbox_profile() {
                    return Err(anyhow!(
                        "{} could not be executed in the sandbox. This may be due to missing {} toolchain support in the current environment.\n\n\
                        Next steps:\n\
                        - Verify that the {} toolchain is installed and accessible.\n\
                        - Disable sandbox with `/sandbox disable` to run development tools with local toolchain access.\n\
                        - Alternatively, run the command directly in your terminal outside VT Code.",
                        program,
                        program,
                        program
                    ));
                }
            }
        }

        let working_dir = self
            .pty_manager()
            .resolve_working_dir(payload.get("working_dir").and_then(|value| value.as_str()))
            .await?;

        let rows =
            parse_pty_dimension("rows", payload.get("rows"), self.pty_config().default_rows)?;
        let cols =
            parse_pty_dimension("cols", payload.get("cols"), self.pty_config().default_cols)?;

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        debug!(
            target: "vtcode::pty",
            session_id = %session_id,
            command = ?command_parts,
            working_dir = %working_dir.display(),
            rows,
            cols,
            "creating PTY session"
        );

        self.start_pty_session()?;
        let result = match self.pty_manager().create_session(
            session_id.clone(),
            command_parts.clone(),
            working_dir,
            size,
        ) {
            Ok(meta) => meta,
            Err(error) => {
                self.end_pty_session();
                return Err(error);
            }
        };

        let mut response = snapshot_to_map(result, PtySnapshotViewOptions::default());
        response.insert("success".to_string(), Value::Bool(true));

        Ok(Value::Object(response))
    }

    async fn execute_list_pty_sessions(&self) -> Result<Value> {
        let sessions = self.pty_manager().list_sessions();
        let identifiers: Vec<String> = sessions.iter().map(|session| session.id.clone()).collect();
        let details: Vec<Value> = sessions
            .into_iter()
            .map(|session| {
                Value::Object(snapshot_to_map(session, PtySnapshotViewOptions::default()))
            })
            .collect();

        Ok(json!({
            "success": true,
            "sessions": identifiers,
            "details": details,
        }))
    }

    async fn execute_close_pty_session(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "close_pty_session expects an object payload")?;
        let session_id =
            parse_session_id(payload, "close_pty_session requires a 'session_id' string")?;

        let metadata = self
            .pty_manager()
            .close_session(session_id.as_str())
            .with_context(|| format!("failed to close PTY session '{session_id}'"))?;
        self.end_pty_session();

        let mut response = snapshot_to_map(metadata, PtySnapshotViewOptions::default());
        response.insert("success".to_string(), Value::Bool(true));

        Ok(Value::Object(response))
    }

    async fn execute_send_pty_input(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "send_pty_input expects an object payload")?;
        let input = PtyInputPayload::from_map(payload)?;

        let written = self
            .pty_manager()
            .send_input_to_session(
                input.session_id.as_str(),
                &input.buffer,
                input.append_newline,
            )
            .with_context(|| format!("failed to write to PTY session '{}'", input.session_id))?;

        if input.wait_ms > 0 {
            sleep(Duration::from_millis(input.wait_ms)).await;
        }

        let output = self
            .pty_manager()
            .read_session_output(input.session_id.as_str(), input.drain_output)
            .with_context(|| format!("failed to read PTY session '{}' output", input.session_id))?;
        let snapshot = self
            .pty_manager()
            .snapshot_session(input.session_id.as_str())
            .with_context(|| format!("failed to snapshot PTY session '{}'", input.session_id))?;

        let mut response = snapshot_to_map(snapshot, PtySnapshotViewOptions::default());
        response.insert("success".to_string(), Value::Bool(true));
        response.insert("written_bytes".to_string(), Value::from(written));
        response.insert(
            "appended_newline".to_string(),
            Value::Bool(input.append_newline),
        );
        if let Some(output) = output {
            response.insert("output".to_string(), Value::String(strip_ansi(&output)));
        }

        Ok(Value::Object(response))
    }

    async fn execute_read_pty_session(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "read_pty_session expects an object payload")?;
        let view_args = PtySessionViewArgs::from_map(payload)?;

        let output = self
            .pty_manager()
            .read_session_output(view_args.session_id.as_str(), view_args.drain_output)
            .with_context(|| {
                format!(
                    "failed to read PTY session '{}' output",
                    view_args.session_id
                )
            })?;
        let snapshot = self
            .pty_manager()
            .snapshot_session(view_args.session_id.as_str())
            .with_context(|| {
                format!("failed to snapshot PTY session '{}'", view_args.session_id)
            })?;

        let mut response = snapshot_to_map(snapshot, view_args.view);
        response.insert("success".to_string(), Value::Bool(true));
        if let Some(output) = output {
            response.insert("output".to_string(), Value::String(strip_ansi(&output)));
        }

        Ok(Value::Object(response))
    }

    async fn execute_resize_pty_session(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "resize_pty_session expects an object payload")?;
        let session_id =
            parse_session_id(payload, "resize_pty_session requires a 'session_id' string")?;

        let current = self
            .pty_manager()
            .snapshot_session(session_id.as_str())
            .with_context(|| format!("failed to snapshot PTY session '{session_id}'"))?;

        let rows = parse_pty_dimension("rows", payload.get("rows"), current.rows)?;
        let cols = parse_pty_dimension("cols", payload.get("cols"), current.cols)?;

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let snapshot = self
            .pty_manager()
            .resize_session(session_id.as_str(), size)
            .with_context(|| format!("failed to resize PTY session '{session_id}'"))?;

        let mut response = snapshot_to_map(snapshot, PtySnapshotViewOptions::default());
        response.insert("success".to_string(), Value::Bool(true));

        Ok(Value::Object(response))
    }
}

fn value_as_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| anyhow!("{}", context))
}

fn parse_command_parts(
    payload: &Map<String, Value>,
    missing_error: &str,
    empty_error: &str,
) -> Result<Vec<String>> {
    let mut parts = match payload.get("command") {
        Some(Value::String(command)) => vec![command.to_string()],
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(|part| part.to_string())
                    .ok_or_else(|| anyhow!("command array must contain only strings"))
            })
            .collect::<Result<Vec<_>>>()?,
        Some(_) => {
            return Err(anyhow!("command must be a string or string array"));
        }
        None => {
            return Err(anyhow!("{}", missing_error));
        }
    };

    if let Some(args_value) = payload.get("args") {
        let args_array = args_value
            .as_array()
            .ok_or_else(|| anyhow!("args must be an array of strings"))?;
        for value in args_array {
            let Some(part) = value.as_str() else {
                return Err(anyhow!("args array must contain only strings"));
            };
            parts.push(part.to_string());
        }
    }

    if parts.is_empty() {
        return Err(anyhow!("{}", empty_error));
    }

    Ok(parts)
}

fn parse_pty_dimension(name: &str, value: Option<&Value>, default: u16) -> Result<u16> {
    let Some(raw) = value else {
        return Ok(default);
    };

    let numeric = raw
        .as_u64()
        .ok_or_else(|| anyhow!("{name} must be an integer"))?;
    if numeric == 0 {
        return Err(anyhow!("{name} must be greater than zero"));
    }
    if numeric > u16::MAX as u64 {
        return Err(anyhow!("{name} exceeds maximum value {}", u16::MAX));
    }

    Ok(numeric as u16)
}

fn bool_from_map(map: &Map<String, Value>, key: &str, default: bool) -> bool {
    map.get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn parse_session_id(payload: &Map<String, Value>, missing_error: &str) -> Result<String> {
    let raw_id = payload
        .get("session_id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow!(missing_error.to_string()))?;
    let trimmed = raw_id.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("session_id cannot be empty"));
    }

    Ok(trimmed.to_string())
}

#[derive(Clone, Copy, Debug)]
struct PtySnapshotViewOptions {
    include_screen: bool,
    include_scrollback: bool,
}

impl PtySnapshotViewOptions {
    fn new(include_screen: bool, include_scrollback: bool) -> Self {
        Self {
            include_screen,
            include_scrollback,
        }
    }
}

impl Default for PtySnapshotViewOptions {
    fn default() -> Self {
        Self {
            include_screen: true,
            include_scrollback: true,
        }
    }
}

fn snapshot_to_map(
    snapshot: VTCodePtySession,
    options: PtySnapshotViewOptions,
) -> Map<String, Value> {
    let VTCodePtySession {
        id,
        command,
        args,
        working_dir,
        rows,
        cols,
        screen_contents,
        scrollback,
    } = snapshot;

    let mut response = Map::new();
    response.insert("session_id".to_string(), Value::String(id));
    response.insert("command".to_string(), Value::String(command));
    response.insert(
        "args".to_string(),
        Value::Array(args.into_iter().map(Value::String).collect()),
    );
    let working_directory = working_dir.unwrap_or_else(|| ".".to_string());
    response.insert(
        "working_directory".to_string(),
        Value::String(working_directory),
    );
    response.insert("rows".to_string(), Value::from(rows));
    response.insert("cols".to_string(), Value::from(cols));

    if options.include_screen {
        if let Some(screen) = screen_contents {
            response.insert(
                "screen_contents".to_string(),
                Value::String(strip_ansi(&screen)),
            );
        }
    }

    if options.include_scrollback {
        if let Some(scrollback) = scrollback {
            response.insert(
                "scrollback".to_string(),
                Value::String(strip_ansi(&scrollback)),
            );
        }
    }

    response
}

fn strip_ansi(text: &str) -> String {
    crate::utils::ansi_parser::strip_ansi(text)
}

struct PtySessionViewArgs {
    session_id: String,
    drain_output: bool,
    view: PtySnapshotViewOptions,
}

impl PtySessionViewArgs {
    fn from_map(map: &Map<String, Value>) -> Result<Self> {
        let session_id = parse_session_id(map, "read_pty_session requires a 'session_id' string")?;
        let drain_output = bool_from_map(map, "drain", false);
        let include_screen = bool_from_map(map, "include_screen", true);
        let include_scrollback = bool_from_map(map, "include_scrollback", true);

        Ok(Self {
            session_id,
            drain_output,
            view: PtySnapshotViewOptions::new(include_screen, include_scrollback),
        })
    }
}

#[derive(Debug)]
struct PtyInputPayload {
    session_id: String,
    buffer: Vec<u8>,
    append_newline: bool,
    wait_ms: u64,
    drain_output: bool,
}

impl PtyInputPayload {
    fn from_map(map: &Map<String, Value>) -> Result<Self> {
        let session_id = parse_session_id(map, "send_pty_input requires a 'session_id' string")?;
        let append_newline = bool_from_map(map, "append_newline", false);
        let wait_ms = map
            .get("wait_ms")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let drain_output = bool_from_map(map, "drain", true);

        let input_text = map.get("input").and_then(Value::as_str);
        let input_base64_text = map.get("input_base64").and_then(Value::as_str);
        let input_preview = input_text.map(Self::preview_string);
        let input_base64_preview = input_base64_text.map(Self::preview_string);

        debug!(
            target: "vtcode::pty",
            session_id = %session_id,
            append_newline,
            wait_ms,
            drain_output,
            input_len = input_text.map(|text| text.len()).unwrap_or(0),
            input_preview = input_preview.as_deref(),
            input_base64_len = input_base64_text.map(|text| text.len()).unwrap_or(0),
            input_base64_preview = input_base64_preview.as_deref(),
            "received send_pty_input payload"
        );

        let mut buffer = Vec::new();

        // Prefer input_base64 if present, else use input
        if let Some(encoded) = map
            .get("input_base64")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
        {
            let decoded = BASE64_STANDARD
                .decode(encoded.as_bytes())
                .context("input_base64 must be valid base64")?;
            buffer.extend_from_slice(&decoded);
        } else if let Some(text) = map.get("input").and_then(|value| value.as_str()) {
            buffer.extend_from_slice(text.as_bytes());
        }

        debug!(
            target: "vtcode::pty",
            session_id = %session_id,
            buffer_len = buffer.len(),
            buffer_preview = %Self::preview_bytes(&buffer),
            "prepared PTY input buffer"
        );

        if buffer.is_empty() && !append_newline {
            debug!(
                target: "vtcode::pty",
                session_id = %session_id,
                "rejecting empty PTY input without append_newline"
            );
            return Err(anyhow!(
                "send_pty_input requires 'input' or 'input_base64' unless append_newline is true"
            ));
        }

        trace!(
            target: "vtcode::pty",
            session_id = session_id.as_str(),
            append_newline,
            wait_ms,
            drain_output,
            has_input = map.contains_key("input"),
            has_input_base64 = map.contains_key("input_base64"),
            buffer_len = buffer.len(),
            "parsed PTY input payload"
        );

        Ok(Self {
            session_id,
            buffer,
            append_newline,
            wait_ms,
            drain_output,
        })
    }

    fn preview_string(text: &str) -> String {
        const MAX_PREVIEW: usize = 64;
        if text.len() <= MAX_PREVIEW {
            text.to_string()
        } else {
            format!("{}…", &text[..MAX_PREVIEW])
        }
    }

    fn preview_bytes(bytes: &[u8]) -> String {
        const MAX_BYTES: usize = 64;
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Self::preview_string(text);
        }

        let mut hex = String::new();
        for byte in bytes.iter().take(MAX_BYTES / 2) {
            use std::fmt::Write as _;
            let _ = write!(hex, "{:02x}", byte);
        }
        if bytes.len() > MAX_BYTES / 2 {
            hex.push('…');
        }
        format!("hex:{}", hex)
    }
}

fn resolve_shell_preference(explicit: Option<&str>, config: &PtyConfig) -> String {
    use std::env;
    use std::path::PathBuf;

    explicit
        .and_then(sanitize_shell_candidate)
        .or_else(|| {
            config
                .preferred_shell
                .as_deref()
                .and_then(sanitize_shell_candidate)
        })
        .or_else(|| {
            env::var("SHELL")
                .ok()
                .and_then(|value| sanitize_shell_candidate(&value))
        })
        .or_else(detect_posix_shell_candidate)
        .unwrap_or_else(|| resolve_shell_candidate().display().to_string())
}

fn resolve_shell_candidate() -> std::path::PathBuf {
    // Resolve the preferred shell for sandbox execution
    // Detects available shells based on platform
    if cfg!(windows) {
        // Windows: prefer PowerShell if available, fall back to cmd.exe
        if Path::new("C:\\Windows\\System32\\pwsh.exe").exists() {
            std::path::PathBuf::from("C:\\Windows\\System32\\pwsh.exe")
        } else if Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
            std::path::PathBuf::from("C:\\Program Files\\PowerShell\\7\\pwsh.exe")
        } else {
            std::path::PathBuf::from("cmd.exe")
        }
    } else {
        // POSIX systems: use detected shell or default to /bin/sh
        detect_posix_shell_candidate()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/bin/sh"))
    }
}

fn sanitize_shell_candidate(shell: &str) -> Option<String> {
    let trimmed = shell.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn detect_posix_shell_candidate() -> Option<String> {
    if cfg!(windows) {
        return None;
    }

    const CANDIDATES: [&str; 6] = [
        "/bin/zsh",
        "/usr/bin/zsh",
        "/bin/bash",
        "/usr/bin/bash",
        "/bin/sh",
        "/usr/bin/sh",
    ];

    for candidate in CANDIDATES {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    None
}

fn is_default_shell_placeholder(program: &str) -> bool {
    matches!(normalized_shell_name(program).as_str(), "bash" | "sh")
}

fn should_use_windows_command_tokenizer(shell_hint: Option<&str>) -> bool {
    if let Some(shell) = shell_hint {
        if is_windows_shell(shell) {
            return true;
        }
    }

    cfg!(windows)
}

fn is_windows_shell(shell: &str) -> bool {
    matches!(
        normalized_shell_name(shell).as_str(),
        "cmd" | "cmd.exe" | "powershell" | "powershell.exe" | "pwsh"
    )
}

fn normalized_shell_name(shell: &str) -> String {
    Path::new(shell)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(shell)
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::engine::general_purpose::STANDARD as BASE64;
    use serde_json::json;

    #[test]
    fn pty_input_prefers_base64_over_plain_text() {
        let mut payload = Map::new();
        payload.insert(
            "session_id".to_string(),
            Value::String("test-session".into()),
        );
        payload.insert("append_newline".to_string(), Value::Bool(false));
        payload.insert("input".to_string(), Value::String("plain".into()));
        let encoded = BASE64.encode(b"decoded");
        payload.insert("input_base64".to_string(), Value::String(encoded));

        let parsed = PtyInputPayload::from_map(&payload).expect("pty payload");
        assert_eq!(parsed.buffer, b"decoded");
        assert!(!parsed.append_newline);
    }

    #[test]
    fn pty_input_rejects_empty_payload_without_newline() {
        let mut payload = Map::new();
        payload.insert(
            "session_id".to_string(),
            Value::String("empty-session".into()),
        );

        let err = PtyInputPayload::from_map(&payload).expect_err("expected failure");
        assert!(
            err.to_string()
                .contains("send_pty_input requires 'input' or 'input_base64'")
        );
    }

    #[test]
    fn pty_input_prefers_base64_over_plain_text_v2() {
        let map: Map<String, Value> = json!({
            "session_id": "pty-1",
            "input": "ls",
            "input_base64": BASE64.encode("pwd"),
            "append_newline": false,
        })
        .as_object()
        .unwrap()
        .clone();

        let payload = PtyInputPayload::from_map(&map).expect("payload");
        assert_eq!(payload.buffer, b"pwd");
        assert!(!payload.append_newline);
    }

    #[test]
    fn pty_input_uses_plain_text_when_base64_missing() {
        let map: Map<String, Value> = json!({
            "session_id": "pty-2",
            "input": "echo hello",
        })
        .as_object()
        .unwrap()
        .clone();

        let payload = PtyInputPayload::from_map(&map).expect("payload");
        assert_eq!(payload.buffer, b"echo hello");
        assert!(!payload.append_newline);
    }

    #[test]
    fn pty_input_rejects_empty_without_newline() {
        let map: Map<String, Value> = json!({
            "session_id": "pty-3",
            "input": "",
            "append_newline": false,
        })
        .as_object()
        .unwrap()
        .clone();

        let err = PtyInputPayload::from_map(&map).unwrap_err();
        assert!(
            err.to_string()
                .contains("send_pty_input requires 'input' or 'input_base64'")
        );
    }

    #[test]
    fn pty_input_allows_empty_when_newline_requested() {
        let map: Map<String, Value> = json!({
            "session_id": "pty-4",
            "input": "",
            "append_newline": true,
        })
        .as_object()
        .unwrap()
        .clone();

        let payload = PtyInputPayload::from_map(&map).expect("payload");
        assert!(payload.buffer.is_empty());
        assert!(payload.append_newline);
    }

    #[test]
    fn resolve_shell_preference_uses_explicit_value() {
        let mut config = PtyConfig::default();
        config.preferred_shell = Some("/bin/bash".to_string());
        let resolved = super::resolve_shell_preference(Some("/custom/zsh"), &config);
        assert_eq!(resolved, "/custom/zsh");
    }

    #[test]
    fn resolve_shell_preference_uses_config_value() {
        let mut config = PtyConfig::default();
        config.preferred_shell = Some("/bin/zsh".to_string());
        let resolved = super::resolve_shell_preference(None, &config);
        assert_eq!(resolved, "/bin/zsh");
    }

    #[test]
    fn resolve_shell_preference_always_returns_value() {
        let config = PtyConfig::default();
        let resolved = super::resolve_shell_preference(None, &config);
        // Should never return empty string - guaranteed to have a fallback
        assert!(!resolved.is_empty());
    }
}
