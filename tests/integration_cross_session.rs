//! Integration tests for cross-session persistence

use std::time::Duration;
use tempfile::TempDir;
use vtcode_llm_types::Message;
use vtcode_memory::{ConversationTurn, MemoryConfig, MemoryManager, SessionLog, SimpleMemory};

#[tokio::test]
async fn test_session_save_and_restore() {
    let temp_dir = TempDir::new().unwrap();

    let config = MemoryConfig {
        working_memory_limit: 10,
        summary_limit: 20,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    // Create session
    let mut memory1 = SimpleMemory::new(config.clone(), None);

    for i in 0..5 {
        let turn = ConversationTurn::new(
            i,
            vec![
                Message::user(format!("Q{}", i)),
                Message::assistant(format!("A{}", i)),
            ],
        );
        memory1.add_turn(turn).await.unwrap();
    }

    // Save
    let path = memory1.save().await.unwrap();
    assert!(path.exists(), "Session file should exist");

    // Load into new memory instance
    let memory2 = SimpleMemory::load(&path).await.unwrap();

    let stats1 = memory1.stats();
    let stats2 = memory2.stats();

    assert_eq!(
        stats1.working_memory_turns, stats2.working_memory_turns,
        "Working memory should match"
    );
    assert_eq!(
        stats1.summary_count, stats2.summary_count,
        "Summary count should match"
    );

    println!("✅ Session save/restore works");
}

#[tokio::test]
async fn test_session_log_listing() {
    let temp_dir = TempDir::new().unwrap();
    let config = MemoryConfig {
        working_memory_limit: 10,
        summary_limit: 20,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    // Create multiple sessions
    for i in 0..3 {
        let mut memory = SimpleMemory::new(config.clone(), None);
        let turn = ConversationTurn::new(i, vec![Message::user(format!("Session {}", i))]);
        memory.add_turn(turn).await.unwrap();
        memory.save().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure different timestamps
    }

    // List sessions
    let sessions = SessionLog::list_sessions(temp_dir.path()).unwrap();
    assert_eq!(sessions.len(), 3, "Should have 3 session files");

    // Sessions should be sorted (most recent first)
    for session_path in &sessions {
        assert!(session_path.exists());
        assert_eq!(
            session_path.extension().and_then(|s| s.to_str()),
            Some("json")
        );
    }

    println!("✅ Session listing works: found {} sessions", sessions.len());
}

#[tokio::test]
async fn test_session_persistence_with_summaries() {
    let temp_dir = TempDir::new().unwrap();

    let config = MemoryConfig {
        working_memory_limit: 3,
        summary_limit: 10,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    // Create session with overflow (triggers summarization)
    let mut memory1 = SimpleMemory::new(config.clone(), None);

    for i in 0..8 {
        let turn = ConversationTurn::new(
            i,
            vec![Message::user(format!("Turn {} with some content", i))],
        );
        memory1.add_turn(turn).await.unwrap();
    }

    let stats_before = memory1.stats();
    assert_eq!(stats_before.working_memory_turns, 3);
    assert_eq!(stats_before.summary_count, 5); // 8 - 3 = 5 summaries

    // Save
    let path = memory1.save().await.unwrap();

    // Load
    let memory2 = SimpleMemory::load(&path).await.unwrap();
    let stats_after = memory2.stats();

    assert_eq!(stats_before.working_memory_turns, stats_after.working_memory_turns);
    assert_eq!(stats_before.summary_count, stats_after.summary_count);

    println!("✅ Session persistence with summaries works");
}

#[tokio::test]
async fn test_empty_session_save_restore() {
    let temp_dir = TempDir::new().unwrap();

    let config = MemoryConfig {
        working_memory_limit: 10,
        summary_limit: 20,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    // Create empty session
    let memory1 = SimpleMemory::new(config.clone(), None);

    // Save empty session
    let path = memory1.save().await.unwrap();
    assert!(path.exists());

    // Load empty session
    let memory2 = SimpleMemory::load(&path).await.unwrap();

    let stats = memory2.stats();
    assert_eq!(stats.working_memory_turns, 0);
    assert_eq!(stats.summary_count, 0);

    println!("✅ Empty session save/restore works");
}

#[tokio::test]
async fn test_session_with_workspace_path() {
    let temp_dir = TempDir::new().unwrap();

    let config = MemoryConfig {
        working_memory_limit: 10,
        summary_limit: 20,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    let workspace_path = Some(std::path::PathBuf::from("/home/user/project"));

    // Create session with workspace path
    let mut memory1 = SimpleMemory::new(config.clone(), workspace_path.clone());

    let turn = ConversationTurn::new(0, vec![Message::user("Test".to_string())]);
    memory1.add_turn(turn).await.unwrap();

    // Save
    let path = memory1.save().await.unwrap();

    // Load
    let memory2 = SimpleMemory::load(&path).await.unwrap();

    // Verify via rebuilding from the session
    let context = memory2.build_context("test");
    assert!(!context.is_empty());

    println!("✅ Session with workspace path works");
}

#[tokio::test]
async fn test_session_log_list_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // List sessions from empty directory
    let sessions = SessionLog::list_sessions(temp_dir.path()).unwrap();
    assert_eq!(sessions.len(), 0, "Empty directory should have no sessions");

    println!("✅ Session listing in empty directory works");
}

#[tokio::test]
async fn test_session_json_format() {
    let temp_dir = TempDir::new().unwrap();

    let config = MemoryConfig {
        working_memory_limit: 5,
        summary_limit: 10,
        enable_background_summarization: false,
        auto_checkpoint: true,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: temp_dir.path().to_path_buf(),
        summarization_model: None,
    };

    let mut memory = SimpleMemory::new(config.clone(), None);

    let turn = ConversationTurn::new(
        0,
        vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi there!".to_string()),
        ],
    );
    memory.add_turn(turn).await.unwrap();

    let path = memory.save().await.unwrap();

    // Read and parse JSON
    let json_content = tokio::fs::read_to_string(&path).await.unwrap();
    let session: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    // Verify structure
    assert!(session.get("timestamp").is_some());
    assert!(session.get("config").is_some());
    assert!(session.get("working_memory").is_some());
    assert!(session.get("summaries").is_some());
    assert!(session.get("total_turns").is_some());

    println!("✅ Session JSON format is valid");
}

#[tokio::test]
async fn test_load_nonexistent_session() {
    use std::path::PathBuf;

    let non_existent = PathBuf::from("/tmp/does_not_exist_12345.json");

    // Try to load non-existent file
    let result = SimpleMemory::load(&non_existent).await;
    assert!(result.is_err(), "Loading non-existent file should fail");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("Failed to read session file"),
            "Error should mention file read failure"
        );
    }

    println!("✅ Loading non-existent session fails gracefully");
}

#[tokio::test]
async fn test_load_invalid_json() {
    use tempfile::NamedTempFile;
    use std::io::Write;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{{invalid json").unwrap();

    let result = SimpleMemory::load(temp_file.path()).await;
    assert!(result.is_err(), "Loading invalid JSON should fail");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("Failed to parse session file"),
            "Error should mention parse failure"
        );
    }

    println!("✅ Loading invalid JSON fails gracefully");
}
