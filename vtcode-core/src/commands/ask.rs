//! Ask command implementation - single prompt without tools

use crate::config::models::ModelId;
use crate::config::types::AgentConfig;
use crate::llm::make_client;
use crate::llm::provider::{LLMProvider, LLMRequest, Message};
use crate::prompts::generate_lightweight_instruction;
use anyhow::Result;

/// Handle the ask command - single prompt without tools
pub async fn handle_ask_command(config: AgentConfig, prompt: Vec<String>) -> Result<()> {
    let model_id = config
        .model
        .parse::<ModelId>()
        .map_err(|_| anyhow::anyhow!("Invalid model: {}", config.model))?;
    let mut client = make_client(config.api_key.clone(), model_id.clone());
    let prompt_text = prompt.join(" ");

    if config.verbose {
        println!("Sending prompt to {}: {}", config.model, prompt_text);
    }

    let lightweight_instruction = generate_lightweight_instruction();

    let request = LLMRequest {
        messages: vec![Message::user(prompt_text)],
        system_prompt: Some(lightweight_instruction),
        tools: None,
        model: model_id.as_str().to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
        tool_choice: None,
        parallel_tool_calls: None,
        parallel_tool_config: None,
        reasoning_effort: None,
    };

    let response = client.generate(request).await?;

    // Print the response content directly
    println!("{}", response.content.unwrap_or_else(|| "No response content".to_string()));

    Ok(())
}
