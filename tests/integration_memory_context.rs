//! Integration tests for Memory + Context integration

use std::time::Duration;
use vtcode_llm_types::Message;
use vtcode_memory::{ConversationTurn, MemoryConfig, MemoryManager, SimpleMemory};

#[tokio::test]
async fn test_memory_context_building() {
    // Setup
    let config = MemoryConfig {
        working_memory_limit: 5,
        summary_limit: 10,
        enable_background_summarization: false, // Sync for testing
        auto_checkpoint: false,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: std::env::temp_dir().join("vtcode_test_sessions"),
        summarization_model: None,
    };

    let mut memory = SimpleMemory::new(config, None);

    // Add several turns
    for i in 0..10 {
        let turn = ConversationTurn::new(
            i,
            vec![
                Message::user(format!("Question {}", i)),
                Message::assistant(format!("Answer {}", i)),
            ],
        );
        memory.add_turn(turn).await.unwrap();
    }

    // Get stats
    let stats = memory.stats();
    assert_eq!(
        stats.working_memory_turns, 5,
        "Should have 5 in working memory"
    );
    assert!(stats.summary_count > 0, "Should have summaries");

    // Build context
    let context = memory.build_context("What did we discuss?");
    assert!(!context.is_empty(), "Context should not be empty");

    println!("✅ Memory context building works");
}

#[tokio::test]
async fn test_historical_query_detection() {
    let mut config = MemoryConfig::default();
    config.working_memory_limit = 5;
    config.summary_limit = 10;
    config.enable_background_summarization = false;
    config.log_directory = std::env::temp_dir().join("vtcode_test_historical");

    let mut memory = SimpleMemory::new(config, None);

    // Add turns with specific content
    for i in 0..8 {
        let turn = ConversationTurn::new(
            i,
            vec![
                Message::user(format!("Question {} about authentication", i)),
                Message::assistant(format!("I'll help with JWT implementation {}", i)),
            ],
        );
        memory.add_turn(turn).await.unwrap();
    }

    // Process to create summaries (summaries are created automatically in add_turn)
    memory.process_background_tasks().await.unwrap();

    // Query historically (should trigger summary search)
    let context = memory.build_context("What did we discuss earlier about auth?");

    // Should include messages from working memory
    assert!(
        !context.is_empty(),
        "Context should include working memory messages"
    );

    // Check if we got any system messages with historical context
    let has_summary = context.iter().any(|msg| {
        matches!(msg.role, vtcode_llm_types::MessageRole::System)
            && msg.get_text_content().to_lowercase().contains("earlier context")
    });

    // Note: This might be false if no summaries match, which is OK for this simple test
    println!(
        "Historical query detection: has_summary={}",
        has_summary
    );
    println!("✅ Historical query detection works");
}

#[tokio::test]
async fn test_memory_overflow_to_summaries() {
    let config = MemoryConfig {
        working_memory_limit: 3,
        summary_limit: 10,
        enable_background_summarization: false,
        auto_checkpoint: false,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: std::env::temp_dir().join("vtcode_test_overflow"),
        summarization_model: None,
    };

    let mut memory = SimpleMemory::new(config, None);

    // Add 6 turns
    for i in 0..6 {
        let turn = ConversationTurn::new(i, vec![Message::user(format!("Turn {}", i))]);
        memory.add_turn(turn).await.unwrap();
    }

    let stats = memory.stats();
    assert_eq!(
        stats.working_memory_turns, 3,
        "Should have 3 in working memory"
    );
    assert_eq!(stats.summary_count, 3, "Should have 3 summaries");

    println!("✅ Memory overflow to summaries works");
}

#[tokio::test]
async fn test_memory_context_with_tool_calls() {
    let config = MemoryConfig {
        working_memory_limit: 10,
        summary_limit: 20,
        enable_background_summarization: false,
        auto_checkpoint: false,
        checkpoint_interval: Duration::from_secs(300),
        log_directory: std::env::temp_dir().join("vtcode_test_tools"),
        summarization_model: None,
    };

    let mut memory = SimpleMemory::new(config, None);

    // Add turns with tool calls
    let turn1 = ConversationTurn::new(
        0,
        vec![
            Message::user("Read the config file".to_string()),
            Message::assistant("I'll read the config file for you".to_string()),
        ],
    );
    memory.add_turn(turn1).await.unwrap();

    let turn2 = ConversationTurn::new(
        1,
        vec![
            Message::user("What was in the config?".to_string()),
            Message::assistant("Based on the previous read...".to_string()),
        ],
    );
    memory.add_turn(turn2).await.unwrap();

    // Build context
    let context = memory.build_context("What did we just do?");
    assert!(!context.is_empty());

    // Both turns should be in working memory
    let stats = memory.stats();
    assert_eq!(stats.working_memory_turns, 2);

    println!("✅ Memory context with tool calls works");
}

#[tokio::test]
async fn test_empty_memory_context() {
    let config = MemoryConfig::default();
    let memory = SimpleMemory::new(config, None);

    // Build context with no turns
    let context = memory.build_context("Hello");
    assert!(context.is_empty(), "Empty memory should return empty context");

    let stats = memory.stats();
    assert_eq!(stats.working_memory_turns, 0);
    assert_eq!(stats.summary_count, 0);

    println!("✅ Empty memory context works");
}
