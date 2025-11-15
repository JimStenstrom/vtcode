//! Turn loop control and budget checking

use super::{AgentState, ExecutionContext, Transition};
use crate::core::agent::task::TaskOutcome;
use anyhow::Result;

/// Handles continuation logic, budget checks, and completion detection
pub struct LoopController;

impl LoopController {
    pub fn new() -> Self {
        Self
    }

    /// Check if execution should continue or terminate
    pub fn check_continuation(
        &self,
        had_tool_call: bool,
        response_text: &str,
        ctx: &mut ExecutionContext,
    ) -> Result<Transition> {
        // Increment turn counter
        ctx.increment_turn();

        // Check turn limit
        if ctx.has_exceeded_turn_limit() {
            return Ok(Transition::Terminal(TaskOutcome::TurnLimitReached));
        }

        // Check tool loop limit
        if ctx.has_exceeded_tool_loop_limit() {
            return Ok(Transition::Terminal(TaskOutcome::ToolLoopLimitReached));
        }

        // Check task completion
        if ctx.task_completed {
            return Ok(Transition::Terminal(TaskOutcome::Success));
        }

        // Detect completion from response text
        if self.detect_completion(response_text) {
            ctx.task_completed = true;
            return Ok(Transition::Terminal(TaskOutcome::Success));
        }

        // Update tool loop tracking
        if had_tool_call {
            ctx.register_tool_loop();
        } else {
            ctx.reset_tool_loop_counter();
        }

        // Decide whether to continue
        let should_continue = had_tool_call
            || (!ctx.task_completed && !ctx.has_exceeded_turn_limit());

        if should_continue {
            // Continue - prepare next messages
            Ok(Transition::Continue(AgentState::PreparingMessages))
        } else {
            // No more actions needed
            Ok(Transition::Terminal(TaskOutcome::StoppedNoAction))
        }
    }

    /// Detect task completion from response text
    pub fn detect_completion(&self, response_text: &str) -> bool {
        let response_lower = response_text.to_lowercase();

        // Comprehensive completion detection
        let completion_indicators = [
            "task completed",
            "task done",
            "finished",
            "complete",
            "summary",
            "i have successfully",
            "i've completed",
            "i have finished",
            "task accomplished",
            "mission accomplished",
            "objective achieved",
            "work is done",
            "all done",
            "completed successfully",
            "task execution complete",
            "operation finished",
        ];

        let has_indicator = completion_indicators
            .iter()
            .any(|&indicator| response_lower.contains(indicator));

        let has_explicit_completion = response_lower.contains("the task is complete")
            || response_lower.contains("task has been completed")
            || response_lower.contains("i am done")
            || response_lower.contains("that's all")
            || response_lower.contains("no more actions needed");

        has_indicator || has_explicit_completion
    }
}

impl Default for LoopController {
    fn default() -> Self {
        Self::new()
    }
}
