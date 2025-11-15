# Trajectory Logging

VTCode records routing decisions and tool calls as structured JSON lines to support trajectory evaluation and continuous improvement. This enables detailed analysis of agent behavior, tool usage patterns, and model selection strategies.

**For comprehensive observability documentation**, see [Telemetry Overview](./README.md).

## Overview

Trajectory logging captures two types of events:
1. **Routing decisions** - When the router selects a model based on task complexity
2. **Tool executions** - When tools are called with their arguments and results

All events are written to `.vtcode/logs/trajectory.jsonl` as newline-delimited JSON.

## Output Location

- **File**: `.vtcode/logs/trajectory.jsonl`
- **Format**: JSON Lines (one JSON object per line)
- **Implementation**: `vtcode-core/src/core/trajectory.rs` (`TrajectoryLogger`)

## Record Types

### Route Records

Captures routing decisions when the agent selects a model:

```json
{
  "kind": "route",
  "turn": 5,
  "selected_model": "gemini-2.5-pro",
  "class": "complex",
  "input_preview": "Refactor the authentication system to use...",
  "ts": "2025-11-13T12:34:56.789Z"
}
```

**Fields:**
- `kind`: Always `"route"`
- `turn`: Turn number in the conversation
- `selected_model`: Model ID selected by the router
- `class`: Complexity classification (`"simple"`, `"standard"`, `"complex"`, `"codegen_heavy"`, `"retrieval_heavy"`)
- `input_preview`: First 100 characters of the user input
- `ts`: ISO 8601 timestamp

### Tool Records

Captures tool execution events:

```json
{
  "kind": "tool",
  "turn": 6,
  "name": "read_file",
  "args": {
    "path": "src/auth/mod.rs",
    "max_bytes": 50000
  },
  "ok": true,
  "ts": "2025-11-13T12:35:02.123Z"
}
```

**Fields:**
- `kind`: Always `"tool"`
- `turn`: Turn number in the conversation
- `name`: Tool name (e.g., `"read_file"`, `"grep_file"`, `"run_terminal_cmd"`)
- `args`: Tool arguments as JSON object
- `ok`: Boolean indicating success/failure
- `ts`: ISO 8601 timestamp

## Configuration

Trajectory logging is **enabled by default** whenever chat loops run (single-agent and unified tool chat).

To disable trajectory logging:

```toml
[telemetry]
trajectory_logging = false
```

To change the log location:

```toml
[telemetry]
trajectory_log_path = "/custom/path/trajectory.jsonl"
```

## Usage

### Viewing Trajectory Logs

```bash
# View all events
cat .vtcode/logs/trajectory.jsonl | jq .

# View only routing decisions
cat .vtcode/logs/trajectory.jsonl | jq 'select(.kind == "route")'

# View only tool calls
cat .vtcode/logs/trajectory.jsonl | jq 'select(.kind == "tool")'
```

### Safe for Long Sessions

- Appends one JSON object per line
- No performance impact on agent execution
- Automatic log rotation when files exceed size limits
- Safe for concurrent writes

## Analysis Examples

### 1. Aggregate by Complexity Class

See which complexity classes are most common:

```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "route") | .class' | \
  sort | uniq -c | sort -rn
```

Output:
```
    45 complex
    23 standard
    12 simple
     5 codegen_heavy
```

### 2. Model Selection Patterns

See which models are selected most often:

```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "route") | .selected_model' | \
  sort | uniq -c | sort -rn
```

### 3. Tool Success Rates

Calculate success rates for each tool:

```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "tool") | "\(.name),\(.ok)"' | \
  sort | uniq -c
```

Output:
```
    120 read_file,true
      3 read_file,false
     85 grep_file,true
      2 grep_file,false
     45 run_terminal_cmd,true
      8 run_terminal_cmd,false
```

### 4. Tool Usage by Complexity Class

Join routing and tool events to see which tools are used for each complexity:

```bash
# Extract turn-to-class mapping
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "route") | "\(.turn):\(.class)"' > /tmp/routes.txt

# Extract tool calls by turn
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "tool") | "\(.turn):\(.name)"' > /tmp/tools.txt

# Analyze patterns
join -t: /tmp/routes.txt /tmp/tools.txt | \
  awk -F: '{print $2, $3}' | \
  sort | uniq -c
```

### 5. Reconstruct Agent Timeline

View the complete agent execution timeline:

```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq -r '"\(.ts) [\(.kind)] \(.selected_model // .name) \(if .ok != null then (if .ok then "✓" else "✗" end) else "" end)"' | \
  sort
```

Output:
```
2025-11-13T12:34:56.789Z [route] gemini-2.5-pro
2025-11-13T12:35:02.123Z [tool] read_file ✓
2025-11-13T12:35:03.456Z [tool] grep_file ✓
2025-11-13T12:35:10.789Z [tool] edit_file ✓
```

### 6. Failed Tool Analysis

Identify failing tools and their arguments:

```bash
cat .vtcode/logs/trajectory.jsonl | \
  jq 'select(.kind == "tool" and .ok == false) | {name, args, ts}'
```

### 7. Tool Execution Time Analysis

If your logs include execution time:

```bash
# Aggregate average execution times per tool
cat .vtcode/logs/trajectory.jsonl | \
  jq -r 'select(.kind == "tool" and .duration_ms != null) | "\(.name),\(.duration_ms)"' | \
  awk -F, '{sum[$1]+=$2; count[$1]++} END {for (name in sum) print name, sum[name]/count[name]}'
```

## Use Cases

### 1. Continuous Improvement

- Identify which complexity classes need better routing heuristics
- Find tools with low success rates that need improvement
- Discover common usage patterns to optimize defaults

### 2. Debugging Agent Behavior

- Reconstruct exact sequence of routing and tool decisions
- Identify when and why routing changes models
- Debug tool failures with complete argument history

### 3. Performance Optimization

- Identify frequently used tools for caching optimization
- Find long-running tools that could benefit from streaming
- Discover tools that are called unnecessarily

### 4. A/B Testing

- Compare routing strategies by analyzing model selection patterns
- Evaluate tool success rates across different configurations
- Measure impact of configuration changes

### 5. Evaluation and Benchmarking

- Build datasets for model evaluation
- Calculate baseline success rates
- Track improvements over time

## Privacy Considerations

Trajectory logs contain:
- ✅ Tool names and arguments (may include file paths, commands)
- ✅ Model names and routing decisions
- ✅ Success/failure status
- ❌ Tool output content (not logged)
- ❌ User messages (only preview in route records)
- ❌ API keys or credentials (redacted)

**Warning**: Tool arguments may contain sensitive file paths or command parameters. Review trajectory logs before sharing.

To disable trajectory logging in sensitive environments:

```toml
[telemetry]
trajectory_logging = false
```

## Log Rotation

Trajectory logs are automatically rotated when they exceed the configured size:

```toml
[telemetry]
max_trajectory_log_size = 10485760  # 10MB (default)
trajectory_log_rotation_count = 5   # Keep 5 rotated files
```

Rotated files are named:
- `.vtcode/logs/trajectory.jsonl`
- `.vtcode/logs/trajectory.jsonl.1`
- `.vtcode/logs/trajectory.jsonl.2`
- etc.

## Related Documentation

- **[Telemetry Overview](./README.md)** - Complete observability guide
- **[Configuration Reference](../config.md#observability-and-telemetry)** - Telemetry configuration
- **[Router Configuration](../config/ROUTER.md)** - Understanding routing decisions
- **[Tool Specifications](../tools/TOOL_SPECS.md)** - Tool names and arguments

---

**Last Updated**: November 2025
**VTCode Version**: 0.43.6
