//! Application constants organized by domain
//!
//! This module provides a well-organized collection of constants used throughout
//! the VTCode application. Constants are grouped into focused modules by domain:
//!
//! - `app`: Application metadata
//! - `paths`: File paths and directories
//! - `ui`: UI dimensions, colors, and display strings
//! - `tools`: Tool names and bash security rules
//! - `limits`: Size and boundary limits for file operations
//! - `context`: Context window management
//! - `models`: Model IDs for all LLM providers
//! - `commands`: Command execution defaults
//! - `defaults`: Default configuration values
//! - `prompt_cache`: Prompt caching defaults
//! - `model_helpers`: Model validation functions
//! - `env`: Environment variable names
//! - `reasoning`: Reasoning effort configuration
//! - `message_roles`: Message role constants
//! - `urls`: API endpoint URLs
//! - `env_vars`: Environment variable names for base URLs
//! - `headers`: HTTP header constants
//! - `llm_generation`: LLM generation parameters

pub mod app;
pub mod commands;
pub mod context;
pub mod defaults;
pub mod env;
pub mod env_vars;
pub mod headers;
pub mod limits;
pub mod llm_generation;
pub mod message_roles;
pub mod model_helpers;
pub mod models;
pub mod paths;
pub mod prompt_cache;
pub mod reasoning;
pub mod tools;
pub mod ui;
pub mod urls;

// Re-export commonly used constants for backwards compatibility

// Application metadata
pub use app::*;

// Paths (previously in prompts module)
pub mod prompts {
    pub use super::paths::*;
}

// Commands
pub use commands::*;

// Models - re-export the entire models module structure
pub use models::*;

// Prompt cache
pub use prompt_cache::*;

// Model helpers
pub use model_helpers::*;

// Environment
pub use env::*;

// Defaults
pub use defaults::*;

// UI
pub use ui::*;

// Reasoning
pub use reasoning::*;

// Message roles
pub use message_roles::*;

// URLs
pub use urls::*;

// Environment variables
pub use env_vars::*;

// Headers
pub use headers::*;

// Tools - re-export tool constants but keep bash and mcp as submodules
pub use tools::{
    APPLY_PATCH, CLOSE_PTY_SESSION, CREATE_FILE, CREATE_PTY_SESSION, DELETE_FILE, EDIT_FILE,
    EXECUTE_CODE, GREP_FILE, LIST_FILES, LIST_PTY_SESSIONS, READ_FILE, READ_PTY_SESSION,
    RESIZE_PTY_SESSION, RUN_COMMAND, RUN_PTY_CMD, SEARCH_TOOLS, SEND_PTY_INPUT, UPDATE_PLAN,
    WEB_FETCH, WILDCARD_ALL, WRITE_FILE,
};

// Bash module (keep as submodule)
pub use tools::bash;

// MCP module (keep as submodule)
pub use tools::mcp;

// Limits - re-export commonly used constants but keep submodules
pub use limits::{
    chunking, diff, instructions, project_doc, CHUNK_END_LINES, CHUNK_START_LINES,
    CONTEXT_RADIUS, HEAD_LINE_COUNT, MAX_LINES_THRESHOLD, MAX_PREVIEW_BYTES, MAX_PREVIEW_LINES,
    MAX_WRITE_CONTENT_SIZE, TAIL_LINE_COUNT, WRITE_CHUNK_SIZE,
};

// LLM generation
pub use llm_generation::*;

// Context - re-export specific constants to avoid ambiguous glob re-exports with llm_generation
pub use context::{
    AGGRESSIVE_PRESERVE_RECENT_TURNS, CHAR_PER_TOKEN_APPROXIMATION, CHAR_PER_TOKEN_APPROX,
    CONTEXT_ERROR_RETRY_LIMIT, CONTEXT_UTILIZATION_PRECISION, DEFAULT_MAX_STRUCTURAL_DEPTH,
    DEFAULT_PRESERVE_RECENT_TOOLS, DEFAULT_PRESERVE_RECENT_TURNS, DEFAULT_SEMANTIC_CACHE_SCORE,
    DEFAULT_SEMANTIC_COMPRESSION_ENABLED, DEFAULT_SEMANTIC_SCORE, DEFAULT_TOKENS_FOR_PARTS,
    DEFAULT_TOOL_AWARE_RETENTION_ENABLED, DEFAULT_TRIM_TO_PERCENT, MAX_PRESERVE_RECENT_TOOLS,
    MAX_STRUCTURAL_DEPTH, MAX_TRIM_RATIO_PERCENT, MIN_PRESERVE_RECENT_TOOLS,
    MIN_PRESERVE_RECENT_TURNS, MIN_STRUCTURAL_DEPTH, MIN_TOKEN_COUNT, MIN_TRIM_RATIO_PERCENT,
    PERCENTAGE_CONVERSION_FACTOR, SEMANTIC_SCORE_SCALING_FACTOR, SEMANTIC_VALUE_PRECISION,
    SYSTEM_MESSAGE_SEMANTIC_SCORE, USER_MESSAGE_SEMANTIC_SCORE,
};

// Note: context::DEFAULT_MAX_TOKENS is intentionally not re-exported here
// to avoid conflict with llm_generation::DEFAULT_MAX_TOKENS.
// Use context::DEFAULT_MAX_TOKENS explicitly when needed for context window management.
