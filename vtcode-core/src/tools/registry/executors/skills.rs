//! Skill management executors
//!
//! Handles save_skill, load_skill, list_skills, and search_skills tools
//! for managing reusable code skills.

use anyhow::{Context, Result};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{Value, json};

use super::super::ToolRegistry;

impl ToolRegistry {
    pub(in crate::tools::registry) fn save_skill_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let workspace_root = self.inventory.workspace_root().to_path_buf();
        Box::pin(async move {
            use crate::exec::{Skill, SkillManager, SkillMetadata};

            #[derive(Debug, Deserialize)]
            struct SaveSkillArgs {
                name: String,
                code: String,
                language: String,
                description: String,
                #[serde(default)]
                inputs: Option<Vec<serde_json::Value>>,
                output: String,
                #[serde(default)]
                tags: Option<Vec<String>>,
                #[serde(default)]
                examples: Option<Vec<String>>,
            }

            let parsed: SaveSkillArgs = serde_json::from_value(args)
                .context("save_skill requires name, code, language, description, and output")?;

            // Parse inputs
            let inputs = if let Some(input_values) = parsed.inputs {
                input_values
                    .iter()
                    .map(|v| {
                        let obj = v.as_object().context("input must be an object")?;
                        Ok(crate::exec::skill_manager::ParameterDoc {
                            name: obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .context("input.name required")?
                                .to_string(),
                            r#type: obj
                                .get("type")
                                .and_then(|v| v.as_str())
                                .context("input.type required")?
                                .to_string(),
                            description: obj
                                .get("description")
                                .and_then(|v| v.as_str())
                                .context("input.description required")?
                                .to_string(),
                            required: obj
                                .get("required")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        })
                    })
                    .collect::<Result<Vec<_>>>()
                    .context("failed to parse inputs")?
            } else {
                Vec::new()
            };

            let metadata = SkillMetadata {
                name: parsed.name.clone(),
                description: parsed.description,
                language: parsed.language,
                inputs,
                output: parsed.output,
                examples: parsed.examples.unwrap_or_default(),
                tags: parsed.tags.unwrap_or_default(),
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                tool_dependencies: vec![],
            };

            let skill = Skill {
                metadata,
                code: parsed.code,
            };

            let manager = SkillManager::new(&workspace_root);
            manager.save_skill(skill).await?;

            Ok(json!({
                "success": true,
                "message": format!("Skill '{}' saved successfully", parsed.name)
            }))
        })
    }

    pub(in crate::tools::registry) fn load_skill_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let workspace_root = self.inventory.workspace_root().to_path_buf();
        Box::pin(async move {
            use crate::exec::SkillManager;

            #[derive(Debug, Deserialize)]
            struct LoadSkillArgs {
                name: String,
            }

            let parsed: LoadSkillArgs =
                serde_json::from_value(args).context("load_skill requires 'name' field")?;

            let manager = SkillManager::new(&workspace_root);
            let skill = manager.load_skill(&parsed.name).await?;

            Ok(json!({
                "name": skill.metadata.name,
                "code": skill.code,
                "language": skill.metadata.language,
                "description": skill.metadata.description,
                "inputs": skill.metadata.inputs,
                "output": skill.metadata.output,
                "examples": skill.metadata.examples,
                "tags": skill.metadata.tags,
                "created_at": skill.metadata.created_at,
                "modified_at": skill.metadata.modified_at,
            }))
        })
    }

    pub(in crate::tools::registry) fn list_skills_executor(&mut self, _args: Value) -> BoxFuture<'_, Result<Value>> {
        let workspace_root = self.inventory.workspace_root().to_path_buf();
        Box::pin(async move {
            use crate::exec::SkillManager;

            let manager = SkillManager::new(&workspace_root);
            let skills = manager.list_skills().await?;

            Ok(json!({
                "skills": skills,
                "count": skills.len(),
            }))
        })
    }

    pub(in crate::tools::registry) fn search_skills_executor(&mut self, args: Value) -> BoxFuture<'_, Result<Value>> {
        let workspace_root = self.inventory.workspace_root().to_path_buf();
        Box::pin(async move {
            use crate::exec::SkillManager;

            #[derive(Debug, Deserialize)]
            struct SearchSkillsArgs {
                query: String,
            }

            let parsed: SearchSkillsArgs =
                serde_json::from_value(args).context("search_skills requires 'query' field")?;

            let manager = SkillManager::new(&workspace_root);
            let results = manager.search_skills(&parsed.query).await?;

            Ok(json!({
                "query": parsed.query,
                "results": results,
                "count": results.len(),
            }))
        })
    }
}
