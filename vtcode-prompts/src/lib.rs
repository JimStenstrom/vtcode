//! # vtcode-prompts - Prompt Generation and Management for VTCode
//!
//! `vtcode-prompts` provides the prompt generation and management infrastructure for VT Code.
//! It handles system prompts, custom user prompts, and built-in documentation.
//!
//! ## Highlights
//!
//! - **System Prompts**: Default, lightweight, and specialized system instructions for different use cases
//! - **Custom Prompts**: User-defined prompt templates with variable substitution
//! - **Built-in Documentation**: Embedded documentation for self-documentation queries
//! - **Configuration**: Flexible configuration for prompt generation and customization
//!
//! ## Architecture
//!
//! The crate is organized into several key modules:
//!
//! - `config`: Configuration types for prompt generation
//! - `context`: Context information for prompt customization
//! - `templates`: Reusable prompt templates
//! - `system`: System instruction generation
//! - `custom`: Custom prompt registry and management
//! - `generator`: High-level prompt generation interface
//!
//! ## Usage
//!
//! ```rust,ignore
//! use vtcode_prompts::{SystemPromptConfig, PromptContext, generate_system_instruction};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let config = SystemPromptConfig::default();
//!     let instruction = generate_system_instruction(&config).await;
//!     println!("{}", instruction);
//!     Ok(())
//! }
//! ```
//!
//! ## Custom Prompts
//!
//! Custom prompts allow users to define reusable prompt templates with variable substitution:
//!
//! ```markdown
//! ---
//! description: Review code for issues
//! argument_hint: FILE=<path>
//! ---
//! Please review $FILE and focus on $1. All arguments: $ARGUMENTS
//! ```
//!
//! ## Extensibility
//!
//! The crate is designed to be provider-agnostic, returning raw prompt strings that can be
//! used with any LLM provider. Provider-specific formatting should be done in the
//! provider integration layer.

pub mod config;
pub mod context;
pub mod custom;
pub mod generator;
pub mod system;
pub mod templates;

// Re-export main types for convenience
pub use config::{AgentPersonality, ResponseStyle, SystemPromptConfig};
pub use context::{PromptContext, UserPreferences};
pub use custom::{
    BuiltinDocs, CustomPrompt, CustomPromptRegistry, PromptInvocation,
};
pub use generator::{SystemPromptGenerator, generate_system_instruction_with_config};
pub use system::{
    compose_system_instruction_text, default_system_prompt, generate_lightweight_instruction,
    generate_specialized_instruction, generate_system_instruction,
    generate_system_instruction_with_config as generate_system_instruction_with_config_simple,
    generate_system_instruction_with_guidelines, read_system_prompt_from_md,
};
pub use templates::PromptTemplates;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Test that all public exports are accessible
        let _config = SystemPromptConfig::default();
        let _context = PromptContext::default();
        let _prompt = default_system_prompt();
    }

    #[tokio::test]
    async fn test_system_instruction_generation() {
        let config = SystemPromptConfig::default();
        let instruction = generate_system_instruction(&config).await;
        assert!(!instruction.is_empty());
        assert!(instruction.contains("VT Code") || instruction.contains("VTCode"));
    }

    #[test]
    fn test_prompt_context() {
        let mut context = PromptContext::default();
        context.add_language("Rust".to_string());
        context.add_language("Python".to_string());
        context.add_tool("read_file".to_string());

        assert_eq!(context.languages.len(), 2);
        assert_eq!(context.available_tools.len(), 1);
    }

    #[test]
    fn test_custom_prompt_invocation() {
        let invocation = PromptInvocation::parse("arg1 arg2 KEY=value").unwrap();
        assert_eq!(invocation.positional().len(), 2);
        assert!(invocation.named().contains_key("KEY"));
        assert_eq!(invocation.named().get("KEY").unwrap(), "value");
    }
}
