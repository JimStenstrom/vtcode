# vtcode-pods

[Root AGENTS.md](../AGENTS.md) | GPU pod management. Layer 0 crate — depends only on vtcode-commons.

## Module Groups

| Area | Modules |
|---|---|
| Catalog | `catalog` — pod catalog and model discovery |
| Manager | `manager` — pod lifecycle management (start, stop, status) |
| State | `state` — pod state tracking and transitions |
| Store | `store` — persistent pod state storage |
| Transport | `transport` — SSH and command transport layers |

## Rules

- Zero internal vtcode dependencies beyond vtcode-commons.
- Re-export facade in vtcode-core (`pods/mod.rs`) must stay in sync.
- Uses `parking_lot` for mutex (not tokio::sync::Mutex) — all lock持有 are short.

## Gotchas

- `PodManager` requires `PodCatalog` at construction — catalog must be loaded first.
- `SshTransport` uses shell-words for command parsing.
- State persistence uses JSON files in the user's data directory.
