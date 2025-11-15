//! Message preparation for LLM requests

use super::{AgentState, ExecutionContext, Transition};
use crate::core::agent::conversation::build_messages_from_conversation;
use crate::llm::provider::Message;
use anyhow::Result;
use vtcode_llm_gemini::Content;

/// Handles message preparation for LLM requests
pub struct MessageBuilder;

impl MessageBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Prepare messages from conversation history for the next LLM call
    pub fn prepare_messages(
        &self,
        system_instruction: &str,
        conversation: &[Content],
        _ctx: &ExecutionContext,
    ) -> Result<Transition> {
        // Build messages from conversation
        let messages = build_messages_from_conversation(system_instruction, conversation);

        // Validate messages
        self.validate_messages(&messages)?;

        // Transition to calling LLM
        Ok(Transition::Continue(AgentState::CallingLLM {
            messages,
            system_instruction: system_instruction.to_string(),
            attempt: 1,
        }))
    }

    /// Validate that messages are ready for LLM call
    fn validate_messages(&self, messages: &[Message]) -> Result<()> {
        if messages.is_empty() {
            anyhow::bail!("No messages to send to LLM");
        }
        Ok(())
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
