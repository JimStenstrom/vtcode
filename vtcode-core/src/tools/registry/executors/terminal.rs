//! Terminal command execution module.
//!
//! This module handles ephemeral terminal command execution using PTY (pseudo-terminal).
//! It provides functionality for running one-off commands with proper timeout handling,
//! output collection, and shell integration.

use crate::config::PtyConfig;
use crate::tools::types::{EnhancedTerminalInput, VTCodePtySession};
use crate::tools::{PtyCommandRequest, PtyCommandResult, PtyManager};
use anyhow::{Context, Result, anyhow};
use futures::future::BoxFuture;
use portable_pty::PtySize;
use serde_json::{Map, Value, json};
use shell_words::{join, split};
use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::debug;

use super::super::ToolRegistry;

const DEFAULT_TERMINAL_TIMEOUT_SECS: u64 = 180;
const DEFAULT_PTY_TIMEOUT_SECS: u64 = 300;
const RUN_PTY_POLL_TIMEOUT_SECS: u64 = 5;
const LONG_RUNNING_COMMAND_TIMEOUT_SECS: u64 = 600;
// For known long-running commands, wait longer before returning partial output
const RUN_PTY_POLL_TIMEOUT_LONG_RUNNING: u64 = 30;
const LONG_RUNNING_COMMANDS: &[&str] = &[
    "cargo", "npm", "yarn", "pnpm", "pip", "python", "make", "docker",
];
const INTERACTIVE_COMMANDS: &[&str] = &[
    "python",
    "python3",
    "node",
    "npm",
    "yarn",
    "pnpm",
    "bun",
    "irb",
    "pry",
    "node-repl",
    "mysql",
    "psql",
    "sqlite3",
    "vim",
    "nvim",
    "nano",
    "emacs",
    "code",
    "top",
    "htop",
    "ssh",
    "telnet",
    "ftp",
    "sftp",
    "cargo",
    "make",
    "cmake",
    "ninja",
    "gradle",
    "mvn",
    "ant",
    "go",
    "rustc",
    "gcc",
    "g++",
    "clang",
    "javac",
    "dotnet",
];

impl ToolRegistry {
    pub(in crate::tools::registry) fn run_command_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { self.execute_run_command(args).await })
    }

    /// Unified command execution that combines terminal and PTY modes
    async fn execute_run_command(&mut self, mut args: Value) -> Result<Value> {
        normalize_run_command_payload(&mut args)?;

        let resolved_mode = resolve_run_mode(&args);
        ensure_default_timeout(
            &mut args,
            "run_command expects an object payload",
            resolved_mode.default_timeout(),
        )?;

        if resolved_mode.is_pty() {
            self.execute_run_pty_command(args).await
        } else {
            self.execute_run_terminal_internal(args).await
        }
    }

    async fn execute_run_terminal_internal(&mut self, mut args: Value) -> Result<Value> {
        match prepare_terminal_execution(&mut args)? {
            TerminalExecution::Pty { args } => self.execute_run_pty_command(args).await,
            TerminalExecution::Terminal(execution) => {
                let plan = self.build_terminal_command_plan(execution).await?;
                plan.execute(self.pty_manager()).await
            }
        }
    }

    async fn execute_run_pty_command(&mut self, args: Value) -> Result<Value> {
        let payload = value_as_object(&args, "run_pty_cmd expects an object payload")?;
        let setup = self.prepare_ephemeral_pty_command(payload).await?;
        self.run_ephemeral_pty_command(setup).await
    }

    async fn prepare_ephemeral_pty_command(
        &self,
        payload: &Map<String, Value>,
    ) -> Result<PtyCommandSetup> {
        let mut command = parse_command_parts(
            payload,
            "run_pty_cmd requires a 'command' value",
            "PTY command cannot be empty",
        )?;

        let raw_command = payload
            .get("raw_command")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        let shell_program = resolve_shell_preference(
            payload.get("shell").and_then(|value| value.as_str()),
            self.pty_config(),
        );
        let login_shell = payload
            .get("login")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        {
            let normalized_shell = normalized_shell_name(&shell_program);
            let existing_shell = command
                .first()
                .map(|existing| normalized_shell_name(existing));
            if existing_shell != Some(normalized_shell.clone()) {
                let command_string =
                    build_shell_command_string(raw_command.as_deref(), &command, &shell_program);

                let mut shell_invocation = Vec::with_capacity(4);
                shell_invocation.push(shell_program.clone());

                if login_shell && !should_use_windows_command_tokenizer(Some(&shell_program)) {
                    shell_invocation.push("-l".to_string());
                }

                let command_flag = if should_use_windows_command_tokenizer(Some(&shell_program)) {
                    match normalized_shell.as_str() {
                        "cmd" | "cmd.exe" => "/C".to_string(),
                        "powershell" | "powershell.exe" | "pwsh" => "-Command".to_string(),
                        _ => "-c".to_string(),
                    }
                } else {
                    "-c".to_string()
                };

                shell_invocation.push(command_flag);
                shell_invocation.push(command_string);
                command = shell_invocation;
            }
        }

        let timeout_secs = parse_timeout_secs(
            payload.get("timeout_secs"),
            self.pty_config().command_timeout_seconds,
        )?;
        let rows =
            parse_pty_dimension("rows", payload.get("rows"), self.pty_config().default_rows)?;
        let cols =
            parse_pty_dimension("cols", payload.get("cols"), self.pty_config().default_cols)?;

        let working_dir_path = self
            .pty_manager()
            .resolve_working_dir(payload.get("working_dir").and_then(|value| value.as_str()))
            .await?;
        let working_dir_display = self.pty_manager().describe_working_dir(&working_dir_path);

        Ok(PtyCommandSetup {
            command,
            working_dir_path,
            working_dir_display,
            session_id: generate_session_id("run"),
            rows,
            cols,
            timeout_secs,
        })
    }

    async fn run_ephemeral_pty_command(&mut self, setup: PtyCommandSetup) -> Result<Value> {
        let mut lifecycle = PtySessionLifecycle::start(self)?;
        self.pty_manager()
            .create_session(
                setup.session_id.clone(),
                setup.command.clone(),
                setup.working_dir_path.clone(),
                setup.size(),
            )
            .with_context(|| {
                format!(
                    "failed to create PTY session '{}' for command {:?}",
                    setup.session_id, setup.command
                )
            })?;
        lifecycle.commit();

        // Use adaptive timeout: longer for known long-running commands
        let poll_timeout = if is_long_running_command(&setup.command) {
            Duration::from_secs(RUN_PTY_POLL_TIMEOUT_LONG_RUNNING)
        } else {
            Duration::from_secs(RUN_PTY_POLL_TIMEOUT_SECS)
        };

        let capture =
            collect_ephemeral_session_output(self.pty_manager(), &setup.session_id, poll_timeout)
                .await;

        let snapshot = self
            .pty_manager()
            .snapshot_session(&setup.session_id)
            .with_context(|| format!("failed to snapshot PTY session '{}'", setup.session_id))?;

        Ok(build_ephemeral_pty_response(&setup, capture, snapshot))
    }

    async fn build_terminal_command_plan(
        &mut self,
        execution: TerminalExecutionInput,
    ) -> Result<TerminalCommandPlan> {
        let TerminalExecutionInput { input, mode_label } = execution;
        let invocation = self
            .inventory
            .command_tool()
            .prepare_invocation(&input)
            .await?;

        let working_dir_path = self
            .pty_manager()
            .resolve_working_dir(input.working_dir.as_deref())
            .await?;

        let timeout_secs = validated_timeout_secs(
            input.timeout_secs,
            self.pty_config().command_timeout_seconds,
        )?;

        let command = assemble_command_segments(&invocation.program, &invocation.args);
        let request = PtyCommandRequest {
            command,
            working_dir: working_dir_path.clone(),
            timeout: Duration::from_secs(timeout_secs),
            size: default_pty_size(
                self.pty_config().default_rows,
                self.pty_config().default_cols,
            ),
        };

        let working_directory = self.pty_manager().describe_working_dir(&working_dir_path);

        Ok(TerminalCommandPlan {
            request,
            command_display: invocation.display,
            working_directory,
            mode_label,
            timeout_secs,
        })
    }
}

fn copy_value_if_absent(map: &mut Map<String, Value>, source_key: &str, target_key: &str) {
    if map.contains_key(target_key) {
        return;
    }

    if let Some(value) = map.get(source_key).cloned() {
        map.insert(target_key.to_string(), value);
    }
}

fn normalize_payload<'a>(
    args: &'a mut Value,
    context: &str,
    legacy_error: &str,
) -> Result<&'a mut Map<String, Value>> {
    let map = value_as_object_mut(args, context)?;
    if map.contains_key("bash_command") {
        return Err(anyhow!(legacy_error.to_string()));
    }
    copy_value_if_absent(map, "cwd", "working_dir");
    Ok(map)
}

fn normalize_run_command_payload(args: &mut Value) -> Result<()> {
    normalize_payload(
        args,
        "run_command expects an object payload",
        "bash_command is no longer supported. Use run_command instead.",
    )?;
    Ok(())
}

fn normalize_terminal_payload(args: &mut Value) -> Result<()> {
    {
        let map = normalize_payload(
            args,
            "run_terminal_cmd expects an object payload",
            "bash_command is no longer supported. Use run_terminal_cmd or run_pty_cmd instead.",
        )?;
        if !map.contains_key("mode") {
            match map.get("tty").and_then(|value| value.as_bool()) {
                Some(true) => {
                    map.insert("mode".to_string(), Value::String("pty".to_string()));
                }
                Some(false) => {
                    map.insert("mode".to_string(), Value::String("terminal".to_string()));
                }
                None => {}
            }
        }
        copy_value_if_absent(map, "timeout", "timeout_secs");
    }
    Ok(())
}

fn ensure_default_timeout(args: &mut Value, context: &str, default: u64) -> Result<()> {
    let map = value_as_object_mut(args, context)?;
    if !map.contains_key("timeout_secs") {
        let timeout = if let Some(cmd_parts) = collect_command_array(map).ok().flatten() {
            if is_long_running_command(&cmd_parts) {
                LONG_RUNNING_COMMAND_TIMEOUT_SECS
            } else {
                default
            }
        } else {
            default
        };
        map.insert("timeout_secs".to_string(), Value::Number(timeout.into()));
    }
    Ok(())
}

fn collect_command_array(map: &Map<String, Value>) -> Result<Option<Vec<String>>> {
    if let Some(cmd) = map.get("command") {
        if let Some(cmd_str) = cmd.as_str() {
            return Ok(Some(vec![cmd_str.to_string()]));
        } else if let Some(cmd_array) = cmd.as_array() {
            return Ok(Some(
                cmd_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
            ));
        }
    }
    Ok(None)
}

fn parse_timeout_secs(value: Option<&Value>, fallback: u64) -> Result<u64> {
    let parsed = value
        .map(|raw| {
            raw.as_u64()
                .ok_or_else(|| anyhow!("timeout_secs must be a positive integer"))
        })
        .transpose()?;
    validated_timeout_secs(parsed, fallback)
}

fn validated_timeout_secs(raw: Option<u64>, fallback: u64) -> Result<u64> {
    let timeout_secs = raw.unwrap_or(fallback);
    if timeout_secs == 0 {
        return Err(anyhow!("timeout_secs must be greater than zero"));
    }
    Ok(timeout_secs)
}

fn assemble_command_segments(program: &str, args: &[String]) -> Vec<String> {
    let mut command = Vec::with_capacity(1 + args.len());
    command.push(program.to_string());
    command.extend(args.iter().cloned());
    command
}

fn default_pty_size(default_rows: u16, default_cols: u16) -> PtySize {
    PtySize {
        rows: default_rows,
        cols: default_cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn run_mode_label(args: &Value) -> String {
    args.get("mode")
        .and_then(|value| value.as_str())
        .unwrap_or("terminal")
        .to_string()
}

fn build_terminal_command_response(
    result: &PtyCommandResult,
    mode_label: &str,
    command_display: &str,
    working_directory: String,
    timeout_secs: u64,
) -> Value {
    json!({
        "success": result.exit_code == 0,
        "exit_code": result.exit_code,
        "stdout": result.output,
        "stderr": "",
        "output": result.output,
        "mode": mode_label,
        "pty_enabled": true,
        "command": command_display,
        "working_directory": working_directory,
        "timeout_secs": timeout_secs,
        "duration_ms": result.duration.as_millis(),
        "pty": {
            "rows": result.size.rows,
            "cols": result.size.cols,
        },
    })
}

fn convert_command_string_to_array(
    args: &mut Value,
    shell_hint: Option<&str>,
    context: &str,
) -> Result<Option<String>> {
    let map = value_as_object_mut(args, context)?;
    let maybe_command = map
        .get("command")
        .and_then(|value| value.as_str().map(|value| value.to_string()));
    let Some(command_str) = maybe_command else {
        return Ok(None);
    };

    let sanitized = sanitize_command_string(&command_str);
    let segments = tokenize_command_string(sanitized.as_ref(), shell_hint)
        .map_err(|err| anyhow!("failed to parse command string: {}", err))?;
    if segments.is_empty() {
        return Err(anyhow!("command string cannot be empty"));
    }

    let command_array = segments.into_iter().map(Value::String).collect::<Vec<_>>();
    map.insert("command".to_string(), Value::Array(command_array));

    Ok(Some(command_str))
}

fn collect_command_vector(
    args: &Value,
    missing_error: &str,
    type_error: &str,
) -> Result<Vec<String>> {
    let map = value_as_object(args, missing_error)?;
    let array = map
        .get("command")
        .ok_or_else(|| anyhow!(missing_error.to_string()))?
        .as_array()
        .ok_or_else(|| anyhow!(missing_error.to_string()))?;

    array
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|part| part.to_string())
                .ok_or_else(|| anyhow!(type_error.to_string()))
        })
        .collect()
}

fn determine_terminal_run_mode(args: &Value) -> Result<RunMode> {
    if matches!(
        args.get("mode").and_then(|value| value.as_str()),
        Some("streaming")
    ) {
        return Err(anyhow!("run_terminal_cmd does not support streaming mode"));
    }

    Ok(resolve_run_mode(args))
}

fn build_terminal_command_payload(
    args: &Value,
    command_vec: &[String],
    raw_command: Option<&str>,
) -> Map<String, Value> {
    let mut sanitized = serde_json::Map::new();
    let command_array = command_vec
        .iter()
        .cloned()
        .map(Value::String)
        .collect::<Vec<Value>>();
    sanitized.insert("command".to_string(), Value::Array(command_array));

    if let Some(working_dir) = args.get("working_dir").cloned() {
        sanitized.insert("working_dir".to_string(), working_dir);
    }
    if let Some(timeout) = args.get("timeout_secs").cloned() {
        sanitized.insert("timeout_secs".to_string(), timeout);
    }
    if let Some(response_format) = args.get("response_format").cloned() {
        sanitized.insert("response_format".to_string(), response_format);
    }
    if let Some(raw) = raw_command {
        sanitized.insert("raw_command".to_string(), Value::String(raw.to_string()));
    }
    if let Some(shell) = args.get("shell").cloned() {
        sanitized.insert("shell".to_string(), shell);
    }
    if let Some(login) = args.get("login").cloned() {
        sanitized.insert("login".to_string(), login);
    }

    sanitized
}

fn build_pty_args_from_terminal(args: &Value, command_vec: &[String]) -> Map<String, Value> {
    let mut pty_args = serde_json::Map::new();
    if let Some(program) = command_vec.first() {
        pty_args.insert("command".to_string(), Value::String(program.clone()));
    }
    if command_vec.len() > 1 {
        let rest = command_vec[1..]
            .iter()
            .cloned()
            .map(Value::String)
            .collect::<Vec<Value>>();
        pty_args.insert("args".to_string(), Value::Array(rest));
    }
    if let Some(timeout) = args.get("timeout_secs").cloned() {
        pty_args.insert("timeout_secs".to_string(), timeout);
    }
    if let Some(working_dir) = args.get("working_dir").cloned() {
        pty_args.insert("working_dir".to_string(), working_dir);
    }
    if let Some(response_format) = args.get("response_format").cloned() {
        pty_args.insert("response_format".to_string(), response_format);
    }
    if let Some(rows) = args.get("rows").cloned() {
        pty_args.insert("rows".to_string(), rows);
    }
    if let Some(cols) = args.get("cols").cloned() {
        pty_args.insert("cols".to_string(), cols);
    }
    if let Some(shell) = args.get("shell").cloned() {
        pty_args.insert("shell".to_string(), shell);
    }
    if let Some(login) = args.get("login").cloned() {
        pty_args.insert("login".to_string(), login);
    }
    if let Some(raw_command) = args.get("raw_command").cloned() {
        pty_args.insert("raw_command".to_string(), raw_command);
    }

    pty_args
}

struct TerminalExecutionInput {
    input: EnhancedTerminalInput,
    mode_label: String,
}

enum TerminalExecution {
    Terminal(TerminalExecutionInput),
    Pty { args: Value },
}

struct TerminalCommandPayload {
    sanitized: Map<String, Value>,
    run_mode: RunMode,
    mode_label: String,
    pty_args: Option<Map<String, Value>>,
}

impl TerminalCommandPayload {
    fn parse(args: &mut Value) -> Result<Self> {
        normalize_terminal_payload(args)?;

        let mode_label = run_mode_label(args);
        let shell_hint = args
            .get("shell")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        let raw_command = convert_command_string_to_array(
            args,
            shell_hint.as_deref(),
            "run_terminal_cmd expects an object payload",
        )?;
        let command_vec = collect_command_vector(
            args,
            "run_terminal_cmd requires a 'command' array",
            "command array must contain only strings",
        )?;
        if command_vec.is_empty() {
            return Err(anyhow!("command array cannot be empty"));
        }

        let run_mode = determine_terminal_run_mode(args)?;
        let sanitized = build_terminal_command_payload(args, &command_vec, raw_command.as_deref());
        let pty_args = run_mode
            .is_pty()
            .then(|| build_pty_args_from_terminal(args, &command_vec));

        Ok(Self {
            sanitized,
            run_mode,
            mode_label,
            pty_args,
        })
    }

    fn into_execution(self) -> Result<TerminalExecution> {
        let TerminalCommandPayload {
            sanitized,
            run_mode,
            mode_label,
            pty_args,
        } = self;

        match run_mode {
            RunMode::Pty => {
                let args = pty_args
                    .ok_or_else(|| anyhow!("failed to prepare PTY payload for terminal command"))?;
                Ok(TerminalExecution::Pty {
                    args: Value::Object(args),
                })
            }
            RunMode::Terminal => {
                let sanitized_value = Value::Object(sanitized);
                let input: EnhancedTerminalInput = serde_json::from_value(sanitized_value)
                    .context("failed to parse terminal command input")?;
                Ok(TerminalExecution::Terminal(TerminalExecutionInput {
                    input,
                    mode_label,
                }))
            }
        }
    }
}

struct TerminalCommandPlan {
    request: PtyCommandRequest,
    command_display: String,
    working_directory: String,
    mode_label: String,
    timeout_secs: u64,
}

impl TerminalCommandPlan {
    async fn execute(self, manager: &PtyManager) -> Result<Value> {
        let TerminalCommandPlan {
            request,
            command_display,
            working_directory,
            mode_label,
            timeout_secs,
        } = self;

        let result = manager.run_command(request).await?;
        Ok(build_terminal_command_response(
            &result,
            &mode_label,
            &command_display,
            working_directory,
            timeout_secs,
        ))
    }
}

fn prepare_terminal_execution(args: &mut Value) -> Result<TerminalExecution> {
    let payload = TerminalCommandPayload::parse(args)?;
    payload.into_execution()
}

fn value_as_object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| anyhow!("{}", context))
}

fn value_as_object_mut<'a>(
    value: &'a mut Value,
    context: &str,
) -> Result<&'a mut Map<String, Value>> {
    value.as_object_mut().ok_or_else(|| anyhow!("{}", context))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunMode {
    Terminal,
    Pty,
}

impl RunMode {
    fn default_timeout(self) -> u64 {
        match self {
            RunMode::Terminal => DEFAULT_TERMINAL_TIMEOUT_SECS,
            RunMode::Pty => DEFAULT_PTY_TIMEOUT_SECS,
        }
    }

    fn is_pty(self) -> bool {
        matches!(self, RunMode::Pty)
    }
}

fn resolve_run_mode(args: &Value) -> RunMode {
    if let Some(mode_value) = args.get("mode").and_then(|value| value.as_str()) {
        return match mode_value {
            "pty" => RunMode::Pty,
            "terminal" => RunMode::Terminal,
            "auto" => detect_auto_mode(args),
            _ => RunMode::Terminal,
        };
    }

    if args
        .get("tty")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return RunMode::Pty;
    }

    detect_auto_mode(args)
}

fn detect_auto_mode(args: &Value) -> RunMode {
    if let Some(command) = primary_command(args) {
        if should_use_pty_for_command(&command) {
            return RunMode::Pty;
        }
    }

    RunMode::Terminal
}

fn primary_command(args: &Value) -> Option<String> {
    let command_value = args.get("command")?;

    if let Some(command) = command_value.as_str() {
        return Some(command.to_string());
    }

    command_value
        .as_array()
        .and_then(|values| values.get(0))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn should_use_pty_for_command(command: &str) -> bool {
    INTERACTIVE_COMMANDS
        .iter()
        .any(|candidate| command.contains(candidate))
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

struct PtyCommandSetup {
    command: Vec<String>,
    working_dir_path: PathBuf,
    working_dir_display: String,
    session_id: String,
    rows: u16,
    cols: u16,
    timeout_secs: u64,
}

impl PtyCommandSetup {
    fn size(&self) -> PtySize {
        PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

fn strip_ansi(text: &str) -> String {
    crate::utils::ansi_parser::strip_ansi(text)
}

struct PtyEphemeralCapture {
    output: String,
    exit_code: Option<i32>,
    completed: bool,
    duration: Duration,
}

async fn collect_ephemeral_session_output(
    manager: &PtyManager,
    session_id: &str,
    poll_timeout: Duration,
) -> PtyEphemeralCapture {
    let mut output = String::new();
    let start = Instant::now();
    let mut completed = false;
    let mut exit_code = None;
    let poll_interval = Duration::from_millis(50);
    let min_wait = Duration::from_millis(200); // Wait at least 200ms for fast commands

    loop {
        if let Ok(Some(new_output)) = manager.read_session_output(session_id, true) {
            if !new_output.is_empty() {
                output.push_str(&new_output);
            }
        }

        if let Ok(Some(code)) = manager.is_session_completed(session_id) {
            completed = true;
            exit_code = Some(code);
            // Drain any remaining output
            if let Ok(Some(final_output)) = manager.read_session_output(session_id, true) {
                output.push_str(&final_output);
            }
            break;
        }

        let elapsed = start.elapsed();

        // For long-running commands, return partial output early
        if elapsed > poll_timeout {
            break;
        }

        // If we have output and minimum wait time passed, check if we should return early
        if !output.is_empty() && elapsed > min_wait {
            // Return early if command is still running and we have output
            // This allows the agent to show progress
            if elapsed > Duration::from_secs(2) {
                break;
            }
        }

        sleep(poll_interval).await;
    }

    PtyEphemeralCapture {
        output,
        exit_code,
        completed,
        duration: start.elapsed(),
    }
}

fn build_ephemeral_pty_response(
    setup: &PtyCommandSetup,
    capture: PtyEphemeralCapture,
    snapshot: VTCodePtySession,
) -> Value {
    let PtyEphemeralCapture {
        output,
        exit_code,
        completed,
        duration,
    } = capture;

    let session_reference = if completed {
        None
    } else {
        Some(setup.session_id.clone())
    };
    let code = if completed { exit_code } else { None };
    let status = if completed { "completed" } else { "running" };

    // Build a clear message for the agent based on status
    let message = if completed {
        if let Some(exit_code) = code {
            if exit_code == 0 {
                "Command completed successfully".to_string()
            } else {
                format!("Command failed with exit code {}", exit_code)
            }
        } else {
            "Command completed".to_string()
        }
    } else {
        "Command is still running. Backend continues polling automatically. Do NOT call read_pty_session.".to_string()
    };

    json!({
        "success": true,
        "command": setup.command.clone(),
        "output": strip_ansi(&output),
        "code": code,
        "status": status,
        "message": message,
        "mode": "pty",
        "session_id": session_reference,
        "pty": {
            "rows": snapshot.rows,
            "cols": snapshot.cols,
        },
        "working_directory": setup.working_dir_display.clone(),
        "timeout_secs": setup.timeout_secs,
        "duration_ms": if completed { duration.as_millis() } else { 0 },
    })
}

fn build_shell_command_string(
    raw_command: Option<&str>,
    parts: &[String],
    shell_hint: &str,
) -> String {
    if let Some(raw) = raw_command {
        return raw.to_string();
    }

    if should_use_windows_command_tokenizer(Some(shell_hint)) {
        return join_windows_command(parts);
    }

    join(parts.iter().map(|part| part.as_str()))
}

fn join_windows_command(parts: &[String]) -> String {
    parts
        .iter()
        .map(|part| quote_windows_argument(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_windows_argument(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    let requires_quotes = arg
        .chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '\t');
    if !requires_quotes {
        return arg.to_string();
    }

    let mut result = String::with_capacity(arg.len() + 2);
    result.push('"');

    let mut backslashes = 0;
    for ch in arg.chars() {
        match ch {
            '\\' => {
                backslashes += 1;
            }
            '"' => {
                result.extend(std::iter::repeat('\\').take(backslashes * 2 + 1));
                result.push('"');
                backslashes = 0;
            }
            _ => {
                if backslashes > 0 {
                    result.extend(std::iter::repeat('\\').take(backslashes));
                    backslashes = 0;
                }
                result.push(ch);
            }
        }
    }

    if backslashes > 0 {
        result.extend(std::iter::repeat('\\').take(backslashes * 2));
    }

    result.push('"');
    result
}

fn sanitize_command_string(command: &str) -> Cow<'_, str> {
    let trimmed = command.trim_end_matches(char::is_whitespace);

    for &quote in &['\'', '"'] {
        let quote_count = trimmed.matches(quote).count();
        if quote_count % 2 != 0 && trimmed.ends_with(quote) {
            let mut adjusted = trimmed.to_string();
            adjusted.pop();
            return Cow::Owned(adjusted);
        }
    }

    if trimmed.len() != command.len() {
        Cow::Owned(trimmed.to_string())
    } else {
        Cow::Borrowed(command)
    }
}

fn tokenize_command_string(command: &str, shell_hint: Option<&str>) -> Result<Vec<String>> {
    if should_use_windows_command_tokenizer(shell_hint) {
        return tokenize_windows_command(command);
    }

    split(command).map_err(|err| anyhow!(err))
}

fn should_use_windows_command_tokenizer(shell_hint: Option<&str>) -> bool {
    if let Some(shell) = shell_hint {
        if is_windows_shell(shell) {
            return true;
        }
    }

    cfg!(windows)
}

fn tokenize_windows_command(command: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut token_started = false;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if matches!(chars.peek(), Some('"')) {
                        current.push('"');
                        token_started = true;
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                    token_started = true;
                }
            }
            c if c.is_whitespace() && !in_quotes => {
                if token_started {
                    tokens.push(current);
                    current = String::new();
                    token_started = false;
                }
            }
            _ => {
                current.push(ch);
                token_started = true;
            }
        }
    }

    if in_quotes {
        return Err(anyhow!("unterminated quote in command string"));
    }

    if token_started {
        tokens.push(current);
    }

    Ok(tokens)
}

fn resolve_shell_preference(explicit: Option<&str>, config: &PtyConfig) -> String {
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

fn resolve_shell_candidate() -> PathBuf {
    // Resolve the preferred shell for sandbox execution
    // Detects available shells based on platform
    if cfg!(windows) {
        // Windows: prefer PowerShell if available, fall back to cmd.exe
        if Path::new("C:\\Windows\\System32\\pwsh.exe").exists() {
            PathBuf::from("C:\\Windows\\System32\\pwsh.exe")
        } else if Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
            PathBuf::from("C:\\Program Files\\PowerShell\\7\\pwsh.exe")
        } else {
            PathBuf::from("cmd.exe")
        }
    } else {
        // POSIX systems: use detected shell or default to /bin/sh
        detect_posix_shell_candidate()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/bin/sh"))
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

fn generate_session_id(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis();
    format!("{prefix}-{timestamp}")
}

struct PtySessionLifecycle<'a> {
    registry: &'a ToolRegistry,
    active: bool,
}

impl<'a> PtySessionLifecycle<'a> {
    fn start(registry: &'a ToolRegistry) -> Result<Self> {
        registry.start_pty_session()?;
        Ok(Self {
            registry,
            active: true,
        })
    }

    fn commit(&mut self) {
        self.active = false;
    }
}

impl Drop for PtySessionLifecycle<'_> {
    fn drop(&mut self) {
        if self.active {
            self.registry.end_pty_session();
        }
    }
}

/// Detects if a command is known to be long-running (build tools, package managers, etc.)
fn is_long_running_command(command_parts: &[String]) -> bool {
    if let Some(first) = command_parts.first() {
        let cmd = first.to_lowercase();
        let basename = std::path::Path::new(&cmd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if it's a long-running command
        LONG_RUNNING_COMMANDS
            .iter()
            .any(|&long_cmd| basename.starts_with(long_cmd) || basename == long_cmd)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("hello"), "hello");
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("\x1b[1;32mbold green\x1b[0m"), "bold green");
        assert_eq!(
            strip_ansi("Checking \x1b[0m\x1b[1m\x1b[32mvtcode\x1b[0m"),
            "Checking vtcode"
        );
    }

    #[test]
    fn windows_tokenizer_preserves_paths_with_spaces() {
        let command = r#""C:\Program Files\Git\bin\bash.exe" -lc "echo hi""#;
        let tokens = tokenize_command_string(command, Some("cmd.exe")).expect("tokens");
        assert_eq!(
            tokens,
            vec![
                r"C:\Program Files\Git\bin\bash.exe".to_string(),
                "-lc".to_string(),
                "echo hi".to_string(),
            ]
        );
    }

    #[test]
    fn windows_tokenizer_handles_empty_arguments() {
        let tokens = tokenize_windows_command("\"\"").expect("tokens");
        assert_eq!(tokens, vec![String::new()]);
    }

    #[test]
    fn windows_tokenizer_errors_on_unterminated_quotes() {
        let err = tokenize_windows_command("\"unterminated").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn windows_join_quotes_arguments_with_spaces() {
        let parts = vec![
            r"C:\Program Files\Git\bin\git.exe".to_string(),
            "--version".to_string(),
        ];
        let joined = join_windows_command(&parts);
        assert_eq!(
            joined,
            r#""C:\Program Files\Git\bin\git.exe" --version"#.to_string()
        );
    }

    #[test]
    fn windows_join_leaves_simple_arguments_unquoted() {
        let parts = vec!["cmd".to_string(), "/C".to_string(), "dir".to_string()];
        let joined = join_windows_command(&parts);
        assert_eq!(joined, "cmd /C dir");
    }

    #[test]
    fn tokenizer_uses_posix_rules_for_posix_shells() {
        let tokens =
            tokenize_command_string("echo 'hello world'", Some("/bin/bash")).expect("tokens");
        assert_eq!(tokens, vec!["echo", "hello world"]);
    }

    #[test]
    fn detects_windows_shell_name_variants() {
        assert!(should_use_windows_command_tokenizer(Some(
            "C:/Windows/System32/cmd.exe"
        )));
        assert!(should_use_windows_command_tokenizer(Some("pwsh")));
        assert_eq!(normalized_shell_name("/bin/bash"), "bash");
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
