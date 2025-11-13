//! Stub types for prompt-related UI components

/// A custom prompt available to the user
#[derive(Debug, Clone)]
pub struct CustomPrompt {
    /// Prompt name/identifier
    pub name: String,
    /// Prompt description
    pub description: String,
    /// Prompt content
    pub content: String,
}

impl CustomPrompt {
    /// Create a new custom prompt
    pub fn new(name: String, description: String, content: String) -> Self {
        Self {
            name,
            description,
            content,
        }
    }
}

/// Registry for managing custom prompts
#[derive(Debug, Clone, Default)]
pub struct CustomPromptRegistry {
    prompts: Vec<CustomPrompt>,
}

impl CustomPromptRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a prompt to the registry
    pub fn add(&mut self, prompt: CustomPrompt) {
        self.prompts.push(prompt);
    }

    /// Get all prompts
    pub fn all(&self) -> &[CustomPrompt] {
        &self.prompts
    }

    /// Find a prompt by name
    pub fn find(&self, name: &str) -> Option<&CustomPrompt> {
        self.prompts.iter().find(|p| p.name == name)
    }

    /// Get number of prompts
    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    /// Get an iterator over prompts
    pub fn iter(&self) -> impl Iterator<Item = &CustomPrompt> {
        self.prompts.iter()
    }

    /// Check if the registry is enabled (has prompts)
    pub fn enabled(&self) -> bool {
        !self.is_empty()
    }

    /// Get built-in prompts (currently returns empty vec)
    pub fn builtin_prompts() -> Vec<CustomPrompt> {
        Vec::new()
    }
}
