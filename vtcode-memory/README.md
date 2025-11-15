# vtcode-memory

Three-tier memory system for vtcode providing non-destructive context management with working memory, summaries, and persistent session logs.

## Overview

This crate implements a hierarchical memory architecture inspired by reallm:

- **Working Memory (Hot)**: Last N turns with full message fidelity
- **Recent Summaries (Warm)**: Compressed summaries with temporal decay
- **Session Logs (Cold)**: Persistent JSON storage for cross-session learning

## Features

- Non-destructive pruning - information is never lost, only moved to summaries
- Temporal decay scoring for summary relevance
- Historical query detection ("remember when...", "earlier we discussed...")
- Background async summarization (optional)
- Automatic checkpointing and session persistence
- Cross-session restoration from disk

## Usage

```rust
use vtcode_memory::{SimpleMemory, MemoryConfig, MemoryManager, ConversationTurn};
use vtcode_llm_types::Message;

// Initialize with defaults
let mut memory = SimpleMemory::with_defaults(Some(workspace_path));

// Or configure explicitly
let config = MemoryConfig {
    working_memory_limit: 20,
    summary_limit: 100,
    enable_background_summarization: true,
    auto_checkpoint: true,
    checkpoint_interval_seconds: 300,
    ..Default::default()
};
let mut memory = SimpleMemory::new(config, Some(workspace_path));

// Add conversation turns
let turn = ConversationTurn::new(
    0,
    vec![
        Message::user("Implement authentication".to_string()),
        Message::assistant("I'll help with that...".to_string()),
    ],
);
memory.add_turn(turn).await?;

// Build context for LLM (automatically includes summaries for historical queries)
let context = memory.build_context("What did we discuss earlier about auth?");

// Process background tasks (summarization)
memory.process_background_tasks().await?;

// Save session to disk
let path = memory.save().await?;

// Later: restore session
let memory = SimpleMemory::load(&path).await?;
```

## Configuration

See `MemoryConfig` for all available options:

- `enabled`: Enable/disable memory system
- `working_memory_limit`: Number of recent turns to keep in full fidelity (default: 20)
- `summary_limit`: Maximum number of summaries to retain (default: 100)
- `enable_background_summarization`: Async summarization (default: true)
- `auto_checkpoint`: Periodic saves (default: true)
- `checkpoint_interval_seconds`: How often to checkpoint (default: 300)
- `log_directory`: Where to save sessions (default: `~/.vtcode/sessions`)

## Architecture

The memory system integrates with vtcode's context management to provide:

1. **Non-destructive pruning**: When context exceeds token budget, older turns move to summaries instead of being deleted
2. **Intelligent retrieval**: Historical queries automatically search summaries
3. **Temporal relevance**: Summaries decay over time but accessed summaries stay relevant
4. **Session continuity**: Save/restore allows picking up where you left off

## Testing

```bash
cargo test --package vtcode-memory
```

Integration tests are in `tests/integration_memory_context.rs` and `tests/integration_cross_session.rs`.

## Related Crates

- `vtcode-llm-types`: Message types used in conversation turns
- `vtcode-core`: Context management integration
- `vtcode-config`: Configuration types
