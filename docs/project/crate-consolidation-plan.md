# Crate Consolidation Plan

**Date:** 2026-06-14
**Scope:** All `vtcode-*` workspace crates
**Goal:** Reduce redundancy, eliminate duplicate types, merge trivially-small crates

---

## Current State

21 workspace members (20 `vtcode-*` + `xtask`). Audit found:

- 3 crates that are thin facades over `vtcode-core` (candidates for merge)
- 2 spec crates with identical patterns (candidates for merge)
- 5 type definitions duplicated across crates
- 3 type definitions duplicated within `vtcode-core` under the same name but different semantics

---

## Phase 1: Merge Spec Crates (Low Risk)

### 1.1 Merge `vtcode-collaboration-tool-specs` into `vtcode-utility-tool-specs` -- DONE

**Status:** Completed 2026-06-14.

**Approach:** Merged collaboration schemas as a `collaboration` submodule inside the existing `vtcode-utility-tool-specs` crate. This avoids creating a new crate and preserves all existing import paths for the utility crate.

**What changed:**
- `vtcode-utility-tool-specs/src/collaboration.rs` -- all 8 HITL/collaboration schema functions + 3 tests moved here
- `vtcode-utility-tool-specs/src/lib.rs` -- added `mod collaboration` + `pub use` re-exports
- All consumers updated to import from `vtcode_utility_tool_specs` instead of `vtcode_collaboration_tool_specs`
- `vtcode-collaboration-tool-specs/` directory deleted
- Documentation updated across `AGENTS.md`, `README.md`, `COMPATIBILITY.md`

---

## Phase 2: Consolidate Duplicate Types (Medium Risk)

### 2.1 Unify `ToolPolicy` to Single Definition -- DONE

**Status:** Completed 2026-06-14.

| Location | Action |
|---|---|
| `vtcode-config/src/core/tools.rs:222` | **Canonical** -- added `Default` derive with `#[default]` on `Prompt`, removed `Copy` |
| `vtcode-core/src/tool_policy.rs:47` | **Deleted** local enum, re-exports `pub use crate::config::core::tools::ToolPolicy;` |
| `vtcode-ui/src/tui/config/mod.rs:54` | **Deleted** local enum, re-exports `pub use vtcode_config::core::tools::ToolPolicy;` |

**What changed:**
- `vtcode-config/src/core/tools.rs`: Added `Default` derive, removed `Copy`, added `#[default]` on `Prompt`
- `vtcode-core/src/tool_policy.rs`: Removed local enum, added re-export, removed `ConfigToolPolicy` alias, simplified conversion functions (now identity)
- `vtcode-core/src/tools/registry/policy_facade.rs`: Updated `*policy` to `policy.clone()` (Copy removal)
- `vtcode-ui/src/tui/config/mod.rs`: Removed local enum, added re-export

### 2.2 Consolidate `SandboxPermissions` Within `vtcode-core` -- DONE

**Status:** Completed 2026-06-14.

| Location | Action |
|---|---|
| `vtcode-core/src/sandboxing/permissions.rs:13` | **Kept** as canonical (4 variants) |
| `vtcode-core/src/tools/handlers/tool_handler.rs:63` | **Deleted** 3-variant enum, re-exports from `crate::sandboxing::SandboxPermissions` |

### 2.3 Consolidate `RetryPolicy` Within `vtcode-core` -- DONE

**Status:** Completed 2026-06-14.

| Location | Action |
|---|---|
| `vtcode-core/src/retry.rs:17` | **Kept** as canonical (5 fields) |
| `vtcode-core/src/components.rs:434` | **Deleted** 3-field struct, re-exports `crate::retry::RetryPolicy` |

**Additional changes:**
- `retry.rs`: Added `Copy` to derive list (all fields are Copy types)
- `components.rs`: `ExponentialBackoffRetry::backoff_duration` now delegates to `policy.delay_for_attempt()`
- `components/tests.rs`: Updated retry test to use canonical field names

### 2.4 Rename Shadow Types in `vtcode-core` -- DONE

**Status:** Completed 2026-06-14.

| Original | Renamed To | File |
|---|---|---|
| `ToolResult` | `MiddlewareToolResult` | `tools/async_middleware.rs` |
| `ToolMetadata` | `CachedToolMetadata` | `tools/optimized_registry.rs` |
| `ToolMetadata` | `ToolRegistrationSpec` | `tools/registry/registration.rs` |
| `ErrorSeverity` | `ImprovementSeverity` | `tools/improvements_errors.rs` |

Backward-compatible aliases maintained in re-exports (`tools/mod.rs`, `tools/registry/mod.rs`).

---

## Phase 3: Evaluate Facade Crates

### 3.1 Evaluate `vtcode-llm` for Merge into `vtcode-core` -- DONE

**Status:** Completed 2026-06-14.

**Findings:**
- Zero consumers within the workspace -- no crate imports from `vtcode_llm`
- Marked `publish = false` -- no external consumers
- 75% of source is tests (~500 of 796 lines in config.rs)
- Unique items: `ProviderConfig` trait (~150 lines), `StaticResponseClient` mock (~134 lines)
- All other exports are pass-through re-exports from `vtcode-core` and `vtcode-commons`

**What changed:**
1. `ProviderConfig` trait + adapter moved into `vtcode_core::llm::config_adapter`
2. `StaticResponseClient` moved into `vtcode_core::llm` behind `mock` feature
3. `vtcode-llm` removed from workspace members and `clippy.toml`
4. `vtcode-llm/` directory deleted

### 3.2 Evaluate `vtcode-tools` for Merge into `vtcode-core` -- DONE

**Status:** Completed 2026-06-14. `vtcode-tools` stripped to ACP-only.

**Findings:**
- Zero consumers within the workspace -- no crate imports from `vtcode_tools`
- 2,633 lines of original code across 8 modules (cache, middleware, executor, patterns, optimizer, acp_tool, compat, adapters)
- All re-exports serve no indirection purpose (no external consumers)
- The only complication: `acp_tool.rs` depends on `vtcode-acp`, which already depends on `vtcode-core` (circular dependency risk)

**What changed:**
1. `cache.rs`, `middleware.rs`, `patterns.rs`, `executor.rs`, `optimizer.rs` moved into `vtcode-core/src/tools/`
2. `compat.rs` moved as utility
3. `adapters.rs` moved behind existing policy feature gates
4. `vtcode-tools` retains only ACP-specific tools (`acp_tool`)

---

## Validation Checklist

- [x] `cargo check` passes for all affected crates (`vtcode-config`, `vtcode-core`, `vtcode-ui`, `vtcode-tools`)
- [ ] `cargo test --workspace` passes (no new failures)
- [ ] `cargo clippy --workspace` passes
- [x] No duplicate type definitions remain for consolidated types
- [x] Import paths updated in all consumers
- [x] `Cargo.toml` workspace members list is clean
- [x] `ConfigToolPolicy` alias removed from `vtcode-core/src/tools/registry/mod.rs`
- [x] `AGENTS.md` updated: `vtcode-collaboration-tool-specs` entries removed
- [x] Zero stale references to deleted `vtcode-collaboration-tool-specs` across codebase

---

## Out of Scope

- Merging `vtcode-indexer` + `vtcode-file-search` (different concerns: persistent indexing vs fuzzy UI search)
- Merging `vtcode-commons` into anything (foundation crate, correct as leaf)
- Merging `vtcode-bash-runner` into core (substantial independent logic)
- Merging `vtcode-ui` into core (large TUI crate, correct boundary)
- Merging `vtcode-acp` into core (protocol-specific, substantial)
