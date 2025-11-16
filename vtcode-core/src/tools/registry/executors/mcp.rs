//! MCP tool search executor
//!
//! Handles the search_tools tool for discovering and searching MCP tools.

use vtcode_mcp::{DetailLevel, ToolDiscovery};
use anyhow::{Context, Result, anyhow};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{Value, json};

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn search_tools_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let mcp_client = self.mcp_client.clone();
        Box::pin(async move {
            let mcp_client = match mcp_client {
                Some(client) => client,
                None => return Err(anyhow!("MCP client not configured")),
            };

            #[derive(Debug, Deserialize)]
            struct SearchArgs {
                keyword: String,
                #[serde(default)]
                detail_level: Option<String>,
            }

            let parsed: SearchArgs = serde_json::from_value(args)
                .context("search_tools requires 'keyword' and optional 'detail_level'")?;

            let detail_level = match parsed.detail_level.as_deref() {
                Some("name-only") | Some("name") => DetailLevel::NameOnly,
                Some("full") => DetailLevel::Full,
                Some("name-and-description") | Some("description") | None => {
                    DetailLevel::NameAndDescription
                }
                Some(invalid) => {
                    return Err(anyhow!(
                        "Invalid detail_level: '{}'. Must be one of: name-only, name-and-description, full",
                        invalid
                    ));
                }
            };

            let discovery = ToolDiscovery::new(mcp_client);
            let results = discovery
                .search_tools(&parsed.keyword, detail_level)
                .await
                .context("failed to search tools")?;

            if results.is_empty() {
                return Ok(json!({
                    "keyword": parsed.keyword,
                    "matched": 0,
                    "results": []
                }));
            }

            let tools_json: Vec<Value> = results.iter().map(|r| r.to_json(detail_level)).collect();

            Ok(json!({
                "keyword": parsed.keyword,
                "matched": results.len(),
                "detail_level": detail_level.as_str(),
                "results": tools_json
            }))
        })
    }
}
