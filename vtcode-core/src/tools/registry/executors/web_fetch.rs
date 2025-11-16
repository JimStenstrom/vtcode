//! Web fetch executor
//!
//! Handles the web_fetch tool for fetching web content.

use crate::tools::traits::Tool;
use crate::tools::web_fetch::WebFetchTool;
use anyhow::Result;
use futures::future::BoxFuture;
use serde_json::Value;

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn web_fetch_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        // Get config from policy gateway or use defaults
        let mode = "restricted".to_string(); // Default mode
        let blocked_domains = Vec::new();
        let blocked_patterns = Vec::new();
        let allowed_domains = Vec::new();
        let strict_https_only = true;

        let tool = WebFetchTool::with_config(
            mode,
            blocked_domains,
            blocked_patterns,
            allowed_domains,
            strict_https_only,
        );
        Box::pin(async move { tool.execute(args).await })
    }
}
