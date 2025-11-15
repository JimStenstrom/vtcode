use crate::error::Result;
use crate::types::{SessionMetadata, TurnSummary};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use vtcode_llm_types::Message;

/// Persisted session log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    /// All messages from the session
    pub messages: Vec<Message>,

    /// Generated summaries
    pub summaries: Vec<TurnSummary>,

    /// Session timestamp
    pub timestamp: DateTime<Utc>,

    /// Session metadata
    pub metadata: SessionMetadata,
}

impl SessionLog {
    /// Create a new session log
    pub fn new(metadata: SessionMetadata) -> Self {
        Self {
            messages: Vec::new(),
            summaries: Vec::new(),
            timestamp: Utc::now(),
            metadata,
        }
    }

    /// Save session log to disk
    pub fn save(&self, directory: &Path) -> Result<PathBuf> {
        // Ensure directory exists
        fs::create_dir_all(directory)?;

        // Generate filename with timestamp
        let filename = format!("{}.json", self.timestamp.format("%Y%m%d_%H%M%S"));
        let path = directory.join(filename);

        // Write to file
        let file = File::create(&path)?;
        serde_json::to_writer_pretty(file, self)?;

        tracing::debug!("Session log saved to: {:?}", path);
        Ok(path)
    }

    /// Load session log from disk
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let log: SessionLog = serde_json::from_reader(file)?;
        tracing::debug!("Session log loaded from: {:?}", path);
        Ok(log)
    }

    /// List all session logs in a directory
    pub fn list_sessions(directory: &Path) -> Result<Vec<PathBuf>> {
        if !directory.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                sessions.push(path);
            }
        }

        // Sort by modification time (newest first)
        sessions.sort_by(|a, b| {
            let a_meta = fs::metadata(a).ok();
            let b_meta = fs::metadata(b).ok();
            match (a_meta, b_meta) {
                (Some(a), Some(b)) => b
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&a.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(sessions)
    }

    /// Add messages to the log
    pub fn add_messages(&mut self, messages: Vec<Message>) {
        self.messages.extend(messages);
        self.metadata.total_turns += 1;
    }

    /// Add a summary to the log
    pub fn add_summary(&mut self, summary: TurnSummary) {
        self.summaries.push(summary);
    }

    /// Get the most recent N sessions from a directory
    pub fn get_recent_sessions(directory: &Path, limit: usize) -> Result<Vec<SessionLog>> {
        let session_paths = Self::list_sessions(directory)?;
        let mut sessions = Vec::new();

        for path in session_paths.iter().take(limit) {
            match Self::load(path) {
                Ok(log) => sessions.push(log),
                Err(e) => {
                    tracing::warn!("Failed to load session from {:?}: {}", path, e);
                }
            }
        }

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_session_log_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = SessionMetadata::new(Some(PathBuf::from("/test/workspace")));
        let log = SessionLog::new(metadata);

        let path = log.save(temp_dir.path()).unwrap();
        assert!(path.exists());

        let loaded = SessionLog::load(&path).unwrap();
        assert_eq!(loaded.messages.len(), log.messages.len());
        assert_eq!(loaded.summaries.len(), log.summaries.len());
    }

    #[test]
    fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = SessionMetadata::new(None);

        // Create a few session logs
        for _ in 0..3 {
            let log = SessionLog::new(metadata.clone());
            log.save(temp_dir.path()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure different timestamps (filenames are second-precision)
        }

        let sessions = SessionLog::list_sessions(temp_dir.path()).unwrap();
        assert_eq!(sessions.len(), 3, "Expected 3 session files, found: {:?}", sessions);
    }

    #[test]
    fn test_add_messages() {
        let metadata = SessionMetadata::new(None);
        let mut log = SessionLog::new(metadata);

        let messages = vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi there!".to_string()),
        ];

        log.add_messages(messages);
        assert_eq!(log.messages.len(), 2);
        assert_eq!(log.metadata.total_turns, 1);
    }
}
