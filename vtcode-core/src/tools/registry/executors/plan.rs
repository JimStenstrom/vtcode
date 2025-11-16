//! Plan update executor
//!
//! Handles the update_plan tool for managing plan state.

use crate::tools::{PlanUpdateResult, UpdatePlanArgs};
use anyhow::{Context, Result};
use futures::future::BoxFuture;
use serde_json::Value;

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn update_plan_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let manager = self.inventory.plan_manager();
        Box::pin(async move {
            let parsed: UpdatePlanArgs = serde_json::from_value(args)
                .context("update_plan requires plan items with step and status")?;
            let updated_plan = manager
                .update_plan(parsed)
                .context("failed to update plan state")?;
            let payload = PlanUpdateResult::success(updated_plan);
            serde_json::to_value(payload).context("failed to serialize plan update result")
        })
    }
}
