//! Tool executors organized by functionality
//!
//! Each executor module handles argument parsing, validation,
//! execution, and result formatting for specific tools.
//!
//! This module was refactored from a monolithic 2,689-line file
//! into focused modules averaging ~200-300 lines each.

// Re-export all executor modules
mod grep;
mod file_ops;
mod terminal;
mod pty;
mod patch;
mod web_fetch;
mod plan;
mod mcp;
mod code_execution;
mod skills;

// All modules add methods to ToolRegistry via impl blocks,
// so no explicit re-exports are needed. The methods become
// available when this module is imported.
