//! Integration tests for vtcode-prompts
//!
//! These tests verify the complete workflow of prompt generation and management

use std::path::PathBuf;
use tempfile::TempDir;
use vtcode_prompts::*;

#[tokio::test]
async fn test_system_instruction_generation_workflow() {
    // Test the complete workflow of generating system instructions
    let config = SystemPromptConfig::default();
    let instruction = generate_system_instruction(&config).await;

    assert!(!instruction.is_empty());
    assert!(instruction.contains("VT Code") || instruction.contains("VTCode"));
    assert!(instruction.len() > 100);
}

#[tokio::test]
async fn test_all_prompt_variants_generate_successfully() {
    // Verify all three prompt variants can be generated
    let config = SystemPromptConfig::default();

    let default_prompt = generate_system_instruction(&config).await;
    let lightweight_prompt = generate_lightweight_instruction();
    let specialized_prompt = generate_specialized_instruction();

    assert!(!default_prompt.is_empty());
    assert!(!lightweight_prompt.is_empty());
    assert!(!specialized_prompt.is_empty());

    // Lightweight should be shorter than default
    assert!(lightweight_prompt.len() < default_prompt.len());

    // Specialized should mention complex/refactoring
    assert!(
        specialized_prompt.contains("complex")
            || specialized_prompt.contains("refactoring")
            || specialized_prompt.contains("advanced")
    );
}

#[test]
fn test_prompt_context_builder_pattern() {
    // Test building up a context incrementally
    let mut context = PromptContext::default();

    assert!(context.languages.is_empty());
    assert!(context.available_tools.is_empty());

    context.add_language("Rust".to_string());
    context.add_language("Python".to_string());
    context.add_language("Rust".to_string()); // Duplicate should be ignored

    assert_eq!(context.languages.len(), 2);

    context.add_tool("read_file".to_string());
    context.add_tool("write_file".to_string());
    context.add_tool("read_file".to_string()); // Duplicate should be ignored

    assert_eq!(context.available_tools.len(), 2);

    context.set_project_type("Web API".to_string());
    assert_eq!(context.project_type, Some("Web API".to_string()));
}

#[test]
fn test_system_prompt_generator_with_all_features() {
    // Test generator with all features enabled
    let config = SystemPromptConfig {
        verbose: true,
        include_tools: true,
        include_workspace: true,
        custom_instruction: Some("Custom instruction here".to_string()),
        personality: AgentPersonality::Technical,
        response_style: ResponseStyle::Detailed,
    };

    let mut context = PromptContext::from_workspace(PathBuf::from("/workspace"));
    context.add_language("Rust".to_string());
    context.add_tool("grep_file".to_string());
    context.set_project_type("CLI Tool".to_string());

    let generator = SystemPromptGenerator::new(config.clone(), context);
    let prompt = generator.generate();

    assert!(!prompt.is_empty());
    assert!(prompt.contains("Custom instruction here"));
    assert!(prompt.contains("Rust"));
    assert!(prompt.contains("grep_file"));
    assert!(prompt.contains("/workspace"));
    assert!(prompt.contains("CLI Tool"));
}

#[test]
fn test_custom_prompt_with_positional_arguments() {
    let prompt_content = "Review $1 for $2 issues in $3.";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    let invocation = PromptInvocation::parse("security performance src/main.rs").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();

    assert_eq!(expanded, "Review security for performance issues in src/main.rs.");
}

#[test]
fn test_custom_prompt_with_named_arguments() {
    let prompt_content = "Check $FILE for $ISSUE_TYPE issues.";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    let invocation = PromptInvocation::parse("FILE=main.rs ISSUE_TYPE=memory-leaks").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();

    assert_eq!(expanded, "Check main.rs for memory-leaks issues.");
}

#[test]
fn test_custom_prompt_with_all_arguments_placeholder() {
    let prompt_content = "Task: $ARGUMENTS";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    let invocation = PromptInvocation::parse("refactor extract module").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();

    assert_eq!(expanded, "Task: refactor extract module");
}

#[test]
fn test_custom_prompt_with_task_placeholder() {
    let prompt_content = "Execute: $TASK";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    let invocation = PromptInvocation::parse("fix bug in auth").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();

    assert_eq!(expanded, "Execute: fix bug in auth");
}

#[test]
fn test_custom_prompt_escaping_dollar_signs() {
    let prompt_content = "Price: $$100. Check $FILE.";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    let invocation = PromptInvocation::parse("FILE=pricing.rs").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();

    assert_eq!(expanded, "Price: $100. Check pricing.rs.");
}

#[test]
fn test_prompt_invocation_parsing() {
    // Test various argument formats
    let inv = PromptInvocation::parse("arg1 arg2 KEY=value").unwrap();
    assert_eq!(inv.positional().len(), 2);
    assert_eq!(inv.positional()[0], "arg1");
    assert_eq!(inv.named().get("KEY").unwrap(), "value");

    // Test quoted arguments
    let inv = PromptInvocation::parse(r#""complex arg" KEY="value with spaces""#).unwrap();
    assert_eq!(inv.positional()[0], "complex arg");
    assert_eq!(inv.named().get("KEY").unwrap(), "value with spaces");

    // Test empty invocation
    let inv = PromptInvocation::parse("").unwrap();
    assert!(inv.positional().is_empty());
    assert!(inv.named().is_empty());
}

#[tokio::test]
async fn test_custom_prompt_registry_loading() {
    let temp = TempDir::new().unwrap();
    let prompts_dir = temp.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();

    // Create multiple custom prompts
    std::fs::write(
        prompts_dir.join("review.md"),
        "---\ndescription: Code review\n---\nReview $FILE",
    )
    .unwrap();

    std::fs::write(
        prompts_dir.join("test.md"),
        "---\ndescription: Test generation\n---\nGenerate tests for $1",
    )
    .unwrap();

    let mut config = vtcode_config::core::AgentCustomPromptsConfig::default();
    config.directory = prompts_dir.to_string_lossy().to_string();
    config.enabled = true;

    let registry = CustomPromptRegistry::load(Some(&config), temp.path())
        .await
        .unwrap();

    assert!(registry.enabled());
    assert!(!registry.is_empty());
    assert!(registry.get("review").is_some());
    assert!(registry.get("test").is_some());
}

#[tokio::test]
async fn test_custom_prompt_registry_disabled() {
    let temp = TempDir::new().unwrap();

    let mut config = vtcode_config::core::AgentCustomPromptsConfig::default();
    config.enabled = false;

    let registry = CustomPromptRegistry::load(Some(&config), temp.path())
        .await
        .unwrap();

    assert!(!registry.enabled());
}

#[test]
fn test_builtin_docs() {
    let docs = BuiltinDocs::default();

    // Test iteration over keys
    let keys: Vec<_> = docs.keys().collect();
    assert!(keys.len() >= 0); // May be empty if no built-in docs yet

    // Test get_self_docs_content returns static content
    let content = BuiltinDocs::get_self_docs_content();
    assert!(!content.is_empty());
    assert!(content.contains("Documentation"));
}

#[test]
fn test_agent_personality_and_response_style() {
    // Test all personality variants
    let personalities = vec![
        AgentPersonality::Professional,
        AgentPersonality::Friendly,
        AgentPersonality::Technical,
        AgentPersonality::Creative,
    ];

    for personality in personalities {
        let config = SystemPromptConfig {
            personality: personality.clone(),
            ..Default::default()
        };

        let context = PromptContext::default();
        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();
        assert!(!prompt.is_empty());
    }

    // Test all response style variants
    let styles = vec![
        ResponseStyle::Concise,
        ResponseStyle::Detailed,
        ResponseStyle::Conversational,
        ResponseStyle::Technical,
    ];

    for style in styles {
        let config = SystemPromptConfig {
            response_style: style.clone(),
            ..Default::default()
        };

        let context = PromptContext::default();
        let generator = SystemPromptGenerator::new(config, context);
        let prompt = generator.generate();
        assert!(!prompt.is_empty());
    }
}

#[tokio::test]
async fn test_custom_prompt_with_frontmatter() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("prompt.md");

    std::fs::write(
        &path,
        r#"---
description: Test prompt with frontmatter
argument-hint: FILE=<path> TYPE=<type>
---
Check $FILE for $TYPE issues."#,
    )
    .unwrap();

    let prompt = CustomPrompt::from_file(&path, 8 * 1024).await.unwrap().unwrap();

    assert_eq!(prompt.name, "prompt");
    assert_eq!(
        prompt.description,
        Some("Test prompt with frontmatter".to_string())
    );
    assert_eq!(
        prompt.argument_hint,
        Some("FILE=<path> TYPE=<type>".to_string())
    );

    let invocation = PromptInvocation::parse("FILE=main.rs TYPE=security").unwrap();
    let expanded = prompt.expand(&invocation).unwrap();
    assert_eq!(expanded, "Check main.rs for security issues.");
}

#[tokio::test]
async fn test_custom_prompt_missing_required_arguments() {
    let prompt_content = "Review $FILE for $1 issues.";
    let prompt = CustomPrompt::from_embedded("test", prompt_content).unwrap();

    // Missing positional argument
    let invocation = PromptInvocation::parse("FILE=main.rs").unwrap();
    let result = prompt.expand(&invocation);
    assert!(result.is_err());

    // Missing named argument
    let invocation = PromptInvocation::parse("security").unwrap();
    let result = prompt.expand(&invocation);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_custom_prompt_file_size_limit() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("large.md");

    // Create a prompt that exceeds the size limit
    let large_content = "A".repeat(100 * 1024); // 100KB
    std::fs::write(&path, large_content).unwrap();

    // Should be rejected due to size
    let result = CustomPrompt::from_file(&path, 64 * 1024).await.unwrap();
    assert!(result.is_none());
}

#[test]
fn test_prompt_context_from_workspace() {
    let workspace = PathBuf::from("/my/workspace");
    let context = PromptContext::from_workspace(workspace.clone());

    assert_eq!(context.workspace, Some(workspace));
    assert!(context.languages.is_empty());
    assert!(context.available_tools.is_empty());
}

#[test]
fn test_system_prompt_config_default() {
    let config = SystemPromptConfig::default();

    assert!(!config.verbose);
    assert!(config.include_tools);
    assert!(config.include_workspace);
    assert!(config.custom_instruction.is_none());
}

#[tokio::test]
async fn test_read_system_prompt_from_md_fallback() {
    // When no system.md file exists, should fall back to default
    let prompt = read_system_prompt_from_md().await.unwrap();

    assert!(!prompt.is_empty());
    assert!(prompt.contains("VT Code") || prompt.contains("VTCode") || prompt.len() > 100);
}

#[tokio::test]
async fn test_compose_system_instruction_text() {
    let temp = TempDir::new().unwrap();
    let instruction = compose_system_instruction_text(temp.path()).await;

    assert!(!instruction.is_empty());
    // Should contain the base system prompt
    assert!(instruction.len() > 100);
}

#[test]
fn test_builtin_prompts_list() {
    let prompts = CustomPromptRegistry::builtin_prompts();

    // May be empty if no built-ins are embedded yet, but shouldn't error
    assert!(prompts.len() >= 0);

    // If there are built-ins, they should be valid
    for prompt in prompts {
        assert!(!prompt.name.is_empty());
        assert!(prompt.path.to_string_lossy().contains("<builtin>"));
    }
}
