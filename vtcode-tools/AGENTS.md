# vtcode-tools

[Root AGENTS.md](../AGENTS.md) | ACP tool integration for inter-agent communication.

## Modules

`acp_tool` ACP discovery, health, and execution tools | `compat` re-exports from vtcode-core for backward compatibility

## Rules

- Tool schemas belong in `vtcode-utility-tool-specs`, not here.
- All tool infrastructure (cache, middleware, patterns, executor, optimizer) now lives in `vtcode-core::tools`.
- This crate exists solely to bridge `vtcode-core` and `vtcode-acp` without creating a circular dependency.

## Gotchas

- `compat::current_timestamp_rfc3339` is a re-export from `vtcode_core::tools::time_compat`.
- No feature flags — this crate is always built with default features.
