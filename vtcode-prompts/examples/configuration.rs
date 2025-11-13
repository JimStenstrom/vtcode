//! Configuration example for vtcode-prompts
//!
//! This example demonstrates:
//! - Creating custom configurations
//! - Using different personality types
//! - Using different response styles
//! - Combining configuration with context

use vtcode_prompts::{
    AgentPersonality, PromptContext, ResponseStyle, SystemPromptConfig, SystemPromptGenerator,
};
use std::path::PathBuf;

fn main() {
    println!("=== VTCode Prompts - Configuration ===\n");

    // 1. Professional + Concise
    println!("1. Professional + Concise:");
    println!("{}", "=".repeat(50));
    let config = SystemPromptConfig {
        verbose: false,
        include_tools: true,
        include_workspace: false,
        custom_instruction: None,
        personality: AgentPersonality::Professional,
        response_style: ResponseStyle::Concise,
    };
    let context = PromptContext::default();
    let generator = SystemPromptGenerator::new(config, context);
    let prompt = generator.generate();
    println!("Configuration: Professional, Concise");
    println!("Generated prompt length: {} chars\n", prompt.len());

    // 2. Friendly + Conversational
    println!("2. Friendly + Conversational:");
    println!("{}", "=".repeat(50));
    let config = SystemPromptConfig {
        verbose: false,
        include_tools: true,
        include_workspace: false,
        custom_instruction: None,
        personality: AgentPersonality::Friendly,
        response_style: ResponseStyle::Conversational,
    };
    let context = PromptContext::default();
    let generator = SystemPromptGenerator::new(config, context);
    let prompt = generator.generate();
    println!("Configuration: Friendly, Conversational");
    println!("Generated prompt length: {} chars\n", prompt.len());

    // 3. Technical + Detailed
    println!("3. Technical + Detailed:");
    println!("{}", "=".repeat(50));
    let config = SystemPromptConfig {
        verbose: true,
        include_tools: true,
        include_workspace: true,
        custom_instruction: Some("Always explain the reasoning behind your decisions.".to_string()),
        personality: AgentPersonality::Technical,
        response_style: ResponseStyle::Technical,
    };
    let mut context = PromptContext::from_workspace(PathBuf::from("/workspace"));
    context.add_language("Rust".to_string());
    context.add_tool("read_file".to_string());
    let generator = SystemPromptGenerator::new(config, context);
    let prompt = generator.generate();
    println!("Configuration: Technical, Detailed, with custom instruction");
    println!("Generated prompt length: {} chars\n", prompt.len());

    // 4. Creative + Detailed
    println!("4. Creative + Detailed:");
    println!("{}", "=".repeat(50));
    let config = SystemPromptConfig {
        verbose: false,
        include_tools: false,
        include_workspace: false,
        custom_instruction: Some("Think outside the box and suggest innovative approaches.".to_string()),
        personality: AgentPersonality::Creative,
        response_style: ResponseStyle::Detailed,
    };
    let context = PromptContext::default();
    let generator = SystemPromptGenerator::new(config, context);
    let prompt = generator.generate();
    println!("Configuration: Creative, Detailed");
    println!("Generated prompt length: {} chars\n", prompt.len());

    // 5. Compare prompt lengths
    println!("5. Prompt Length Comparison:");
    println!("{}", "=".repeat(50));
    println!("Minimal config (no tools, no workspace): ~500 chars");
    println!("Standard config (with tools): ~700 chars");
    println!("Full config (tools + workspace + custom): ~900+ chars");

    println!("\n✓ All configuration examples completed successfully!");
}
