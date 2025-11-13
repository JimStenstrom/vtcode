use super::config::{AgentPersonality, ResponseStyle};

/// Prompt template collection
pub struct PromptTemplates;

impl PromptTemplates {
    /// Get base system prompt
    pub fn base_system_prompt() -> &'static str {
        "You are a helpful AI coding assistant. You provide accurate, helpful responses and can execute tools to assist with coding tasks."
    }

    /// Get personality-specific prompt addition
    pub fn personality_prompt(personality: &AgentPersonality) -> &'static str {
        match personality {
            AgentPersonality::Professional => {
                "Maintain a professional, focused approach to problem-solving."
            }
            AgentPersonality::Friendly => {
                "Be friendly and encouraging while helping with coding tasks."
            }
            AgentPersonality::Technical => {
                "Provide detailed technical explanations and focus on best practices."
            }
            AgentPersonality::Creative => {
                "Think creatively and suggest innovative solutions to problems."
            }
        }
    }

    /// Get response style prompt addition
    pub fn response_style_prompt(style: &ResponseStyle) -> &'static str {
        match style {
            ResponseStyle::Concise => "Keep responses concise and to the point.",
            ResponseStyle::Detailed => "Provide detailed explanations and comprehensive answers.",
            ResponseStyle::Conversational => {
                "Use a conversational tone and explain concepts clearly."
            }
            ResponseStyle::Technical => {
                "Focus on technical accuracy and include relevant implementation details."
            }
        }
    }

    /// Get tool usage prompt
    pub fn tool_usage_prompt() -> &'static str {
        "You have tools for files, search, and shell. Plan before calling tools; choose the most specific tool. Prefer small, targeted calls. Default to response_format='concise' and paginate long results with page/per_page (default per_page=50). If a tool truncates output or returns guidance, follow it. Use unambiguous args (e.g., path, max_results)."
    }

    /// Get workspace context prompt
    pub fn workspace_context_prompt() -> &'static str {
        "You are working within a specific workspace. Consider the project structure and existing code when making suggestions."
    }

    /// Get safety guidelines prompt
    pub fn safety_guidelines_prompt() -> &'static str {
        "Prioritize safety. Follow scoped permissions and caps applied by policy. Ask confirmation for destructive operations. If a tool errors, read the message and retry with corrected arguments."
    }

    /// Get pagination guidelines prompt
    pub fn pagination_guidelines_prompt() -> &'static str {
        "PAGINATION GUIDELINES: When working with large datasets, always use pagination to prevent timeouts and token overflow. Default per_page=50 for optimal performance. For edge cases: reduce per_page to 25 for very large directories, handle incomplete pages gracefully, and retry with smaller batches on API failures. Monitor 'has_more' flag and use 'page' parameter to continue pagination."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_system_prompt_not_empty() {
        let prompt = PromptTemplates::base_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.len() > 10);
    }

    #[test]
    fn test_all_personality_prompts() {
        let personalities = vec![
            AgentPersonality::Professional,
            AgentPersonality::Friendly,
            AgentPersonality::Technical,
            AgentPersonality::Creative,
        ];

        for personality in personalities {
            let prompt = PromptTemplates::personality_prompt(&personality);
            assert!(!prompt.is_empty());
            // Each should have distinct content
            assert!(prompt.len() > 10);
        }
    }

    #[test]
    fn test_all_response_style_prompts() {
        let styles = vec![
            ResponseStyle::Concise,
            ResponseStyle::Detailed,
            ResponseStyle::Conversational,
            ResponseStyle::Technical,
        ];

        for style in styles {
            let prompt = PromptTemplates::response_style_prompt(&style);
            assert!(!prompt.is_empty());
            assert!(prompt.len() > 10);
        }
    }

    #[test]
    fn test_personality_prompts_are_distinct() {
        let professional = PromptTemplates::personality_prompt(&AgentPersonality::Professional);
        let friendly = PromptTemplates::personality_prompt(&AgentPersonality::Friendly);
        let technical = PromptTemplates::personality_prompt(&AgentPersonality::Technical);
        let creative = PromptTemplates::personality_prompt(&AgentPersonality::Creative);

        // Each should be different
        assert_ne!(professional, friendly);
        assert_ne!(professional, technical);
        assert_ne!(professional, creative);
        assert_ne!(friendly, technical);
    }

    #[test]
    fn test_response_style_prompts_are_distinct() {
        let concise = PromptTemplates::response_style_prompt(&ResponseStyle::Concise);
        let detailed = PromptTemplates::response_style_prompt(&ResponseStyle::Detailed);
        let conversational = PromptTemplates::response_style_prompt(&ResponseStyle::Conversational);
        let technical = PromptTemplates::response_style_prompt(&ResponseStyle::Technical);

        // Each should be different
        assert_ne!(concise, detailed);
        assert_ne!(concise, conversational);
        assert_ne!(concise, technical);
        assert_ne!(detailed, conversational);
    }

    #[test]
    fn test_tool_usage_prompt() {
        let prompt = PromptTemplates::tool_usage_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("tool") || prompt.contains("Tool"));
    }

    #[test]
    fn test_workspace_context_prompt() {
        let prompt = PromptTemplates::workspace_context_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("workspace") || prompt.contains("Workspace"));
    }

    #[test]
    fn test_safety_guidelines_prompt() {
        let prompt = PromptTemplates::safety_guidelines_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("safety") || prompt.contains("Safety") || prompt.len() > 10);
    }

    #[test]
    fn test_pagination_guidelines_prompt() {
        let prompt = PromptTemplates::pagination_guidelines_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("pagination") || prompt.contains("PAGINATION"));
    }

    #[test]
    fn test_all_templates_are_static() {
        // Test that templates can be called multiple times and return same reference
        let base1 = PromptTemplates::base_system_prompt();
        let base2 = PromptTemplates::base_system_prompt();
        assert_eq!(base1.as_ptr(), base2.as_ptr()); // Same static string
    }
}
