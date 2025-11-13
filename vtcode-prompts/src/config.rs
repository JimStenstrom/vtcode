use serde::{Deserialize, Serialize};

/// Configuration for system prompt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptConfig {
    /// Enable verbose mode
    pub verbose: bool,
    /// Include tool descriptions
    pub include_tools: bool,
    /// Include workspace context
    pub include_workspace: bool,
    /// Custom system instruction
    pub custom_instruction: Option<String>,
    /// Agent personality
    pub personality: AgentPersonality,
    /// Response style
    pub response_style: ResponseStyle,
}

impl Default for SystemPromptConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            include_tools: true,
            include_workspace: true,
            custom_instruction: None,
            personality: AgentPersonality::Professional,
            response_style: ResponseStyle::Concise,
        }
    }
}

/// Agent personality options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPersonality {
    Professional,
    Friendly,
    Technical,
    Creative,
}

/// Response style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStyle {
    Concise,
    Detailed,
    Conversational,
    Technical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SystemPromptConfig::default();
        assert!(!config.verbose);
        assert!(config.include_tools);
        assert!(config.include_workspace);
        assert!(config.custom_instruction.is_none());
        assert!(matches!(config.personality, AgentPersonality::Professional));
        assert!(matches!(config.response_style, ResponseStyle::Concise));
    }

    #[test]
    fn test_config_serialization() {
        let config = SystemPromptConfig {
            verbose: true,
            include_tools: false,
            include_workspace: true,
            custom_instruction: Some("Test instruction".to_string()),
            personality: AgentPersonality::Creative,
            response_style: ResponseStyle::Detailed,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("Test instruction"));
        assert!(json.contains("Creative"));
        assert!(json.contains("Detailed"));

        // Deserialize back
        let deserialized: SystemPromptConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.verbose, config.verbose);
        assert!(deserialized.custom_instruction.is_some());
    }

    #[test]
    fn test_all_personality_variants() {
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
            // Should be able to clone
            let _cloned = config.clone();
        }
    }

    #[test]
    fn test_all_response_style_variants() {
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
            // Should be able to clone
            let _cloned = config.clone();
        }
    }

    #[test]
    fn test_config_with_custom_instruction() {
        let config = SystemPromptConfig {
            custom_instruction: Some("Custom instruction here".to_string()),
            ..Default::default()
        };

        assert!(config.custom_instruction.is_some());
        assert_eq!(
            config.custom_instruction.as_ref().unwrap(),
            "Custom instruction here"
        );
    }

    #[test]
    fn test_config_builder_pattern() {
        // Test that we can build configs step by step
        let mut config = SystemPromptConfig::default();
        config.verbose = true;
        config.include_tools = false;
        config.custom_instruction = Some("Test".to_string());

        assert!(config.verbose);
        assert!(!config.include_tools);
    }
}
