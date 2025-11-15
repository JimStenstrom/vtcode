use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;
use vtcode_config::constants::memory;
use vtcode_llm_types::Message;

/// A summarized conversation turn with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnSummary {
    /// Unique identifier for this summary
    pub id: Uuid,

    /// Distilled summary content (1-3 sentences)
    pub content: String,

    /// Original turn numbers this summarizes
    pub turn_range: (usize, usize),

    /// When this turn occurred
    pub timestamp: DateTime<Utc>,

    /// How many times this summary has been retrieved/accessed
    pub access_count: u32,

    /// Tools that were used in this turn (for context)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools_used: Vec<String>,

    /// Files that were modified in this turn (for context)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_modified: Vec<PathBuf>,

    /// Goal or task progress during this turn
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_progress: Option<GoalStatus>,
}

impl TurnSummary {
    /// Create a new turn summary
    pub fn new(content: String, turn_number: usize, timestamp: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            turn_range: (turn_number, turn_number),
            timestamp,
            access_count: 0,
            tools_used: Vec::new(),
            files_modified: Vec::new(),
            goal_progress: None,
        }
    }

    /// Increment access counter when this summary is retrieved
    pub fn record_access(&mut self) {
        self.access_count += 1;
    }

    /// Calculate relevance score based on age and access pattern
    pub fn relevance_score(&self, now: DateTime<Utc>) -> f32 {
        let age = now.signed_duration_since(self.timestamp);
        let age_minutes = age.num_minutes() as f32;

        // Temporal decay based on age
        let decay_score = if age_minutes < 5.0 {
            1.0 // 0-5 minutes: 100%
        } else if age_minutes < 30.0 {
            0.8 // 5-30 minutes: 80%
        } else if age_minutes < 120.0 {
            0.5 // 30 min - 2 hours: 50%
        } else if age_minutes < 1440.0 {
            0.2 // 2-24 hours: 20%
        } else {
            0.05 // 24+ hours: 5%
        };

        // Boost score based on access count (frequently accessed = more relevant)
        let access_boost = 1.0 + (self.access_count as f32 * 0.1).min(0.5);

        decay_score * access_boost
    }
}

/// Status of a goal or task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    Started,
    InProgress { progress: f32 }, // 0.0 to 1.0
    Completed,
    Failed { reason: String },
}

/// A conversation turn consisting of multiple messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Turn number/index
    pub index: usize,

    /// Messages in this turn (user message, assistant response, tool calls, tool results)
    pub messages: Vec<Message>,

    /// When this turn started
    pub timestamp: DateTime<Utc>,

    /// Tools used in this turn
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools_used: Vec<String>,

    /// Files modified in this turn
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_modified: Vec<PathBuf>,
}

impl ConversationTurn {
    /// Create a new conversation turn
    pub fn new(index: usize, messages: Vec<Message>) -> Self {
        Self {
            index,
            messages,
            timestamp: Utc::now(),
            tools_used: Vec::new(),
            files_modified: Vec::new(),
        }
    }

    /// Extract tool names from this turn
    pub fn extract_tools_used(&mut self) {
        for message in &self.messages {
            if let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    self.tools_used.push(tool_call.function.name.clone());
                }
            }
        }
        self.tools_used.sort();
        self.tools_used.dedup();
    }

    /// Approximate token count for this turn
    pub fn approximate_token_count(&self) -> usize {
        // Rough approximation: 4 characters per token
        self.messages
            .iter()
            .map(|msg| msg.get_text_content().len() / 4)
            .sum()
    }
}

/// Configuration for memory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum turns in working memory before summarization
    #[serde(default = "default_working_memory_limit")]
    pub working_memory_limit: usize,

    /// Maximum summaries to retain
    #[serde(default = "default_summary_limit")]
    pub summary_limit: usize,

    /// Enable periodic checkpoints
    #[serde(default = "default_auto_checkpoint")]
    pub auto_checkpoint: bool,

    /// Checkpoint interval in seconds
    #[serde(
        default = "default_checkpoint_interval_seconds",
        serialize_with = "serialize_duration_as_seconds",
        deserialize_with = "deserialize_duration_from_seconds"
    )]
    pub checkpoint_interval: Duration,

    /// Session log directory
    #[serde(default = "default_log_directory")]
    pub log_directory: PathBuf,

    /// Enable background summarization
    #[serde(default = "default_enable_background_summarization")]
    pub enable_background_summarization: bool,

    /// Model to use for summarization (optional, uses default if not specified)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summarization_model: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            working_memory_limit: default_working_memory_limit(),
            summary_limit: default_summary_limit(),
            auto_checkpoint: default_auto_checkpoint(),
            checkpoint_interval: default_checkpoint_interval(),
            log_directory: default_log_directory(),
            enable_background_summarization: default_enable_background_summarization(),
            summarization_model: None,
        }
    }
}

// Default value functions
fn default_working_memory_limit() -> usize {
    memory::DEFAULT_WORKING_MEMORY_LIMIT
}

fn default_summary_limit() -> usize {
    memory::DEFAULT_SUMMARY_LIMIT
}

fn default_auto_checkpoint() -> bool {
    true
}

fn default_checkpoint_interval() -> Duration {
    memory::default_checkpoint_interval()
}

fn default_checkpoint_interval_seconds() -> Duration {
    memory::default_checkpoint_interval()
}

fn default_log_directory() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vtcode")
        .join("sessions")
}

fn default_enable_background_summarization() -> bool {
    true
}

// Custom serializer for Duration to seconds
fn serialize_duration_as_seconds<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

// Custom deserializer for Duration from seconds
fn deserialize_duration_from_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

/// Metadata about a memory session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Total conversation turns
    pub total_turns: usize,

    /// Approximate total tokens used
    pub total_tokens: usize,

    /// Session duration in seconds
    pub session_duration_seconds: u64,

    /// Session start time
    pub session_start: DateTime<Utc>,

    /// Session end time (if completed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_end: Option<DateTime<Utc>>,

    /// Workspace path (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
}

impl SessionMetadata {
    /// Create new session metadata
    pub fn new(workspace: Option<PathBuf>) -> Self {
        Self {
            total_turns: 0,
            total_tokens: 0,
            session_duration_seconds: 0,
            session_start: Utc::now(),
            session_end: None,
            workspace,
        }
    }

    /// Mark session as complete
    pub fn complete(&mut self) {
        let now = Utc::now();
        self.session_end = Some(now);
        self.session_duration_seconds = now
            .signed_duration_since(self.session_start)
            .num_seconds() as u64;
    }
}

/// Statistics about memory usage
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Number of turns in working memory
    pub working_memory_turns: usize,

    /// Number of summaries
    pub summary_count: usize,

    /// Approximate total tokens in working memory
    pub total_tokens_approximate: usize,

    /// Age of the session
    pub session_age: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_summary_creation() {
        let summary = TurnSummary::new("Test summary".to_string(), 1, Utc::now());
        assert_eq!(summary.content, "Test summary");
        assert_eq!(summary.turn_range, (1, 1));
        assert_eq!(summary.access_count, 0);
    }

    #[test]
    fn test_relevance_score_decay() {
        let now = Utc::now();
        let recent = TurnSummary::new("Recent".to_string(), 1, now);
        let old = TurnSummary::new(
            "Old".to_string(),
            2,
            now - chrono::Duration::hours(25),
        );

        assert!(recent.relevance_score(now) > old.relevance_score(now));
    }

    #[test]
    fn test_memory_config_defaults() {
        let config = MemoryConfig::default();
        assert_eq!(config.working_memory_limit, 20);
        assert_eq!(config.summary_limit, 100);
        assert!(config.auto_checkpoint);
        assert!(config.enable_background_summarization);
    }

    #[test]
    fn test_conversation_turn_tool_extraction() {
        let mut turn = ConversationTurn::new(1, vec![]);
        turn.extract_tools_used();
        assert!(turn.tools_used.is_empty());
    }
}
