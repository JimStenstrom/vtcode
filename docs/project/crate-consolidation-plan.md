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

### 1.1 Merge `vtcode-collaboration-tool-specs` + `vtcode-utility-tool-specs` -> `vtcode-tool-specs`

**Why:** Both are passive JSON schema crates (~500 lines combined). `vtcode-utility-tool-specs` already re-exports collaboration schemas internally. Two crates for the same pattern adds Cargo.toml noise with no benefit.

**Steps:**

1. Create `vtcode-tool-specs/` with a merged `Cargo.toml` combining dependencies:
   - `serde_json` (from both)
   - `rmcp` (from utility)
2. Move all source files into `vtcode-tool-specs/src/`:
   - `collaboration.rs` (from `vtcode-collaboration-tool-specs/src/lib.rs`)
   - `utility.rs` (from `vtcode-utility-tool-specs/src/lib.rs`)
   - `lib.rs` re-exports both modules
3. Update `Cargo.toml` workspace members: remove old two, add `vtcode-tool-specs`
4. Update `[patch.crates-io]` section
5. Update all downstream consumers:
   - `vtcode-core/src/tools/` — replace `use vtcode_collaboration_tool_specs` and `use vtcode_utility_tool_specs` with `use vtcode_tool_specs::{collaboration, utility}`
   - `vtcode-tools/src/lib.rs` — update re-exports
6. Delete old crate directories
7. Run `cargo check --workspace` and fix any import paths

**Files affected:**
- `Cargo.toml` (workspace root)
- `vtcode-tool-specs/Cargo.toml` (new)
- `vtcode-tool-specs/src/lib.rs` (new)
- `vtcode-tool-specs/src/collaboration.rs` (moved)
- `vtcode-tool-specs/src/utility.rs` (moved)
- `vtcode-core/Cargo.toml` (dependency update)
- `vtcode-core/src/tools/*.rs` (import path updates)
- `vtcode-tools/Cargo.toml` (dependency update)
- `vtcode-tools/src/lib.rs` (re-export updates)

**Estimated effort:** Small (<1hr)
**Risk:** Low — mechanical rename, no logic changes

---

## Phase 2: Consolidate Duplicate Types (Medium Risk)

### 2.1 Unify `ToolPolicy` to Single Definition

**Why:** Identical `Allow | Prompt | Deny` enum defined in 3 crates independently.

| Location | Action |
|---|---|
| `vtcode-config/src/core/tools.rs:225` | **Keep** as canonical definition |
| `vtcode-core/src/tool_policy.rs:47` | **Delete**, re-export from `vtcode-config` |
| `vtcode-ui/src/tui/config/mod.rs:54` | **Delete**, re-export from `vtcode-config` |

**Steps:**

1. Verify all 3 definitions have identical variants and derive attributes
2. In `vtcode-core/src/tool_policy.rs`: replace local enum with `pub use vtcode_config::core::tools::ToolPolicy;`
3. In `vtcode-ui/src/tui/config/mod.rs`: replace local enum with `pub use vtcode_config::core::tools::ToolPolicy;`
4. Update any `use` statements in downstream code that reference the old paths
5. Run `cargo check --workspace`

**Files affected:**
- `vtcode-core/src/tool_policy.rs`
- `vtcode-core/src/lib.rs` (re-export)
- `vtcode-ui/src/tui/config/mod.rs`
- `vtcode-ui/src/lib.rs` (re-export)
- Any files importing `ToolPolicy` from the old paths

**Estimated effort:** Small (<1hr)
**Risk:** Medium — need to verify no behavioral differences in derives/impls

### 2.2 Consolidate `SandboxPermissions` Within `vtcode-core`

**Why:** Two definitions in the same crate. The `sandboxing/permissions.rs` version is a strict superset (4 variants + helpers vs 3 variants).

| Location | Action |
|---|---|
| `vtcode-core/src/sandboxing/permissions.rs:13` | **Keep** as canonical |
| `vtcode-core/src/tools/handlers/tool_handler.rs:63` | **Delete**, use canonical |

**Steps:**

1. Remove the 3-variant enum from `tool_handler.rs`
2. Add `use crate::sandboxing::permissions::SandboxPermissions;` in `tool_handler.rs`
3. Verify all match arms in `tool_handler.rs` are updated (the missing `BypassSandbox` variant needs a match arm)
4. Run `cargo check -p vtcode-core`

**Estimated effort:** Small (<30min)
**Risk:** Low — same crate, superset relationship

### 2.3 Consolidate `RetryPolicy` Within `vtcode-core`

**Why:** Two definitions in the same crate. `retry.rs` is a strict superset (5 fields vs 3).

| Location | Action |
|---|---|
| `vtcode-core/src/retry.rs:17` | **Keep** as canonical |
| `vtcode-core/src/components.rs:434` | **Delete**, delegate to `retry.rs` |

**Steps:**

1. Remove the 3-field struct from `components.rs`
2. Re-export `crate::retry::RetryPolicy` from `components.rs` if needed for backward compat
3. Update `HasRetryPolicy` trait to use `crate::retry::RetryPolicy`
4. Run `cargo check -p vtcode-core`

**Estimated effort:** Small (<30min)
**Risk:** Low — same crate, superset relationship

### 2.4 Rename Shadow Types in `vtcode-core`

**Why:** Same name, different semantics causes confusion. Not true duplicates — they serve different subsystems.

| Current Name | Location | Rename To |
|---|---|---|
| `ToolResult` | `tools/async_middleware.rs:43` | `MiddlewareToolResult` |
| `ToolMetadata` | `tools/optimized_registry.rs:15` | `CachedToolMetadata` |
| `ToolMetadata` | `tools/registry/registration.rs:34` | `ToolRegistrationSpec` |
| `ErrorSeverity` | `tools/improvements_errors.rs:71` | `ImprovementSeverity` |

**Steps:**

1. For each type, rename the definition and update all references within `vtcode-core`
2. Use `ast-grep` to find all usages before renaming
3. Run `cargo check -p vtcode-core` after each rename

**Estimated effort:** Small (<1hr total)
**Risk:** Low — internal renames only

---

## Phase 3: Evaluate Facade Crates (Needs Analysis)

### 3.1 Evaluate `vtcode-llm` for Merge into `vtcode-core`

**Why:** Almost entirely re-exports from `vtcode-core` + `vtcode-commons`. Adds only `ProviderConfig` trait.

**Decision criteria:**
- If no external crate (outside this workspace) depends on `vtcode-llm` independently -> **merge**
- If `vtcode-llm` provides meaningful API surface separation -> **keep**

**Steps:**

1. Search for crates that depend on `vtcode-llm` but NOT `vtcode-core`
2. Count the unique items `vtcode-llm` adds beyond re-exports (likely just `ProviderConfig` trait)
3. If merge: move `ProviderConfig` trait into `vtcode-core::llm`, delete `vtcode-llm`
4. Update workspace members and patches

**Estimated effort:** Medium (1-2hr)
**Risk:** Medium — may have external consumers

### 3.2 Evaluate `vtcode-tools` for Merge into `vtcode-core`

**Why:** Same facade pattern. Adds `Middleware`, `CachedToolExecutor`, `WorkflowOptimizer` on top of re-exports.

**Decision criteria:**
- Same as 3.1 — check for external-only dependents
- The added types (`Middleware`, `CachedToolExecutor`) are small enough to live in core

**Steps:**

1. Search for crates that depend on `vtcode-tools` but NOT `vtcode-core`
2. If merge: move `Middleware`, `CachedToolExecutor`, `WorkflowOptimizer` into `vtcode-core::tools`
3. Update workspace members and patches

**Estimated effort:** Medium (1-2hr)
**Risk:** Medium — may have external consumers

---

## Execution Order

```
Phase 1 (merge spec crates)        -- independent, do first
    |
Phase 2.1 (ToolPolicy)             -- independent of 1.1
Phase 2.2 (SandboxPermissions)     -- independent of 1.1, 2.1
Phase 2.3 (RetryPolicy)            -- independent of 1.1, 2.1, 2.2
Phase 2.4 (rename shadow types)    -- independent of above
    |
Phase 3.1 (evaluate vtcode-llm)    -- after Phase 1 (spec crate merge may affect dependency graph)
Phase 3.2 (evaluate vtcode-tools)  -- after Phase 1, after 3.1
```

Phases 1 and 2 can run in parallel. Phase 3 depends on Phase 1 completing.

---

## Validation Checklist

After each phase:

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes (or at least no new failures)
- [ ] `cargo clippy --workspace` passes
- [ ] No duplicate type definitions remain (grep for the consolidated names)
- [ ] Import paths updated in all consumers
- [ ] `Cargo.toml` workspace members list is clean

---

## Out of Scope

- Merging `vtcode-indexer` + `vtcode-file-search` (different concerns: persistent indexing vs fuzzy UI search)
- Merging `vtcode-commons` into anything (foundation crate, correct as leaf)
- Merging `vtcode-bash-runner` into core (substantial independent logic)
- Merging `vtcode-ui` into core (large TUI crate, correct boundary)
- Merging `vtcode-acp` into core (protocol-specific, substantial)
