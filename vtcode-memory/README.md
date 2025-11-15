# vtcode-memory

Temporal-decay-based conversation memory system for VTCode agents.

## Overview

`vtcode-memory` provides a three-tier memory architecture for managing conversation history in AI agents, based on the reallm memory system design with adaptations for multi-agent scenarios.

**Documentation**:
- [Architecture Guide](../docs/architecture/memory-system.md) - Detailed system architecture and integration
- [Quick Start Guide](../docs/guides/memory-quick-start.md) - Get started with memory system quickly
- [Configuration Guide](../docs/config.md#memory-configuration) - Configuration options and examples

### Three-Tier Architecture

1. **Working Memory (Hot)** - Recent conversation turns with full fidelity
   - Default: Last 20 turns
   - Zero latency access
   - Complete message history

2. **Recent Summaries (Warm)** - Compressed summaries of older turns
   - Default: Up to 100 summaries
   - Background summarization
   - Relevance-based retrieval

3. **Session Logs (Cold)** - On-disk persistence for historical search
   - JSON-based storage
   - Full session archiving
   - Searchable via standard tools (grep, jq)

## Features

- **Temporal Decay** - Automatic relevance scoring based on age
- **Background Summarization** - Async processing with zero user-perceived latency
- **Configurable Limits** - Tune memory size to your needs
- **Session Persistence** - Save and restore conversation state
- **Historical Query Detection** - Automatically retrieve relevant context for "remember when..." queries
- **Fuzzy Search** - Find relevant summaries based on content similarity

## Usage

### Basic Example

```rust
use vtcode_memory::{SimpleMemory, MemoryManager, ConversationTurn};
use vtcode_llm_types::Message;

#[tokio::main]
async fn main() {
    // Create a memory manager with defaults
    let mut memory = SimpleMemory::with_defaults(None);

    // Add a conversation turn
    let turn = ConversationTurn::new(
        0,
        vec![
            Message::user("Hello!".to_string()),
            Message::assistant("Hi there! How can I help?".to_string()),
        ],
    );
    memory.add_turn(turn).await.unwrap();

    // Build context for the next request
    let context = memory.build_context("What's next?");

    // Process background tasks (summarization)
    memory.process_background_tasks().await.unwrap();

    // Save session when done
    let path = memory.save().await.unwrap();
    println!("Session saved to: {:?}", path);
}
```

### Custom Configuration

```rust
use vtcode_memory::{SimpleMemory, MemoryConfig};
use std::time::Duration;

let config = MemoryConfig {
    working_memory_limit: 30,  // Keep more recent history
    summary_limit: 150,
    auto_checkpoint: true,
    checkpoint_interval: Duration::from_secs(600), // 10 minutes
    enable_background_summarization: true,
    ..Default::default()
};

let memory = SimpleMemory::new(config, Some(workspace_path));
```

### Loading a Previous Session

```rust
use vtcode_memory::{SimpleMemory, MemoryManager};
use std::path::Path;

let session_path = Path::new("~/.vtcode/sessions/20241114_143022.json");
let memory = SimpleMemory::load(session_path).await.unwrap();
```

## Memory Configuration

The `MemoryConfig` struct controls memory behavior:

```rust
pub struct MemoryConfig {
    pub working_memory_limit: usize,           // Default: 20 turns
    pub summary_limit: usize,                  // Default: 100 summaries
    pub auto_checkpoint: bool,                 // Default: true
    pub checkpoint_interval: Duration,         // Default: 5 minutes
    pub log_directory: PathBuf,                // Default: ~/.vtcode/sessions
    pub enable_background_summarization: bool, // Default: true
    pub summarization_model: Option<String>,   // Optional LLM model for summaries
}
```

## Memory Stats

Get current memory usage statistics:

```rust
let stats = memory.stats();
println!("Working memory: {} turns", stats.working_memory_turns);
println!("Summaries: {}", stats.summary_count);
println!("Total tokens (approx): {}", stats.total_tokens_approximate);
println!("Session age: {:?}", stats.session_age);
```

## Historical Query Detection

The memory system automatically detects when users ask about past interactions:

```rust
// These queries will trigger summary retrieval:
memory.build_context("What did we discuss earlier?");
memory.build_context("You said something before about...");
memory.build_context("Remember when we fixed that bug?");

// Regular queries use only working memory:
memory.build_context("What is Rust?");
```

## Temporal Decay

Summaries are scored based on age for relevance:

| Age Range | Relevance Score |
|-----------|----------------|
| 0-5 minutes | 100% |
| 5-30 minutes | 80% |
| 30 min - 2 hours | 50% |
| 2-24 hours | 20% |
| 24+ hours | 5% |

Access frequency boosts relevance (frequently accessed = more important).

## Session Persistence

Sessions are saved as JSON files:

```json
{
  "timestamp": "2025-11-14T14:30:22Z",
  "metadata": {
    "total_turns": 87,
    "total_tokens": 45230,
    "session_duration_seconds": 3600,
    "workspace": "/path/to/project"
  },
  "messages": [...],
  "summaries": [...]
}
```

### Searching Historical Sessions

Use standard CLI tools:

```bash
# Search across all sessions
grep -r "authentication bug" ~/.vtcode/sessions/

# Search in specific date range
grep -r "JWT" ~/.vtcode/sessions/202411*.json

# Use jq for structured queries
jq '.messages[] | select(.content | contains("async"))' \
   ~/.vtcode/sessions/20241114_143022.json

# Count turns per session
jq '.metadata.total_turns' ~/.vtcode/sessions/*.json
```

## Integration with vtcode-core

The memory system integrates with vtcode's agent architecture:

```rust
// In your agent's turn loop
let context = memory.build_context(&user_input);
let response = llm_provider.generate(context).await?;

// After turn completion
let turn = ConversationTurn::new(turn_index, messages);
memory.add_turn(turn).await?;

// Background processing (async, non-blocking)
tokio::spawn(async move {
    memory.process_background_tasks().await.ok();
});
```

## Performance Characteristics

- **Add turn**: O(1) amortized (background summarization)
- **Build context**: O(n) where n ≤ 20 (working memory size)
- **Search summaries**: O(m) where m ≤ 100 (summary count)
- **Memory footprint**: ~30KB per session (runtime)
- **Disk usage**: ~500KB per 100-turn session

## Future Enhancements

Planned for v0.2+:

- [ ] LLM-based summarization (currently uses simple text extraction)
- [ ] Semantic search via embeddings
- [ ] Multi-session memory linking
- [ ] Knowledge graph construction
- [ ] Adaptive compression based on content type
- [ ] Cross-agent memory sharing

## Testing

Run the test suite:

```bash
cargo test -p vtcode-memory
```

All tests should pass. The test suite includes:
- Memory manager creation and basic operations
- Turn addition and context building
- Working memory overflow and summarization
- Historical query detection
- Fuzzy matching for summary search
- Session persistence (save/load)

## Integration with VTCode

This crate is integrated into the main VTCode project to provide:

1. **Context Management**: Replaces destructive context pruning with non-destructive memory preservation
2. **Historical Context**: Enables "remember when..." queries by searching summaries
3. **Cross-Session Learning**: Persistent session logs enable learning from past work
4. **Performance**: Background async summarization ensures zero user-perceived latency

For integration details, see the [Architecture Guide](../docs/architecture/memory-system.md#integration-with-vtcode-core).

## Related Crates

Part of the VTCode memory and storage ecosystem:

- **vtcode-vectordb** - Vector database abstraction for semantic search
- **vtcode-rag** - Document retrieval and RAG pipeline
- **vtcode-graphrag** - Hybrid vector + graph retrieval (planned)

## License

MIT

## Contributing

This is part of the [vtcode](https://github.com/vinhnx/vtcode) project. See the main repository for contribution guidelines.

For questions or issues specific to the memory system:
1. Check the [Troubleshooting Guide](../docs/user-guide/troubleshooting.md#memory-system)
2. Review the [Architecture Documentation](../docs/architecture/memory-system.md)
3. Open an issue on the [main repository](https://github.com/vinhnx/vtcode/issues)
