use crate::error::Result;
use crate::types::{ConversationTurn, MemoryStats};
use async_trait::async_trait;
use std::path::Path;
use vtcode_llm_types::Message;

/// Core memory management interface
#[async_trait]
pub trait MemoryManager: Send + Sync {
    /// Add a completed conversation turn to memory
    async fn add_turn(&mut self, turn: ConversationTurn) -> Result<()>;

    /// Build context for the next LLM request
    fn build_context(&self, user_message: &str) -> Vec<Message>;

    /// Save current session to disk
    async fn save(&self) -> Result<std::path::PathBuf>;

    /// Load session from disk
    async fn load(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Get session statistics
    fn stats(&self) -> MemoryStats;

    /// Process any pending background tasks (summarization, compression, etc.)
    async fn process_background_tasks(&mut self) -> Result<()>;

    /// Clear all memory (for testing or reset)
    fn clear(&mut self);
}
