//! Custom prompts example for vtcode-prompts
//!
//! This example demonstrates:
//! - Creating custom prompts from embedded content
//! - Using variable substitution ($1, $NAME, $ARGUMENTS)
//! - Parsing and expanding prompt invocations

use vtcode_prompts::{CustomPrompt, PromptInvocation};

fn main() -> Result<(), anyhow::Error> {
    println!("=== VTCode Prompts - Custom Prompts ===\n");

    // 1. Create a simple custom prompt
    println!("1. Simple Custom Prompt:");
    println!("{}", "=".repeat(50));
    let prompt_content = "Review the code in $FILE and check for $1.";
    let prompt =
        CustomPrompt::from_embedded("review", prompt_content).expect("Failed to create prompt");

    let invocation = PromptInvocation::parse("security-issues FILE=auth.rs")?;
    let expanded = prompt.expand(&invocation)?;
    println!("Template: {}", prompt_content);
    println!("Arguments: security-issues FILE=auth.rs");
    println!("Expanded: {}\n", expanded);

    // 2. Named arguments
    println!("2. Named Arguments:");
    println!("{}", "=".repeat(50));
    let prompt_content = "Analyze $TARGET for $ISSUE_TYPE issues. Focus: $FOCUS";
    let prompt = CustomPrompt::from_embedded("analyze", prompt_content)?;

    let invocation =
        PromptInvocation::parse("TARGET=api.rs ISSUE_TYPE=performance FOCUS=database-queries")?;
    let expanded = prompt.expand(&invocation)?;
    println!("Template: {}", prompt_content);
    println!("Arguments: TARGET=api.rs ISSUE_TYPE=performance FOCUS=database-queries");
    println!("Expanded: {}\n", expanded);

    // 3. All arguments placeholder
    println!("3. All Arguments Placeholder:");
    println!("{}", "=".repeat(50));
    let prompt_content = "Execute task: $ARGUMENTS\nAdditional context: $TASK";
    let prompt = CustomPrompt::from_embedded("task", prompt_content)?;

    let invocation = PromptInvocation::parse("refactor module extract utilities")?;
    let expanded = prompt.expand(&invocation)?;
    println!("Template: {}", prompt_content);
    println!("Arguments: refactor module extract utilities");
    println!("Expanded: {}\n", expanded);

    // 4. Mixed positional and named arguments
    println!("4. Mixed Arguments:");
    println!("{}", "=".repeat(50));
    let prompt_content = "Review $1 in $FILE. Priority: $2. Context: $ARGUMENTS";
    let prompt = CustomPrompt::from_embedded("review_mixed", prompt_content)?;

    let invocation = PromptInvocation::parse("security auth.rs high FILE=src/auth.rs")?;
    let expanded = prompt.expand(&invocation)?;
    println!("Template: {}", prompt_content);
    println!("Arguments: security auth.rs high FILE=src/auth.rs");
    println!("Expanded: {}\n", expanded);

    // 5. Escaping dollar signs
    println!("5. Escaping Dollar Signs:");
    println!("{}", "=".repeat(50));
    let prompt_content = "The price is $$50. Review $FILE for issues.";
    let prompt = CustomPrompt::from_embedded("price", prompt_content)?;

    let invocation = PromptInvocation::parse("FILE=billing.rs")?;
    let expanded = prompt.expand(&invocation)?;
    println!("Template: {}", prompt_content);
    println!("Arguments: FILE=billing.rs");
    println!("Expanded: {}\n", expanded);

    println!("✓ All custom prompt examples completed successfully!");

    Ok(())
}
