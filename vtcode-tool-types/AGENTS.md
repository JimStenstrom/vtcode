# vtcode-tool-types

[Root AGENTS.md](../AGENTS.md) | Shared runtime types for the tool system. Layer 0 crate — no vtcode dependencies.

## Module Groups

| Area | Modules |
|---|---|
| Tool Names | `tool_names` — canonical tool name constants and lookup |
| Result Types | `result_metadata` — EnhancedToolResult, ResultMetadata, scoring |
| Constants | `constants` — operational constants, error patterns, schemas |

## Rules

- This crate must remain Layer 0: zero vtcode-* dependencies.
- Used by both vtcode-core's `tools/` and `llm/` modules to break circular deps.
- Keep types minimal — if a type is only used in one module, it belongs in that module.

## Gotchas

- `CompactStr` is a re-export of `compact_str::CompactString` — used across the codebase.
- `ResultScorer` trait implementations live in vtcode-core, not here.
- `empty_object_schema()` returns a serde_json Value — used for default tool schemas.
