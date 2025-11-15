//! Agent execution state machine
//!
//! This module provides an explicit state machine pattern for agent task execution,
//! replacing deeply nested control flow with clear state transitions.

use crate::core::agent::task::TaskOutcome;
use crate::llm::provider::{LLMResponse, Message, ToolCall};
use anyhow::Result;
use serde_json::Value;

/// Tool execution result (Clone-able wrapper around Result)
#[derive(Debug, Clone)]
pub enum ToolResult {
    Success(Value),
    Error(String),
}

impl ToolResult {
    pub fn from_result(result: Result<Value>) -> Self {
        match result {
            Ok(value) => ToolResult::Success(value),
            Err(e) => ToolResult::Error(e.to_string()),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, ToolResult::Success(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, ToolResult::Error(_))
    }
}

// Re-export submodules
pub mod message_builder;
pub mod llm_caller;
pub mod tool_executor;
pub mod stream_handler;
pub mod loop_controller;
pub mod response_processor;
pub mod state_machine_executor;

pub use message_builder::MessageBuilder;
pub use llm_caller::LLMCaller;
pub use tool_executor::ToolExecutor;
pub use stream_handler::StreamHandler;
pub use loop_controller::LoopController;
pub use response_processor::{ResponseProcessor, ProcessedResponse};
pub use state_machine_executor::StateMachineExecutor;

/// States in the agent execution lifecycle
#[derive(Debug, Clone)]
pub enum AgentState {
    /// Initial state - preparing messages for LLM
    PreparingMessages,

    /// Calling LLM with prepared messages
    CallingLLM {
        messages: Vec<Message>,
        system_instruction: String,
        attempt: u32,
    },

    /// Processing complete LLM response (non-streaming)
    ProcessingResponse {
        response: LLMResponse,
    },

    /// Executing tool requested by LLM
    ExecutingTool {
        tool_name: String,
        tool_args: Value,
        call_id: String,
    },

    /// Recording tool result
    RecordingToolResult {
        tool_name: String,
        call_id: String,
        result: ToolResult,
    },

    /// Checking budget, limits, and completion status
    CheckingContinuation,

    /// Task completed successfully
    Completed {
        outcome: TaskOutcome,
    },

    /// Task failed with error
    Failed {
        error: String,
    },
}

/// State transition result
#[derive(Debug)]
pub enum Transition {
    /// Continue to next state
    Continue(AgentState),

    /// Terminal state reached (Completed/Failed)
    Terminal(TaskOutcome),
}

impl AgentState {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentState::Completed { .. } | AgentState::Failed { .. }
        )
    }

    /// Get a human-readable description of the current state
    pub fn description(&self) -> &'static str {
        match self {
            AgentState::PreparingMessages => "Preparing messages",
            AgentState::CallingLLM { .. } => "Calling LLM",
            AgentState::ProcessingResponse { .. } => "Processing response",
            AgentState::ExecutingTool { .. } => "Executing tool",
            AgentState::RecordingToolResult { .. } => "Recording tool result",
            AgentState::CheckingContinuation => "Checking continuation",
            AgentState::Completed { .. } => "Completed",
            AgentState::Failed { .. } => "Failed",
        }
    }
}

/// Context shared across all state handlers
pub struct ExecutionContext {
    /// Current turn number (0-indexed)
    pub turn_number: usize,

    /// Maximum allowed turns
    pub max_turns: usize,

    /// Whether streaming is supported
    pub supports_streaming: bool,

    /// Whether task has been marked as completed
    pub task_completed: bool,

    /// Consecutive tool loops counter
    pub consecutive_tool_loops: usize,

    /// Maximum allowed tool loops
    pub max_tool_loops: usize,

    /// Peak tool loop streak
    pub peak_tool_loop_streak: usize,
}

impl ExecutionContext {
    pub fn new(max_turns: usize, max_tool_loops: usize, supports_streaming: bool) -> Self {
        Self {
            turn_number: 0,
            max_turns,
            supports_streaming,
            task_completed: false,
            consecutive_tool_loops: 0,
            max_tool_loops,
            peak_tool_loop_streak: 0,
        }
    }

    pub fn increment_turn(&mut self) {
        self.turn_number += 1;
    }

    pub fn register_tool_loop(&mut self) -> usize {
        self.consecutive_tool_loops += 1;
        if self.consecutive_tool_loops > self.peak_tool_loop_streak {
            self.peak_tool_loop_streak = self.consecutive_tool_loops;
        }
        self.consecutive_tool_loops
    }

    pub fn reset_tool_loop_counter(&mut self) {
        self.consecutive_tool_loops = 0;
    }

    pub fn has_exceeded_turn_limit(&self) -> bool {
        self.turn_number >= self.max_turns
    }

    pub fn has_exceeded_tool_loop_limit(&self) -> bool {
        self.consecutive_tool_loops >= self.max_tool_loops
    }
}
