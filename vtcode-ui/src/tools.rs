//! Stub types for tool-related UI components
//! These are simplified versions for UI rendering purposes

use serde::{Deserialize, Serialize};

/// Status of a plan step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently executing
    InProgress,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

impl StepStatus {
    /// Get a checkbox character representing this status
    pub fn checkbox(&self) -> &'static str {
        match self {
            StepStatus::Pending => "☐",
            StepStatus::InProgress => "⧗",
            StepStatus::Completed => "☑",
            StepStatus::Failed => "☒",
            StepStatus::Skipped => "⊗",
        }
    }
}

/// A step in an execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step identifier
    pub id: String,
    /// Step number/index
    pub step: usize,
    /// Step description
    pub description: String,
    /// Current status
    pub status: StepStatus,
    /// Optional error message if failed
    pub error: Option<String>,
}

/// Overall completion state of a plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanCompletionState {
    /// Plan is empty (no steps)
    Empty,
    /// Plan is not started
    NotStarted,
    /// Plan is in progress
    InProgress,
    /// Plan completed successfully
    Completed,
    /// Plan is done (alias for Completed)
    Done,
    /// Plan failed
    Failed,
    /// Plan was cancelled
    Cancelled,
}

/// Summary metadata for a task plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    /// Total number of steps
    pub total_steps: usize,
    /// Number of completed steps
    pub completed_steps: usize,
    /// Overall completion status
    pub status: PlanCompletionState,
}

/// A complete task plan with multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    /// Plan identifier
    pub id: String,
    /// Plan summary metadata
    pub summary: PlanSummary,
    /// Plan description
    pub description: String,
    /// Steps in the plan
    pub steps: Vec<PlanStep>,
    /// Overall completion state
    pub state: PlanCompletionState,
}

impl Default for TaskPlan {
    fn default() -> Self {
        Self {
            id: String::new(),
            summary: PlanSummary {
                total_steps: 0,
                completed_steps: 0,
                status: PlanCompletionState::Empty,
            },
            description: String::new(),
            steps: Vec::new(),
            state: PlanCompletionState::Empty,
        }
    }
}
