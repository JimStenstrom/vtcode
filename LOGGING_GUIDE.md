# Comprehensive Logging Guide for VTCode

This guide explains how to use the comprehensive logging system added to VTCode to debug and monitor agent behavior, LLM inference, and tool execution.

## Overview

VTCode now has comprehensive logging instrumentation across critical paths:

1. **LLM Inference** (`vtcode-llm-anthropic/src/anthropic.rs`)
   - Request formation and conversion
   - API call timing and HTTP details
   - Response parsing and token usage
   - Cache hit/miss tracking
   - Error handling and diagnostics

2. **Agent Session Loop** (`src/agent/runloop/unified/turn/session.rs`)
   - Session startup and configuration
   - Turn-by-turn execution
   - LLM request dispatch
   - Response handling

3. **Tool Execution** (`src/agent/runloop/unified/tool_pipeline.rs`)
   - Tool execution lifecycle
   - Timeout tracking
   - Progress monitoring
   - Success/failure tracking

## Quick Start

### Enable Logging via Environment Variable

The fastest way to enable logging:

```bash
# Basic logging (info level)
RUST_LOG=info vtcode chat

# Detailed logging (debug level)
RUST_LOG=debug vtcode chat

# Full trace logging (very verbose)
RUST_LOG=trace vtcode chat

# Target-specific logging
RUST_LOG=vtcode_llm_anthropic=debug,vtcode::agent::llm=info vtcode chat

# Log to file
RUST_LOG=debug vtcode chat 2>&1 | tee vtcode.log
```

### Enable Logging via Configuration File

Edit your `.vtcode/config.toml`:

```toml
[debug]
enable_tracing = true
trace_level = "debug"  # Can be: error, warn, info, debug, trace
trace_targets = [
    "vtcode_llm_anthropic",    # LLM provider calls
    "vtcode::agent::llm",       # Agent LLM interactions
    "vtcode::agent",            # General agent behavior
    "vtcode_core::tools",       # Tool execution
]

# Optional: Log to file
debug_log_dir = ".vtcode/logs"
max_debug_log_size_mb = 100
```

## Log Levels and What They Show

### ERROR
- Critical failures
- API errors
- Unexpected conditions
- Tool execution failures

### WARN
- Rate limit hits
- Timeout warnings
- Deprecated feature usage
- Potential issues

### INFO (Recommended for normal debugging)
- Session start/end
- LLM request dispatch
- LLM response completion
- Tool execution start/completion
- Token usage summary
- Cache hit/miss notifications

### DEBUG (Detailed debugging)
- Request parameter details
- Response parsing steps
- Cache configuration
- Timeout calculations
- Progress updates
- Performance metrics

### TRACE (Very verbose - for deep debugging)
- Full request/response bodies
- Individual message processing
- Tool argument details
- Cache breakpoint allocation
- HTTP headers and URLs

## Key Logging Targets

### LLM Provider Logging

```bash
# See all LLM provider activity
RUST_LOG=vtcode_llm_anthropic=debug

# Just the request/response cycle
RUST_LOG=vtcode::agent::llm=info
```

**What you'll see:**
- Request conversion to provider format
- Tool count and message count
- Cache configuration
- HTTP request/response timing
- Token usage (input, output, cache read, cache creation)
- Stop reason and finish status

**Example output:**
```
INFO vtcode_llm_anthropic: Converting request to Anthropic format
INFO vtcode_llm_anthropic: Converting 3 tools to Anthropic format
INFO vtcode_llm_anthropic: Processing system prompt (length: 2451 chars)
INFO vtcode::agent::llm: Dispatching LLM request to provider model=claude-sonnet-4-5-20250929 messages=5 tools=3
INFO vtcode::agent::llm: LLM generation completed elapsed_ms=2341 succeeded=true
INFO vtcode_llm_anthropic: Token usage: input=3245, output=512, total=3757
INFO vtcode_llm_anthropic: Cache READ tokens: 2100 (significant cost savings!)
INFO vtcode_llm_anthropic: Response summary: content_length=1024, tool_calls=1
```

### Tool Execution Logging

```bash
# See tool execution details
RUST_LOG=vtcode::agent::runloop::unified::tool_pipeline=debug
```

**What you'll see:**
- Tool name and arguments
- Timeout configuration
- Execution phases
- Success/failure status
- Modified files
- Execution duration

**Example output:**
```
INFO vtcode::agent::runloop::unified::tool_pipeline: Executing tool: Read
DEBUG vtcode::agent::runloop::unified::tool_pipeline: Tool timeout configuration: category=Fast, ceiling=60s
DEBUG vtcode::agent::runloop::unified::tool_pipeline: Phase 1: Preparation
INFO vtcode::agent::runloop::unified::tool_pipeline: Tool Read completed successfully in 127ms (command_success=true, modified_files=0)
```

### Session Lifecycle Logging

```bash
# See session lifecycle
RUST_LOG=vtcode::agent::runloop=debug
```

**What you'll see:**
- Session startup
- Configuration loaded
- Resume state
- Turn-by-turn progress

## Common Use Cases

### 1. Debug LLM Request/Response Issues

```bash
RUST_LOG=vtcode_llm_anthropic=trace,vtcode::agent::llm=debug vtcode chat 2>&1 | tee llm-debug.log
```

This will show:
- Exact request body sent to API
- All messages in the request
- Tool definitions
- Complete response JSON
- Token breakdowns

### 2. Track Token Usage and Caching

```bash
RUST_LOG=vtcode_llm_anthropic=info vtcode chat
```

Look for lines containing:
- "Token usage"
- "Cache READ tokens"
- "Cache CREATION tokens"
- "cache_control"

### 3. Debug Tool Execution

```bash
RUST_LOG=vtcode::agent::runloop::unified::tool_pipeline=debug vtcode chat
```

This shows:
- Each tool invocation
- Arguments passed
- Execution time
- Success/failure
- Files modified

### 4. Identify Performance Bottlenecks

```bash
RUST_LOG=info vtcode chat 2>&1 | grep -E "(completed|elapsed|duration)"
```

This filters to show timing information:
- LLM request duration
- Tool execution duration
- HTTP round-trip time

### 5. Debug Session Issues

```bash
RUST_LOG=vtcode=debug vtcode chat 2>&1 | tee session-debug.log
```

Full session debugging including:
- Session initialization
- Configuration loading
- LLM interactions
- Tool executions
- Error handling

## Filtering Logs in Real-Time

### Show only errors and warnings:
```bash
RUST_LOG=warn vtcode chat
```

### Show info from LLM, debug from tools:
```bash
RUST_LOG=vtcode_llm_anthropic=info,vtcode::agent::runloop::unified::tool_pipeline=debug vtcode chat
```

### Exclude noisy targets:
```bash
RUST_LOG=debug,tokio=warn,hyper=warn vtcode chat
```

## Log Analysis Tips

### Count LLM requests:
```bash
grep "Dispatching LLM request" vtcode.log | wc -l
```

### Total token usage:
```bash
grep "Token usage:" vtcode.log
```

### Find all errors:
```bash
grep "ERROR" vtcode.log
```

### Tool success rate:
```bash
echo "Successful:" $(grep "completed successfully" vtcode.log | wc -l)
echo "Failed:" $(grep "Tool.*failed" vtcode.log | wc -l)
```

### Cache efficiency:
```bash
grep "Cache READ tokens" vtcode.log
```

## Structured Logging Fields

The logging system uses structured fields for filtering and analysis:

- `model`: LLM model name
- `streaming`: Whether streaming is enabled
- `step`: Turn/step number
- `messages`: Message count in request
- `tools`: Tool count in request
- `elapsed_ms`: Duration in milliseconds
- `workspace`: Workspace path
- `tool_name`: Name of executing tool
- `timeout_secs`: Tool timeout value

## Performance Considerations

- **TRACE level** can generate large logs (MB/minute in active sessions)
- **DEBUG level** is usually sufficient for most debugging
- **INFO level** is recommended for production monitoring
- Use `debug_log_dir` to keep logs separate from stderr

## Troubleshooting

### Logs not appearing?

1. Check RUST_LOG is set: `echo $RUST_LOG`
2. Verify config file: `vtcode config`
3. Try explicit environment variable: `RUST_LOG=info vtcode chat`

### Too much output?

1. Use more specific targets: `RUST_LOG=vtcode_llm_anthropic=info`
2. Lower the level: Use `info` instead of `debug`
3. Pipe to a file: `vtcode chat 2>&1 | tee vtcode.log`

### Missing expected logs?

1. Some logs are INFO level - set `RUST_LOG=info` or higher
2. Some detailed logs are DEBUG - use `RUST_LOG=debug`
3. Request/response bodies are TRACE - use `RUST_LOG=trace`

## Advanced: Programmatic Log Analysis

Parse structured logs with jq (if using JSON output):

```bash
# Future: JSON output support
vtcode chat --log-format=json 2>&1 | jq 'select(.level == "ERROR")'
```

## Getting Help

If logging reveals issues:

1. Save the log: `RUST_LOG=debug vtcode chat 2>&1 | tee issue.log`
2. Include relevant sections in bug reports
3. Redact any sensitive information (API keys, file paths)

## What Gets Logged

### ✅ Logged
- Timing information
- Token counts
- Tool names and execution status
- Model names
- Message counts
- Error messages
- Cache statistics

### ❌ NOT Logged (by default)
- Full message content (use TRACE)
- API keys
- File contents
- User data

**Note:** TRACE level may include sensitive data like full request bodies. Use with caution and don't share TRACE logs publicly.
