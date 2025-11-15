# Memory System Quick Start

## What is it?

VTCode's memory system preserves conversation context across long sessions and even across separate sessions in the same workspace. Instead of deleting old messages when context limits are reached, VTCode maintains a three-tier memory architecture that ensures you never lose important information.

## How it works

VTCode's memory system operates on three tiers:

1. **Working Memory (Hot)** - Keeps the last 20 turns in full detail
2. **Summaries (Warm)** - Compresses older turns into searchable summaries (up to 100)
3. **Session Logs (Cold)** - Saves everything to `~/.vtcode/sessions/` for permanent storage

When you ask about something from earlier in the conversation, VTCode automatically searches the summaries and includes relevant context in the response.

## Quick Setup

### 1. Enable Memory (Default: On)

The memory system is enabled by default. To verify or customize:

```toml
# vtcode.toml
[memory]
enabled = true
working_memory_limit = 20
summary_limit = 100
enable_background_summarization = true
```

### 2. Use VTCode Normally

Simply use VTCode as you normally would:

```bash
vtcode

> Implement user authentication with JWT tokens
... (conversation continues for many turns) ...

> Remember what we discussed about JWT token validation?  # Works!
```

The memory system works automatically in the background. You'll notice:
- Long conversations stay coherent even after 100+ turns
- Historical queries like "remember when..." retrieve past context
- No delay from summarization (it happens asynchronously)

### 3. View Your Session Logs

Sessions are automatically saved to disk:

```bash
# List your sessions
ls ~/.vtcode/sessions/
# Output: 20251115_143022.json, 20251115_151234.json, ...

# View session metadata
jq '.metadata' ~/.vtcode/sessions/20251115_143022.json
# Output:
# {
#   "total_turns": 87,
#   "total_tokens": 45230,
#   "session_duration_seconds": 3600,
#   "workspace": "/home/user/my-project"
# }
```

## Advanced Usage

### Search Past Sessions

Use standard command-line tools to search historical sessions:

```bash
# Find all sessions mentioning authentication
grep -r "authentication" ~/.vtcode/sessions/

# Search for specific topics
grep -r "JWT" ~/.vtcode/sessions/202511*.json

# Extract messages about async/await
jq '.messages[] | select(.content | contains("async"))' \
   ~/.vtcode/sessions/20251115_143022.json

# Find all tools used in a session
jq '.summaries[].tools_used[]' ~/.vtcode/sessions/20251115_143022.json | sort | uniq
```

### Tune Memory Settings

#### For Longer Retention

If you have long, complex conversations:

```toml
[memory]
working_memory_limit = 30  # Keep more recent history
summary_limit = 200        # Store more summaries
```

#### For Better Performance

If you're on a resource-constrained system:

```toml
[memory]
working_memory_limit = 10  # Reduce memory footprint
summary_limit = 50         # Fewer summaries
```

#### For Fastest Speed

Disable background summarization if you notice any delays:

```toml
[memory]
enable_background_summarization = false
```

Note: This will make summarization synchronous, which may add a small delay after each turn.

### Customize Session Storage

Change where sessions are saved:

```toml
[memory]
log_directory = "/custom/path/to/sessions"
```

Or adjust checkpoint frequency:

```toml
[memory]
auto_checkpoint = true
checkpoint_interval_seconds = 600  # Every 10 minutes (default: 300)
```

## Understanding Historical Queries

The memory system automatically detects when you're asking about the past:

**Triggers historical search:**
- "What did we discuss earlier?"
- "You said something before about..."
- "Remember when we fixed that bug?"
- "Last time we talked about..."

**Regular queries (no search):**
- "What is Rust?"
- "How do I implement X?"
- "Explain Y to me"

When a historical query is detected, VTCode:
1. Searches summaries using fuzzy matching
2. Applies temporal decay scoring (recent = more relevant)
3. Returns top 3-5 most relevant summaries
4. Injects them as context for the LLM

## FAQ

### Q: Does this slow down VTCode?

**A:** No. Summarization happens in the background while you type your next message. Context building is < 50ms.

### Q: How much disk space does it use?

**A:** Approximately ~500KB per 100-turn session. A typical day of work might use 1-2MB.

### Q: Can I disable it?

**A:** Yes:

```toml
[memory]
enabled = false
```

VTCode will fall back to the traditional context pruning behavior (messages are deleted when limits are exceeded).

### Q: Where is the data stored?

**A:** By default in `~/.vtcode/sessions/` but this is configurable via `log_directory`.

### Q: Does it work across different sessions?

**A:** Session logs are saved permanently, so you can search them manually with `grep` and `jq`. Automatic cross-session context restoration is planned for a future release (Phase 5).

### Q: What format are session logs?

**A:** JSON format with full message history, summaries, and metadata:

```json
{
  "timestamp": "2025-11-15T14:30:22Z",
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

### Q: Can I use this with multiple workspaces?

**A:** Yes. Each workspace can have its own memory configuration in its `vtcode.toml`, and session logs include workspace metadata for filtering.

## Troubleshooting

### Sessions Not Saving

Check that the directory is writable:

```bash
mkdir -p ~/.vtcode/sessions
ls -la ~/.vtcode/
```

Verify auto-checkpoint is enabled:

```toml
[memory]
auto_checkpoint = true
```

### Historical Queries Not Finding Context

Ensure summarization is enabled and summaries exist:

```bash
# Check if any summaries were created
jq '.summaries | length' ~/.vtcode/sessions/20251115_*.json
```

If zero, enable background summarization:

```toml
[memory]
enable_background_summarization = true
summary_limit = 100
```

### Out of Memory Errors

Reduce memory limits:

```toml
[memory]
working_memory_limit = 10
summary_limit = 50
```

For more troubleshooting, see the [Troubleshooting Guide](../user-guide/troubleshooting.md#memory-system).

## Next Steps

- Read the full [Memory System Architecture](../architecture/memory-system.md)
- Configure advanced settings in the [Configuration Guide](../config.md#memory-configuration)
- Learn about [Context Engineering](../context_engineering.md)

---

**Happy coding with VTCode!** 🚀
