# VTCode Observability and Debugging

This directory contains documentation for observability features including telemetry, logging, and trajectory analysis.

## Overview

VTCode provides comprehensive observability tools to help you monitor, debug, and improve agent performance:

- **Logging** - Detailed execution logs with configurable levels
- **Telemetry** - Usage analytics and performance metrics
- **Trajectory Logging** - Structured records of routing decisions and tool calls

## Documentation

### Trajectory Logging

**[TRAJECTORY_LOGGING.md](./TRAJECTORY_LOGGING.md)** - Structured JSON logging for agent evaluation:
- Records routing decisions and tool executions
- Output: `.vtcode/logs/trajectory.jsonl`
- Supports trajectory evaluation and analysis
- Includes timestamps, models, classes, and success rates

## Configuration

All observability features are configured in `vtcode.toml`. See **[Configuration Reference](../config.md#observability-and-telemetry)** for details.

### Telemetry Configuration

```toml
[telemetry]
# Enable telemetry collection (disabled by default for privacy)
enabled = false

# Whether to include usage analytics
analytics = false

# Whether to report errors to the development team
report_errors = true

# Level of detail: "minimal" | "basic" | "detailed"
level = "minimal"
```

### Logging Configuration

```toml
[logging]
# Enable detailed logging (useful for debugging)
enabled = false

# Log level: "error" | "warn" | "info" | "debug" | "trace"
level = "info"

# Whether to include sensitive information in logs
include_sensitive = false

# Maximum size of log files before rotation (in bytes)
max_log_size = 10485760  # 10MB
```

### Environment Variables

You can also control logging via environment variables:

```bash
# Set log level for Rust tracing
RUST_LOG=info vtcode

# Enable debug logging for specific modules
RUST_LOG=vtcode_core=debug,vtcode_tools=trace vtcode

# Debug specific subsystems
RUST_LOG=vtcode_core::mcp_client=debug vtcode
RUST_LOG=tree_sitter=debug vtcode
RUST_LOG=vtcode_acp_client=trace vtcode
```

## Log Files Location

VTCode stores logs in the following locations:

- **Trajectory logs**: `.vtcode/logs/trajectory.jsonl`
- **Application logs**: `.vtcode/logs/vtcode.log` (when logging enabled)
- **Error logs**: `.vtcode/logs/error.log`

## Debugging Use Cases

### 1. Debugging Agent Behavior

Enable trajectory logging to analyze routing decisions:

```bash
# Trajectory logging is enabled by default
# Check the logs:
cat .vtcode/logs/trajectory.jsonl | jq .
```

Analyze tool success rates:
```bash
# Count successful vs failed tool calls
cat .vtcode/logs/trajectory.jsonl | jq 'select(.kind == "tool") | .ok' | sort | uniq -c
```

### 2. Debugging Tool Execution

Enable debug logging for tool operations:

```bash
RUST_LOG=vtcode_tools=debug vtcode
```

### 3. Debugging MCP Integration

Enable MCP client tracing:

```bash
RUST_LOG=vtcode_core::mcp_client=debug vtcode
```

### 4. Debugging Performance Issues

Enable detailed tracing:

```bash
RUST_LOG=trace vtcode 2>&1 | tee debug.log
```

## Analysis Tips

### Trajectory Analysis

Aggregate routing decisions by complexity class:
```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "route") | .class' | \
  sort | uniq -c
```

Calculate tool success rates per model:
```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "tool") | "\(.name),\(.ok)"' | \
  sort | uniq -c
```

Reconstruct agent timeline:
```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r '"\(.ts) \(.kind) \(.selected_model // .name)"'
```

### Log Analysis

Search for errors in logs:
```bash
grep -i error .vtcode/logs/vtcode.log
```

Monitor real-time execution:
```bash
tail -f .vtcode/logs/vtcode.log
```

## Privacy and Security

- **Telemetry is disabled by default** - You must explicitly opt-in
- **Logs never include sensitive data by default** - API keys, credentials are redacted
- **Trajectory logs include tool arguments** - May contain file paths and commands
- **Log rotation** - Prevents unbounded disk usage

To ensure no sensitive data is logged:
```toml
[logging]
include_sensitive = false  # Always keep this false in production
```

## Related Documentation

- **[Configuration Reference](../config.md)** - Complete configuration options
- **[Security Guide](../guides/security.md)** - Security best practices
- **[Tool Specifications](../tools/TOOL_SPECS.md)** - Understanding tool calls in logs

---

**Last Updated**: November 2025
**VTCode Version**: 0.43.6
