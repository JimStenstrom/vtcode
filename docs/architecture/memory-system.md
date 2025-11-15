# Memory System Architecture

## Overview

vtcode uses a three-tier memory system to maintain conversation context without losing information. This architecture enables long-running conversations while managing context window limits and supporting cross-session learning.

## Architecture

### Three Tiers

The memory system is organized into three distinct tiers, each optimized for different access patterns and retention requirements:

1. **Working Memory (Hot)**
   - Last 20 conversation turns by default
   - Full message fidelity - complete message history
   - Instant access with zero latency
   - Stored in-memory for maximum performance
   - No compression or summarization

2. **Recent Summaries (Warm)**
   - Up to 100 compressed summaries by default
   - Background async summarization
   - Temporal decay scoring for relevance
   - Retrieved automatically on historical queries
   - Includes metadata: tools used, files modified, goal progress

3. **Session Logs (Cold)**
   - Persistent JSON storage on disk
   - Complete conversation history
   - Searchable with standard tools (grep, jq)
   - Enables cross-session learning
   - Location: `~/.vtcode/sessions/YYYYMMDD_HHMMSS.json`

### Data Flow

```
User Message
    ↓
Memory.build_context()
    ├─→ Working Memory (last 20 turns)
    ├─→ Historical Query? → Search Summaries
    └─→ Return combined context
    ↓
LLM Request
    ↓
Response
    ↓
Memory.add_turn()
    ├─→ Add to working memory
    ├─→ Overflow? → Summarize (background)
    └─→ Periodic checkpoint → Session log
```

### Memory Lifecycle

1. **Turn Addition**:
   - New conversation turn added to working memory
   - Session metadata updated (turn count, token count)
   - If working memory exceeds limit, oldest turn moves to summarization queue

2. **Background Summarization** (async):
   - Oldest turns from working memory are summarized
   - Summaries include: content digest, tools used, files modified
   - Summaries stored with timestamps for temporal decay
   - No user-perceived latency

3. **Context Building**:
   - All working memory included by default
   - Historical query detection: "earlier", "before", "you said", etc.
   - Relevant summaries retrieved based on fuzzy matching + temporal decay
   - Context assembled and returned to LLM

4. **Session Persistence**:
   - Periodic checkpoints (configurable interval)
   - All messages and summaries saved to JSON
   - Metadata includes: total turns, tokens, duration, workspace path

## Components

### vtcode-memory

The core memory management crate providing the three-tier architecture.

**Key Types**:
- `SimpleMemory` - Main memory manager implementation
- `ConversationTurn` - Group of messages representing one turn
- `TurnSummary` - Compressed summary with temporal decay
- `SessionLog` - Persistent storage format
- `MemoryConfig` - Configuration options

**Example**:
```rust
use vtcode_memory::{SimpleMemory, MemoryConfig};

let config = MemoryConfig::default();
let mut memory = SimpleMemory::new(config, workspace_path);

// Add turns as conversation progresses
memory.add_turn(turn).await?;

// Build context for LLM request
let context = memory.build_context(user_message);

// Process background tasks (summarization)
memory.process_background_tasks().await?;

// Save session when done
let path = memory.save().await?;
```

### vtcode-vectordb

Abstract vector database interface for semantic search capabilities.

**Backends**:
- `InMemoryVectorDb` - Development and testing (current)
- Future: Qdrant, LanceDB, Weaviate

**Key Traits**:
- `VectorDb` - Core database operations
- `VectorPoint` - Data point with vector and metadata

**Example**:
```rust
use vtcode_vectordb::{InMemoryVectorDb, VectorDb, VectorPoint, Distance};

let db = InMemoryVectorDb::new();
db.create_collection("docs", 384, Distance::Cosine).await?;

let points = vec![VectorPoint::new(id, vector, metadata)];
db.upsert("docs", points).await?;

let results = db.search("docs", query_vector, 10, None).await?;
```

### vtcode-rag

Document retrieval system for RAG (Retrieval-Augmented Generation) capabilities.

**Components**:
- `Document` - Source document with metadata
- `Chunker` - Document chunking strategies (fixed-size, semantic)
- `Embedder` - Text embedding interface
- `IndexingPipeline` - Document indexing workflow
- `QueryPipeline` - Search and retrieval workflow

**Example**:
```rust
use vtcode_rag::{Document, IndexingPipeline, QueryPipeline};

// Index documents
let indexing = IndexingPipeline::new(vectordb, embedder, chunker, "collection");
indexing.index_document(doc).await?;

// Query
let query = QueryPipeline::new(vectordb, embedder, "collection");
let results = query.retrieve("search query", 5, None).await?;
```

## Configuration

Memory system configuration is managed through `vtcode.toml`:

```toml
[memory]
enabled = true                          # Enable memory system
working_memory_limit = 20               # Recent turns in full fidelity
summary_limit = 100                     # Max compressed summaries
enable_background_summarization = true  # Async summarization
auto_checkpoint = true                  # Save periodically
checkpoint_interval_seconds = 300       # Every 5 minutes
log_directory = "~/.vtcode/sessions"    # Where to save sessions
```

See `docs/guides/configuration.md` for detailed configuration options.

## Temporal Decay Scoring

Summaries use temporal decay to determine relevance:

| Age Range | Relevance Score |
|-----------|----------------|
| 0-5 minutes | 100% |
| 5-30 minutes | 80% |
| 30 min - 2 hours | 50% |
| 2-24 hours | 20% |
| 24+ hours | 5% |

Access frequency boosts relevance:
- Each access adds +10% (up to +50% max)
- Frequently accessed summaries remain more relevant

## Historical Query Detection

The system automatically detects when users ask about past interactions:

**Triggers**:
- "earlier", "before", "previously"
- "you said", "we discussed", "remember when"
- "last time", "ago"

**Behavior**:
- Search summaries using fuzzy matching
- Combine match score with temporal decay
- Return top 3-5 most relevant summaries
- Inject as system messages for context

## Performance

- **Memory footprint**: ~30KB per session (runtime)
- **Context building**: < 50ms
- **Background summarization**: < 3s per turn (async, non-blocking)
- **Session save**: < 100ms
- **Add turn**: O(1) amortized
- **Search summaries**: O(m) where m ≤ 100

## Session Persistence Format

Sessions are saved as JSON with full metadata:

```json
{
  "timestamp": "2025-11-15T14:30:22Z",
  "metadata": {
    "total_turns": 87,
    "total_tokens": 45230,
    "session_duration_seconds": 3600,
    "workspace": "/path/to/project",
    "session_start": "2025-11-15T13:30:22Z",
    "session_end": "2025-11-15T14:30:22Z"
  },
  "messages": [
    {
      "role": "user",
      "content": "...",
      "timestamp": "..."
    }
  ],
  "summaries": [
    {
      "id": "uuid",
      "content": "User asked about X. Assistant explained Y.",
      "turn_range": [5, 5],
      "timestamp": "2025-11-15T13:45:00Z",
      "access_count": 2,
      "tools_used": ["Read", "Edit"],
      "files_modified": ["src/main.rs"]
    }
  ]
}
```

## Integration with vtcode-core

The memory system integrates with vtcode's agent architecture through the context management system:

### Context Manager Integration

```rust
// In ContextManager
async fn build_context(&self, user_input: &str) -> Vec<Message> {
    if let Some(memory) = &self.memory {
        // Use memory-backed context building
        memory.build_context(user_input)
    } else {
        // Fall back to traditional context pruning
        self.prune_context()
    }
}
```

### Turn Loop Integration

```rust
// In UnifiedTurnDriver
async fn execute_turn(&mut self, user_input: String) -> Result<()> {
    // Build context from memory
    let context = self.memory.build_context(&user_input);

    // Execute turn with LLM
    let response = self.llm_provider.generate(context).await?;

    // Add turn to memory
    let turn = ConversationTurn::new(self.turn_index, messages);
    self.memory.add_turn(turn).await?;

    // Background processing (async, non-blocking)
    tokio::spawn(async move {
        memory.process_background_tasks().await.ok();
    });

    Ok(())
}
```

### Non-Destructive Context Pruning

Previously, vtcode deleted messages when context limits were exceeded. The new memory system preserves all messages:

**Old Behavior**:
```rust
// Messages permanently deleted
messages.drain(0..prune_count);
```

**New Behavior**:
```rust
// Messages preserved in memory before removal from context
for msg in messages.drain(0..prune_count) {
    memory.archive_message(msg);
}
```

## Cross-Session Learning

Session logs enable learning across conversations:

### Searching Historical Sessions

Use standard CLI tools to search past sessions:

```bash
# Find all sessions mentioning authentication
grep -r "authentication" ~/.vtcode/sessions/

# Search for specific topics in date range
grep -r "JWT token" ~/.vtcode/sessions/202511*.json

# Use jq for structured queries
jq '.messages[] | select(.content | contains("async"))' \
   ~/.vtcode/sessions/20251115_143022.json

# Extract all tool usage
jq '.summaries[].tools_used[]' ~/.vtcode/sessions/*.json | sort | uniq -c

# Count turns per session
jq '.metadata.total_turns' ~/.vtcode/sessions/*.json
```

### Future: Semantic Session Search

Planned enhancements for Phase 2:

```rust
// Find sessions related to a topic (semantic search)
let related_sessions = memory_store
    .search_sessions("authentication implementation")
    .await?;

// Restore context from previous session
let previous_context = memory_store
    .load_session("20251115_143022")
    .await?
    .extract_relevant_context("JWT tokens");
```

## Error Handling

The memory system includes comprehensive error handling:

```rust
pub enum MemoryError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    ConfigurationError(String),
    SummarizationError(String),
}
```

All memory operations return `Result<T, MemoryError>` and include detailed logging at the `debug` and `trace` levels.

## Migration from Legacy System

vtcode previously used destructive context pruning. Migration to the memory system is seamless:

1. **Opt-in via Configuration**:
   ```toml
   [memory]
   enabled = true
   ```

2. **Backward Compatibility**:
   - If `enabled = false`, falls back to traditional pruning
   - No breaking changes to existing workflows

3. **Gradual Rollout**:
   - Feature flag controls memory system activation
   - Can disable per-session if issues occur

## Future Enhancements

### Phase 2: LLM-Based Summarization

Replace simple text extraction with LLM-generated summaries:

```rust
impl SimpleMemory {
    async fn summarize_turn(&self, turn: &ConversationTurn) -> Result<TurnSummary> {
        let prompt = format!(
            "Summarize this conversation turn in 1-2 sentences:\n{}",
            turn.to_text()
        );

        let summary_text = self.llm_provider
            .generate_summary(prompt)
            .await?;

        TurnSummary::new(summary_text, turn.index, turn.timestamp)
    }
}
```

### Phase 3: Semantic Search via Embeddings

Integrate with vtcode-vectordb for semantic search:

```rust
// Index summaries as they're created
let embedding = embedder.embed(&summary.content).await?;
vectordb.upsert("summaries", vec![
    VectorPoint::new(summary.id, embedding, summary.metadata())
]).await?;

// Semantic search for historical queries
let query_embedding = embedder.embed(user_query).await?;
let results = vectordb.search("summaries", query_embedding, 5, None).await?;
```

### Phase 4: GraphRAG Integration

Build knowledge graphs of code relationships:

```rust
// Track relationships between code entities
memory.add_relationship(
    "File", "src/main.rs",
    "Imports", "src/config.rs",
    RelationType::DependsOn
);

// Query: "What files are related to authentication?"
let graph_results = memory.query_graph(
    "authentication",
    RelationType::Any,
    max_depth: 3
).await?;
```

### Phase 5: Multi-Session Context Restoration

Automatically restore relevant context from previous sessions:

```rust
// On session start in same workspace
let workspace_sessions = memory_store
    .list_sessions_for_workspace(&workspace_path)
    .await?;

// Find related previous work
let related_context = memory_store
    .find_related_context(current_task, &workspace_sessions)
    .await?;

// Include in initial context
initial_messages.extend(related_context);
```

## Troubleshooting

See `docs/guides/troubleshooting.md` for common issues and solutions.

## References

- vtcode-memory crate: `/vtcode-memory/`
- Configuration guide: `docs/guides/configuration.md`
- Quick start: `docs/guides/memory-quick-start.md`
- API documentation: Run `cargo doc --open -p vtcode-memory`
