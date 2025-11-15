//! File path and directory constants

/// Prompt path constants to avoid hardcoding throughout the codebase
pub const DEFAULT_SYSTEM_PROMPT_PATH: &str = "prompts/system.md";
pub const DEFAULT_CUSTOM_PROMPTS_DIR: &str = "~/.vtcode/prompts";
pub const CUSTOM_PROMPTS_ENV_VAR: &str = "VTCODE_HOME";
pub const DEFAULT_CUSTOM_PROMPT_MAX_FILE_SIZE_KB: usize = 64;
pub const CORE_BUILTIN_PROMPTS_DIR: &str = "vtcode-core/prompts/custom";
