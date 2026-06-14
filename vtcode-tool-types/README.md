# vtcode-tool-types

Shared runtime types for the VT Code tool system. Breaks circular dependencies
between LLM and tool modules by providing a minimal, dependency-free type
catalog that both sides can import.

## Modules

| Module | Purpose |
|---|---|
| `tool_names` | Canonical tool name constants and lookup |
| `result_metadata` | `EnhancedToolResult`, `ResultMetadata`, `ResultCompleteness`, scoring |
| `constants` | Operational constants (capacities, error patterns, schemas) |

## Public entrypoints

### Types

- `CompactStr` – space-efficient string type alias
- `EnhancedToolResult` – tool execution result with metadata
- `ResultMetadata` – execution metadata (duration, output size, etc.)
- `ResultCompleteness` – completeness classification for results
- `ResultScorer` – trait for scoring tool result quality

### Constants

- `UNIFIED_SEARCH`, `UNIFIED_EXEC`, `RUN_PTY_CMD` – canonical tool names
- `DEFAULT_VEC_CAPACITY`, `MAX_SEARCH_RESULTS` – operational limits
- `ERROR_DETECTION_PATTERNS`, `NETWORK_ERROR_PATTERNS` – error classification

## Usage

```rust
use vtcode_tool_types::{EnhancedToolResult, ResultCompleteness};

let result = EnhancedToolResult::new("output".into());
assert_eq!(result.completeness(), ResultCompleteness::Complete);
```

## API reference

<https://docs.rs/vtcode-tool-types>
