use super::config::SystemPromptConfig;
use super::context::PromptContext;
use super::templates::PromptTemplates;

/// System prompt generator
pub struct SystemPromptGenerator {
    config: SystemPromptConfig,
    context: PromptContext,
}

impl SystemPromptGenerator {
    pub fn new(config: SystemPromptConfig, context: PromptContext) -> Self {
        Self { config, context }
    }

    /// Generate complete system prompt
    pub fn generate(&self) -> String {
        let mut prompt_parts = Vec::new();

        // Base system prompt
        prompt_parts.push(PromptTemplates::base_system_prompt().to_string());

        // Custom instruction if provided
        if let Some(custom) = &self.config.custom_instruction {
            prompt_parts.push(custom.clone());
        }

        // Personality
        prompt_parts
            .push(PromptTemplates::personality_prompt(&self.config.personality).to_string());

        // Response style
        prompt_parts
            .push(PromptTemplates::response_style_prompt(&self.config.response_style).to_string());

        // Tool usage if enabled
        if self.config.include_tools && !self.context.available_tools.is_empty() {
            prompt_parts.push(PromptTemplates::tool_usage_prompt().to_string());
            prompt_parts.push(format!(
                "Available tools: {}",
                self.context.available_tools.join(", ")
            ));
        }

        // Workspace context if enabled
        if self.config.include_workspace {
            if let Some(workspace) = &self.context.workspace {
                prompt_parts.push(PromptTemplates::workspace_context_prompt().to_string());
                prompt_parts.push(format!("Current workspace: {}", workspace.display()));
            }

            if !self.context.languages.is_empty() {
                prompt_parts.push(format!(
                    "Detected languages: {}",
                    self.context.languages.join(", ")
                ));
            }

            if let Some(project_type) = &self.context.project_type {
                prompt_parts.push(format!("Project type: {}", project_type));
            }
        }

        // Safety guidelines
        prompt_parts.push(PromptTemplates::safety_guidelines_prompt().to_string());

        prompt_parts.join("\n\n")
    }
}

/// Generate system instruction with configuration (backward compatibility function)
pub fn generate_system_instruction_with_config(
    config: &SystemPromptConfig,
    context: &PromptContext,
) -> String {
    let generator = SystemPromptGenerator::new(config.clone(), context.clone());
    generator.generate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::{AgentPersonality, ResponseStyle};

    #[test]
    fn test_generator_basic() {
        let config = SystemPromptConfig::default();
        let context = PromptContext::default();
        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        assert!(!prompt.is_empty());
        assert!(prompt.len() > 100);
    }

    #[test]
    fn test_generator_with_custom_instruction() {
        let config = SystemPromptConfig {
            custom_instruction: Some("Custom instruction text".to_string()),
            ..Default::default()
        };
        let context = PromptContext::default();
        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        assert!(prompt.contains("Custom instruction text"));
    }

    #[test]
    fn test_generator_with_tools_disabled() {
        let config = SystemPromptConfig {
            include_tools: false,
            ..Default::default()
        };
        let mut context = PromptContext::default();
        context.add_tool("read_file".to_string());

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        // Should not include tool list when disabled
        assert!(!prompt.contains("Available tools:"));
    }

    #[test]
    fn test_generator_with_tools_enabled() {
        let config = SystemPromptConfig {
            include_tools: true,
            ..Default::default()
        };
        let mut context = PromptContext::default();
        context.add_tool("read_file".to_string());
        context.add_tool("write_file".to_string());

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        assert!(prompt.contains("Available tools:"));
        assert!(prompt.contains("read_file"));
        assert!(prompt.contains("write_file"));
    }

    #[test]
    fn test_generator_with_workspace_disabled() {
        let config = SystemPromptConfig {
            include_workspace: false,
            ..Default::default()
        };
        let context = PromptContext::from_workspace(PathBuf::from("/test"));

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        // Should not include workspace when disabled
        assert!(!prompt.contains("/test"));
    }

    #[test]
    fn test_generator_with_workspace_enabled() {
        let config = SystemPromptConfig {
            include_workspace: true,
            ..Default::default()
        };
        let mut context = PromptContext::from_workspace(PathBuf::from("/workspace"));
        context.add_language("Rust".to_string());
        context.set_project_type("CLI".to_string());

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        assert!(prompt.contains("/workspace"));
        assert!(prompt.contains("Rust"));
        assert!(prompt.contains("CLI"));
    }

    #[test]
    fn test_generator_with_different_personalities() {
        let personalities = vec![
            AgentPersonality::Professional,
            AgentPersonality::Friendly,
            AgentPersonality::Technical,
            AgentPersonality::Creative,
        ];

        for personality in personalities {
            let config = SystemPromptConfig {
                personality,
                ..Default::default()
            };
            let context = PromptContext::default();
            let generator = SystemPromptGenerator::new(config, context);
            let prompt = generator.generate();

            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn test_generator_with_different_styles() {
        let styles = vec![
            ResponseStyle::Concise,
            ResponseStyle::Detailed,
            ResponseStyle::Conversational,
            ResponseStyle::Technical,
        ];

        for style in styles {
            let config = SystemPromptConfig {
                response_style: style,
                ..Default::default()
            };
            let context = PromptContext::default();
            let generator = SystemPromptGenerator::new(config, context);
            let prompt = generator.generate();

            assert!(!prompt.is_empty());
        }
    }

    #[test]
    fn test_generate_system_instruction_with_config_helper() {
        let config = SystemPromptConfig::default();
        let context = PromptContext::default();
        let prompt = generate_system_instruction_with_config(&config, &context);

        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_generator_prompt_order() {
        let config = SystemPromptConfig {
            custom_instruction: Some("CUSTOM".to_string()),
            include_tools: true,
            include_workspace: true,
            ..Default::default()
        };
        let mut context = PromptContext::from_workspace(PathBuf::from("/test"));
        context.add_tool("tool1".to_string());

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        // Find positions
        let base_pos = prompt.find("helpful AI coding assistant");
        let custom_pos = prompt.find("CUSTOM");
        let tool_pos = prompt.find("Available tools:");
        let workspace_pos = prompt.find("Current workspace:");

        // Verify ordering (base should come before custom, etc.)
        assert!(base_pos.is_some());
        assert!(custom_pos.is_some());
        if let (Some(base), Some(custom)) = (base_pos, custom_pos) {
            assert!(base < custom, "Base prompt should come before custom");
        }
    }

    #[test]
    fn test_generator_empty_context() {
        let config = SystemPromptConfig::default();
        let context = PromptContext::default();
        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        // Should still generate valid prompt with empty context
        assert!(!prompt.is_empty());
        assert!(prompt.len() > 50);
    }

    #[test]
    fn test_generator_tools_only_when_non_empty() {
        let config = SystemPromptConfig {
            include_tools: true,
            ..Default::default()
        };
        let context = PromptContext::default(); // No tools added

        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();

        // Should not show "Available tools:" when list is empty
        assert!(!prompt.contains("Available tools:"));
    }
}
