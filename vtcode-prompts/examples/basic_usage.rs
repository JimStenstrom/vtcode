//! Basic usage example for vtcode-prompts
//!
//! This example demonstrates:
//! - Generating system instructions with default configuration
//! - Using different prompt variants (default, lightweight, specialized)
//! - Creating and using prompt context

use vtcode_prompts::{
    PromptContext, SystemPromptConfig, generate_lightweight_instruction,
    generate_specialized_instruction, generate_system_instruction,
    generate_system_instruction_with_config,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("=== VTCode Prompts - Basic Usage ===\n");

    // 1. Default system instruction
    println!("1. Default System Instruction:");
    println!("{}", "=".repeat(50));
    let config = SystemPromptConfig::default();
    let instruction = generate_system_instruction(&config).await;
    println!("{}\n", &instruction[..200.min(instruction.len())]);
    println!("... (truncated, {} total chars)\n", instruction.len());

    // 2. Lightweight instruction
    println!("2. Lightweight Instruction:");
    println!("{}", "=".repeat(50));
    let lightweight = generate_lightweight_instruction();
    println!("{}\n", &lightweight[..200.min(lightweight.len())]);
    println!("... (truncated, {} total chars)\n", lightweight.len());

    // 3. Specialized instruction
    println!("3. Specialized Instruction:");
    println!("{}", "=".repeat(50));
    let specialized = generate_specialized_instruction();
    println!("{}\n", &specialized[..200.min(specialized.len())]);
    println!("... (truncated, {} total chars)\n", specialized.len());

    // 4. Instruction with context
    println!("4. Instruction with Context:");
    println!("{}", "=".repeat(50));
    let mut context = PromptContext::default();
    context.add_language("Rust".to_string());
    context.add_language("Python".to_string());
    context.add_tool("read_file".to_string());
    context.add_tool("write_file".to_string());
    context.set_project_type("Web API".to_string());

    let config = SystemPromptConfig::default();
    let instruction_with_context = generate_system_instruction_with_config(&config, &context);
    println!("Generated instruction with context:");
    println!("- Languages: {:?}", context.languages);
    println!("- Tools: {:?}", context.available_tools);
    println!("- Project type: {:?}", context.project_type);
    println!(
        "- Instruction length: {} chars\n",
        instruction_with_context.len()
    );

    println!("✓ All examples completed successfully!");

    Ok(())
}
