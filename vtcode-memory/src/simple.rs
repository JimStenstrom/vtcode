use crate::error::Result;
use crate::manager::MemoryManager;
use crate::session_log::SessionLog;
use crate::types::{ConversationTurn, MemoryConfig, MemoryStats, SessionMetadata, TurnSummary};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use vtcode_llm_types::Message;

/// Simple temporal-decay memory implementation
///
/// This memory manager uses a three-tier architecture:
/// 1. Working Memory (Hot) - Last N turns with full fidelity
/// 2. Recent Summaries (Warm) - Compressed summaries of older turns
/// 3. Session Logs (Cold) - On-disk persistence
pub struct SimpleMemory {
    /// Last N conversation turns with full message detail
    /// Default: 20 turns
    working_memory: VecDeque<ConversationTurn>,

    /// Summaries of older turns (beyond working memory)
    /// Default: Up to 100 summaries
    recent_summaries: Vec<TurnSummary>,

    /// Session metadata
    session_metadata: SessionMetadata,

    /// Configuration
    config: MemoryConfig,

    /// Background summarization queue
    summarization_queue: Arc<Mutex<VecDeque<ConversationTurn>>>,

    /// Total turns processed
    turn_counter: usize,
}

impl SimpleMemory {
    /// Create a new simple memory manager
    pub fn new(config: MemoryConfig, workspace: Option<PathBuf>) -> Self {
        debug!("Initializing SimpleMemory with config: {:?}", config);

        Self {
            working_memory: VecDeque::with_capacity(config.working_memory_limit),
            recent_summaries: Vec::with_capacity(config.summary_limit),
            session_metadata: SessionMetadata::new(workspace),
            config,
            summarization_queue: Arc::new(Mutex::new(VecDeque::new())),
            turn_counter: 0,
        }
    }

    /// Create a new simple memory manager with default config
    pub fn with_defaults(workspace: Option<PathBuf>) -> Self {
        Self::new(MemoryConfig::default(), workspace)
    }

    /// Check if a user message appears to be a historical query
    fn is_historical_query(msg: &str) -> bool {
        let msg_lower = msg.to_lowercase();
        let patterns = [
            "earlier",
            "before",
            "previously",
            "you said",
            "we discussed",
            "remember when",
            "last time",
            "ago",
        ];
        patterns.iter().any(|p| msg_lower.contains(p))
    }

    /// Simple fuzzy string matching for summary search
    fn fuzzy_match(text: &str, query: &str) -> f32 {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        // Exact substring match
        if text_lower.contains(&query_lower) {
            return 1.0;
        }

        // Word-level matching
        let text_words: std::collections::HashSet<_> =
            text_lower.split_whitespace().collect();
        let query_words: Vec<_> = query_lower.split_whitespace().collect();

        if query_words.is_empty() {
            return 0.0;
        }

        let matches = query_words
            .iter()
            .filter(|w| text_words.contains(*w))
            .count();

        matches as f32 / query_words.len() as f32
    }

    /// Search summaries for relevant context
    fn search_summaries(&self, query: &str) -> Vec<&TurnSummary> {
        let mut scored: Vec<_> = self
            .recent_summaries
            .iter()
            .map(|summary| {
                let match_score = Self::fuzzy_match(&summary.content, query);
                let relevance_score = summary.relevance_score(Utc::now());
                (summary, match_score * relevance_score)
            })
            .filter(|(_, score)| *score > 0.3) // Threshold
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top 5 summaries
        scored.into_iter().take(5).map(|(s, _)| s).collect()
    }

    /// Summarize a conversation turn (placeholder for LLM integration)
    ///
    /// TODO: Integrate with LLM provider for actual summarization
    /// For now, this creates a simple text-based summary
    async fn summarize_turn(&self, turn: &ConversationTurn) -> Result<TurnSummary> {
        debug!("Summarizing turn {}", turn.index);

        // Extract key information
        let mut content_parts = Vec::new();

        for message in &turn.messages {
            match message.role {
                vtcode_llm_types::MessageRole::User => {
                    let text = message.get_text_content();
                    if !text.is_empty() {
                        content_parts.push(format!("User: {}", Self::truncate(&text, 100)));
                    }
                }
                vtcode_llm_types::MessageRole::Assistant => {
                    let text = message.get_text_content();
                    if !text.is_empty() {
                        content_parts.push(format!("Assistant: {}", Self::truncate(&text, 100)));
                    }
                }
                vtcode_llm_types::MessageRole::Tool => {
                    if let Some(origin) = &message.origin_tool {
                        content_parts.push(format!("Tool: {}", origin));
                    }
                }
                _ => {}
            }
        }

        // Create summary
        let content = if content_parts.is_empty() {
            format!("Turn {}: No significant content", turn.index)
        } else {
            content_parts.join(". ")
        };

        let mut summary = TurnSummary::new(content, turn.index, turn.timestamp);
        summary.tools_used = turn.tools_used.clone();
        summary.files_modified = turn.files_modified.clone();

        Ok(summary)
    }

    /// Truncate text to max length
    fn truncate(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len.saturating_sub(3)])
        }
    }

    /// Process summarization queue in background
    async fn process_summarization_queue(&mut self) -> Result<()> {
        let mut queue = self.summarization_queue.lock().await;

        while let Some(turn) = queue.pop_front() {
            match self.summarize_turn(&turn).await {
                Ok(summary) => {
                    debug!("Created summary for turn {}", turn.index);
                    self.recent_summaries.push(summary);

                    // Trim summaries if needed
                    if self.recent_summaries.len() > self.config.summary_limit {
                        self.recent_summaries.remove(0);
                        debug!("Removed oldest summary to maintain limit");
                    }
                }
                Err(e) => {
                    warn!("Failed to summarize turn {}: {}", turn.index, e);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl MemoryManager for SimpleMemory {
    async fn add_turn(&mut self, mut turn: ConversationTurn) -> Result<()> {
        debug!("Adding turn {} to memory", turn.index);

        // Extract tools used
        turn.extract_tools_used();

        // Update session metadata
        self.session_metadata.total_turns += 1;
        self.session_metadata.total_tokens += turn.approximate_token_count();
        self.turn_counter += 1;

        // Add to working memory
        self.working_memory.push_back(turn.clone());

        // If working memory exceeds limit, move oldest to summarization queue
        if self.working_memory.len() > self.config.working_memory_limit {
            if let Some(old_turn) = self.working_memory.pop_front() {
                debug!("Working memory full, queuing turn {} for summarization", old_turn.index);

                if self.config.enable_background_summarization {
                    // Add to summarization queue for background processing
                    let mut queue = self.summarization_queue.lock().await;
                    queue.push_back(old_turn);
                } else {
                    // Summarize immediately (blocking)
                    let summary = self.summarize_turn(&old_turn).await?;
                    self.recent_summaries.push(summary);

                    // Trim summaries if needed
                    if self.recent_summaries.len() > self.config.summary_limit {
                        self.recent_summaries.remove(0);
                    }
                }
            }
        }

        Ok(())
    }

    fn build_context(&self, user_message: &str) -> Vec<Message> {
        debug!("Building context for user message");

        let mut context = Vec::new();

        // Include all working memory (full fidelity)
        for turn in &self.working_memory {
            context.extend(turn.messages.iter().cloned());
        }

        // If this looks like a historical query, include relevant summaries
        if Self::is_historical_query(user_message) {
            let relevant = self.search_summaries(user_message);
            debug!("Found {} relevant summaries for historical query", relevant.len());

            for summary in relevant.iter().take(3) {
                // Add summaries as system messages for context
                context.push(Message::system(format!(
                    "Earlier context (turn {}): {}",
                    summary.turn_range.0, summary.content
                )));
            }
        }

        context
    }

    async fn save(&self) -> Result<PathBuf> {
        debug!("Saving session to disk");

        // Create session log
        let mut log = SessionLog::new(self.session_metadata.clone());

        // Add all messages from working memory
        for turn in &self.working_memory {
            log.add_messages(turn.messages.clone());
        }

        // Add summaries
        for summary in &self.recent_summaries {
            log.add_summary(summary.clone());
        }

        // Complete metadata
        log.metadata.complete();

        // Save to disk
        log.save(&self.config.log_directory)
    }

    async fn load(path: &Path) -> Result<Self> {
        debug!("Loading session from: {:?}", path);

        let log = SessionLog::load(path)?;

        // Reconstruct memory from log
        let config = MemoryConfig::default();
        let mut memory = Self::new(config, log.metadata.workspace.clone());

        // Restore summaries
        memory.recent_summaries = log.summaries;

        // Restore working memory from most recent messages
        // Group messages into turns (this is a simplified version)
        let mut current_turn_messages = Vec::new();
        let mut turn_index = 0;

        for message in log.messages {
            current_turn_messages.push(message.clone());

            // Start new turn after assistant message or tool response
            if matches!(
                message.role,
                vtcode_llm_types::MessageRole::Assistant | vtcode_llm_types::MessageRole::Tool
            ) {
                if !current_turn_messages.is_empty() {
                    let turn = ConversationTurn::new(turn_index, current_turn_messages.clone());
                    memory.working_memory.push_back(turn);
                    current_turn_messages.clear();
                    turn_index += 1;
                }
            }
        }

        // Restore metadata
        memory.session_metadata = log.metadata;
        memory.turn_counter = memory.session_metadata.total_turns;

        Ok(memory)
    }

    fn stats(&self) -> MemoryStats {
        let session_age = Utc::now()
            .signed_duration_since(self.session_metadata.session_start)
            .to_std()
            .unwrap_or(std::time::Duration::ZERO);

        MemoryStats {
            working_memory_turns: self.working_memory.len(),
            summary_count: self.recent_summaries.len(),
            total_tokens_approximate: self.session_metadata.total_tokens,
            session_age,
        }
    }

    async fn process_background_tasks(&mut self) -> Result<()> {
        debug!("Processing background tasks");
        self.process_summarization_queue().await
    }

    fn clear(&mut self) {
        debug!("Clearing all memory");
        self.working_memory.clear();
        self.recent_summaries.clear();
        self.turn_counter = 0;
        self.session_metadata = SessionMetadata::new(self.session_metadata.workspace.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_memory_creation() {
        let memory = SimpleMemory::with_defaults(None);
        let stats = memory.stats();
        assert_eq!(stats.working_memory_turns, 0);
        assert_eq!(stats.summary_count, 0);
    }

    #[tokio::test]
    async fn test_add_turn() {
        let mut memory = SimpleMemory::with_defaults(None);
        let turn = ConversationTurn::new(
            1,
            vec![
                Message::user("Hello".to_string()),
                Message::assistant("Hi there!".to_string()),
            ],
        );

        memory.add_turn(turn).await.unwrap();
        assert_eq!(memory.stats().working_memory_turns, 1);
    }

    #[tokio::test]
    async fn test_working_memory_overflow() {
        let config = MemoryConfig {
            working_memory_limit: 3,
            enable_background_summarization: false,
            ..Default::default()
        };
        let mut memory = SimpleMemory::new(config, None);

        // Add 5 turns
        for i in 0..5 {
            let turn = ConversationTurn::new(
                i,
                vec![
                    Message::user(format!("Message {}", i)),
                    Message::assistant(format!("Response {}", i)),
                ],
            );
            memory.add_turn(turn).await.unwrap();
        }

        // Working memory should be capped at 3
        assert_eq!(memory.stats().working_memory_turns, 3);
        // Should have 2 summaries (turns 0 and 1)
        assert_eq!(memory.stats().summary_count, 2);
    }

    #[test]
    fn test_historical_query_detection() {
        assert!(SimpleMemory::is_historical_query("What did we discuss earlier?"));
        assert!(SimpleMemory::is_historical_query("You said something before about..."));
        assert!(!SimpleMemory::is_historical_query("What is Rust?"));
    }

    #[test]
    fn test_fuzzy_match() {
        assert_eq!(SimpleMemory::fuzzy_match("hello world", "hello"), 1.0);
        assert!(SimpleMemory::fuzzy_match("hello world", "world hello") > 0.9);
        assert!(SimpleMemory::fuzzy_match("hello", "world") < 0.1);
    }

    #[tokio::test]
    async fn test_build_context() {
        let mut memory = SimpleMemory::with_defaults(None);

        // Add a turn
        let turn = ConversationTurn::new(
            0,
            vec![
                Message::user("Hello".to_string()),
                Message::assistant("Hi!".to_string()),
            ],
        );
        memory.add_turn(turn).await.unwrap();

        let context = memory.build_context("What's next?");
        assert_eq!(context.len(), 2); // user + assistant message
    }
}
