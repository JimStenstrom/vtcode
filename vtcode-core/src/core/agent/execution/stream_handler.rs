//! Streaming response handling
//!
//! Note: Streaming support is temporarily disabled in the initial state machine refactoring
//! to reduce complexity. It will be re-enabled in a follow-up enhancement.

use super::{AgentState, ExecutionContext, Transition};
use anyhow::Result;

/// Handles streaming LLM responses
pub struct StreamHandler;

impl StreamHandler {
    pub fn new() -> Self {
        Self
    }

    // Streaming support will be added back in a follow-up enhancement
    // For now, this is a placeholder to maintain the module structure
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}
