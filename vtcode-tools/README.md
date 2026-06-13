# vtcode-tools

ACP tool integration for inter-agent communication.

`vtcode-tools` bridges `vtcode-core` and `vtcode-acp` to provide tools for
agent discovery, health checks, and cross-agent execution. All other tool
infrastructure (cache, middleware, patterns, executor, optimizer) has been
merged into `vtcode-core`.

## Public entrypoints

- **ACP tools** — `AcpTool`, `AcpDiscoveryTool`, `AcpHealthTool`
- **Compat** — `current_timestamp_rfc3339` (re-export from `vtcode-core`)

## Usage

```rust
use vtcode_tools::{AcpTool, AcpDiscoveryTool, AcpHealthTool};
```

## Related docs

- [Tool extraction policy](../docs/modules/vtcode_tools_policy.md)
- [Crate consolidation plan](../docs/project/crate-consolidation-plan.md)
