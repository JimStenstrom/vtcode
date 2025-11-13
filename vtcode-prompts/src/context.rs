use std::path::PathBuf;

/// Context information for prompt generation
#[derive(Debug, Clone)]
pub struct PromptContext {
    /// Current workspace path
    pub workspace: Option<PathBuf>,
    /// Detected programming languages
    pub languages: Vec<String>,
    /// Project type (if detected)
    pub project_type: Option<String>,
    /// Available tools
    pub available_tools: Vec<String>,
    /// User preferences
    pub user_preferences: Option<UserPreferences>,
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            workspace: None,
            languages: Vec::new(),
            project_type: None,
            available_tools: Vec::new(),
            user_preferences: None,
        }
    }
}

/// User preferences for prompt customization
#[derive(Debug, Clone)]
pub struct UserPreferences {
    /// Preferred programming languages
    pub preferred_languages: Vec<String>,
    /// Coding style preferences
    pub coding_style: Option<String>,
    /// Framework preferences
    pub preferred_frameworks: Vec<String>,
}

impl PromptContext {
    /// Create context from workspace
    pub fn from_workspace(workspace: PathBuf) -> Self {
        Self {
            workspace: Some(workspace),
            ..Default::default()
        }
    }

    /// Add detected language
    pub fn add_language(&mut self, language: String) {
        if !self.languages.contains(&language) {
            self.languages.push(language);
        }
    }

    /// Set project type
    pub fn set_project_type(&mut self, project_type: String) {
        self.project_type = Some(project_type);
    }

    /// Add available tool
    pub fn add_tool(&mut self, tool: String) {
        if !self.available_tools.contains(&tool) {
            self.available_tools.push(tool);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_context() {
        let context = PromptContext::default();
        assert!(context.workspace.is_none());
        assert!(context.languages.is_empty());
        assert!(context.project_type.is_none());
        assert!(context.available_tools.is_empty());
        assert!(context.user_preferences.is_none());
    }

    #[test]
    fn test_context_from_workspace() {
        let workspace = PathBuf::from("/test/workspace");
        let context = PromptContext::from_workspace(workspace.clone());

        assert_eq!(context.workspace, Some(workspace));
        assert!(context.languages.is_empty());
        assert!(context.project_type.is_none());
    }

    #[test]
    fn test_add_language() {
        let mut context = PromptContext::default();

        context.add_language("Rust".to_string());
        assert_eq!(context.languages.len(), 1);
        assert_eq!(context.languages[0], "Rust");

        // Adding duplicate should be ignored
        context.add_language("Rust".to_string());
        assert_eq!(context.languages.len(), 1);

        context.add_language("Python".to_string());
        assert_eq!(context.languages.len(), 2);
    }

    #[test]
    fn test_add_tool() {
        let mut context = PromptContext::default();

        context.add_tool("read_file".to_string());
        assert_eq!(context.available_tools.len(), 1);
        assert_eq!(context.available_tools[0], "read_file");

        // Adding duplicate should be ignored
        context.add_tool("read_file".to_string());
        assert_eq!(context.available_tools.len(), 1);

        context.add_tool("write_file".to_string());
        assert_eq!(context.available_tools.len(), 2);
    }

    #[test]
    fn test_set_project_type() {
        let mut context = PromptContext::default();

        assert!(context.project_type.is_none());

        context.set_project_type("Web API".to_string());
        assert_eq!(context.project_type, Some("Web API".to_string()));

        // Can be changed
        context.set_project_type("CLI Tool".to_string());
        assert_eq!(context.project_type, Some("CLI Tool".to_string()));
    }

    #[test]
    fn test_context_builder_pattern() {
        let mut context = PromptContext::from_workspace(PathBuf::from("/workspace"));
        context.add_language("Rust".to_string());
        context.add_language("Python".to_string());
        context.add_tool("grep_file".to_string());
        context.add_tool("read_file".to_string());
        context.set_project_type("Backend".to_string());

        assert!(context.workspace.is_some());
        assert_eq!(context.languages.len(), 2);
        assert_eq!(context.available_tools.len(), 2);
        assert_eq!(context.project_type, Some("Backend".to_string()));
    }

    #[test]
    fn test_user_preferences() {
        let prefs = UserPreferences {
            preferred_languages: vec!["Rust".to_string(), "Go".to_string()],
            coding_style: Some("functional".to_string()),
            preferred_frameworks: vec!["Tokio".to_string()],
        };

        let mut context = PromptContext::default();
        context.user_preferences = Some(prefs);

        assert!(context.user_preferences.is_some());
        let prefs = context.user_preferences.as_ref().unwrap();
        assert_eq!(prefs.preferred_languages.len(), 2);
        assert_eq!(prefs.coding_style, Some("functional".to_string()));
    }

    #[test]
    fn test_context_cloning() {
        let mut context = PromptContext::from_workspace(PathBuf::from("/test"));
        context.add_language("Rust".to_string());

        let cloned = context.clone();
        assert_eq!(cloned.workspace, context.workspace);
        assert_eq!(cloned.languages, context.languages);
    }
}
