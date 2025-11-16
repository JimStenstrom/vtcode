//! Grep file search executor
//!
//! Handles the grep_file tool for searching file contents using pattern matching.

use crate::tools::grep_file::GrepSearchInput;
use anyhow::{Context, Result};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{Value, json};

use super::super::{ToolRegistry, validation};

impl ToolRegistry {
    pub(in crate::tools::registry) fn grep_file_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let manager = self.inventory.grep_file_manager();
        Box::pin(async move {
            #[derive(Debug, Deserialize)]
            struct GrepArgs {
                pattern: String,
                #[serde(default = "default_grep_path", alias = "root", alias = "search_path")]
                path: String,
                #[serde(default)]
                max_results: Option<usize>,
                #[serde(default)]
                case_sensitive: Option<bool>,
                #[serde(default)]
                literal: Option<bool>,
                #[serde(default)]
                glob_pattern: Option<String>,
                #[serde(default)]
                context_lines: Option<usize>,
                #[serde(default)]
                include_hidden: Option<bool>,
                #[serde(default)]
                respect_ignore_files: Option<bool>,
                #[serde(default)]
                max_file_size: Option<usize>,
                #[serde(default)]
                search_hidden: Option<bool>,
                #[serde(default)]
                search_binary: Option<bool>,
                #[serde(default)]
                files_with_matches: Option<bool>,
                #[serde(default)]
                type_pattern: Option<String>,
                #[serde(default)]
                invert_match: Option<bool>,
                #[serde(default)]
                word_boundaries: Option<bool>,
                #[serde(default)]
                line_number: Option<bool>,
                #[serde(default)]
                column: Option<bool>,
                #[serde(default)]
                only_matching: Option<bool>,
                #[serde(default)]
                trim: Option<bool>,
            }

            fn default_grep_path() -> String {
                ".".to_string()
            }

            let payload: GrepArgs =
                serde_json::from_value(args).context("grep_file requires a 'pattern' field")?;

            // Validate all parameters using the validation module
            validation::validate_grep_params(
                &payload.path,
                payload.glob_pattern.as_deref(),
                payload.type_pattern.as_deref(),
                payload.max_results,
                payload.max_file_size,
                payload.context_lines,
            )
            .context("grep_file parameter validation failed")?;

            let input = GrepSearchInput {
                pattern: payload.pattern.clone(),
                path: payload.path.clone(),
                case_sensitive: payload.case_sensitive,
                literal: payload.literal,
                glob_pattern: payload.glob_pattern,
                context_lines: payload.context_lines,
                include_hidden: payload.include_hidden,
                max_results: payload.max_results,
                respect_ignore_files: payload.respect_ignore_files,
                max_file_size: payload.max_file_size,
                search_hidden: payload.search_hidden,
                search_binary: payload.search_binary,
                files_with_matches: payload.files_with_matches,
                type_pattern: payload.type_pattern,
                invert_match: payload.invert_match,
                word_boundaries: payload.word_boundaries,
                line_number: payload.line_number,
                column: payload.column,
                only_matching: payload.only_matching,
                trim: payload.trim,
            };

            let result = manager
                .perform_search(input)
                .await
                .with_context(|| format!("grep_file failed for pattern '{}'", payload.pattern))?;

            Ok(json!({
                "success": true,
                "query": result.query,
                "matches": result.matches,
            }))
        })
    }
}
