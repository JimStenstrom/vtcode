//! Default configuration values

use super::{models, ui};

pub const DEFAULT_MODEL: &str = models::openai::DEFAULT_MODEL;
pub const DEFAULT_CLI_MODEL: &str = models::openai::DEFAULT_MODEL;
pub const DEFAULT_PROVIDER: &str = "openai";
pub const DEFAULT_API_KEY_ENV: &str = "OPENAI_API_KEY";
pub const DEFAULT_THEME: &str = "ciapre-dark";
pub const DEFAULT_FULL_AUTO_MAX_TURNS: usize = 30;
pub const DEFAULT_MAX_TOOL_LOOPS: usize = 100;
pub const DEFAULT_MAX_REPEATED_TOOL_CALLS: usize = 3;
pub const ANTHROPIC_DEFAULT_MAX_TOKENS: u32 = 4_096;
pub const DEFAULT_PTY_STDOUT_TAIL_LINES: usize = 20;
pub const DEFAULT_PTY_SCROLLBACK_LINES: usize = 400;
pub const DEFAULT_TOOL_OUTPUT_MODE: &str = ui::TOOL_OUTPUT_MODE_COMPACT;
