# Phase 1 Execution Guide - Breaking Circular Dependencies

**Goal**: Establish proper dependency hierarchy and eliminate circular dependencies between vtcode-core, vtcode-llm, and vtcode-tools.

**Timeline**: 1-2 weeks
**Parallel Branches**: 3
**Critical**: This phase blocks all subsequent work - must be executed correctly

---

## Overview

### Current Problem
```
vtcode-core ←───→ vtcode-llm    (circular!)
vtcode-core ←───→ vtcode-tools  (circular!)
```

### Target Solution
```
vtcode-llm-types ← vtcode-core ← vtcode-llm    ✅
vtcode-tool-traits ← vtcode-core ← vtcode-tools ✅
vtcode-commons (enhanced) ← all crates          ✅
```

---

## Branch 1.1: Extract LLM Types

### Branch Name
```bash
refactor/extract-llm-types-011CV<session-id>
```

### Checklist

#### 1. Create New Crate
```bash
cargo new --lib vtcode-llm-types
cd vtcode-llm-types
```

#### 2. Setup Cargo.toml
```toml
[package]
name = "vtcode-llm-types"
version = "0.43.6"
edition = "2024"
description = "Shared types for LLM provider implementations"
license = "MIT"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "2.0"
```

#### 3. Move Core Types (Order Matters!)

**Step A: Move Error Types First**
```bash
# Create error module
mkdir -p vtcode-llm-types/src
touch vtcode-llm-types/src/error.rs

# Copy from vtcode-core/src/llm/error.rs
# Move: LLMError, LLMErrorKind, LLMResult
```

**Step B: Move Token Types**
```bash
touch vtcode-llm-types/src/tokens.rs

# Copy from vtcode-core/src/llm/token_metrics.rs
# Move: TokenUsage, TokenMetrics, TokenCount
```

**Step C: Move Request/Response Types**
```bash
touch vtcode-llm-types/src/types.rs

# Copy from vtcode-core/src/llm/types.rs
# Move:
# - LLMRequest, LLMResponse
# - Message, MessageRole
# - Tool, ToolCall, ToolResult
# - StreamingResponse
```

**Step D: Move Provider Traits**
```bash
touch vtcode-llm-types/src/provider.rs

# Copy from vtcode-core/src/llm/provider.rs
# Move: AnyClient trait (interface only)
```

**Step E: Create lib.rs**
```rust
// vtcode-llm-types/src/lib.rs
pub mod error;
pub mod tokens;
pub mod types;
pub mod provider;

pub use error::{LLMError, LLMErrorKind, LLMResult};
pub use tokens::{TokenUsage, TokenMetrics, TokenCount};
pub use types::{LLMRequest, LLMResponse, Message, MessageRole};
pub use provider::AnyClient;
```

#### 4. Update vtcode-core Dependencies
```toml
# vtcode-core/Cargo.toml
[dependencies]
vtcode-llm-types = { path = "../vtcode-llm-types", version = "0.43.6" }
```

#### 5. Update vtcode-core Imports
```bash
# Find all files using moved types
cd vtcode-core
rg "use crate::llm::(error|types|provider)" -l

# Update each file:
# Old: use crate::llm::error::LLMError;
# New: use vtcode_llm_types::LLMError;
```

#### 6. Update vtcode-llm
```toml
# vtcode-llm/Cargo.toml
[dependencies]
vtcode-llm-types = { path = "../vtcode-llm-types", version = "0.43.6" }

# Remove vtcode-core dependency
# [dependencies]
# vtcode-core = { path = "../vtcode-core" }  # DELETE THIS
```

```bash
# Update imports in vtcode-llm
cd vtcode-llm
rg "use vtcode_core::llm" -l

# Replace with vtcode-llm-types imports
```

#### 7. Verify No Circular Dependencies
```bash
# Should compile independently
cd vtcode-llm-types && cargo build
cd vtcode-llm && cargo build  # Should work without vtcode-core!
cd vtcode-core && cargo build
```

#### 8. Run Tests
```bash
# Test each crate
cd vtcode-llm-types && cargo test
cd vtcode-llm && cargo test
cd vtcode-core && cargo test --lib

# Integration tests
cd .. && cargo test
```

### Success Criteria
- [ ] vtcode-llm-types compiles independently
- [ ] vtcode-llm no longer depends on vtcode-core
- [ ] vtcode-core uses vtcode-llm-types
- [ ] All tests pass
- [ ] No circular dependencies detected

### Files Affected
- **Created**: ~5 files in vtcode-llm-types/
- **Modified**: ~30 files in vtcode-core/
- **Modified**: ~10 files in vtcode-llm/
- **LOC**: ~2,000 moved

---

## Branch 1.2: Extract Tool Traits

### Branch Name
```bash
refactor/extract-tool-traits-011CV<session-id>
```

### Checklist

#### 1. Create New Crate
```bash
cargo new --lib vtcode-tool-traits
cd vtcode-tool-traits
```

#### 2. Setup Cargo.toml
```toml
[package]
name = "vtcode-tool-traits"
version = "0.43.6"
edition = "2024"
description = "Core traits and types for tool implementations"
license = "MIT"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
async-trait = "0.1"
```

#### 3. Move Tool Abstractions

**Step A: Move Tool Error Types**
```bash
mkdir -p vtcode-tool-traits/src
touch vtcode-tool-traits/src/error.rs

# Copy from vtcode-core/src/tools/error.rs
# Move: ToolError, ToolErrorKind, ToolResult
```

**Step B: Move Tool Types**
```bash
touch vtcode-tool-traits/src/types.rs

# Copy from vtcode-core/src/tools/types.rs
# Move:
# - ToolRequest, ToolResponse
# - ToolMetadata, ToolParameter
# - ToolExecutionContext
```

**Step C: Move Tool Traits**
```bash
touch vtcode-tool-traits/src/traits.rs

# Copy from vtcode-core/src/tools/registry/traits.rs
# Move:
# - Tool trait
# - ToolExecutor trait
# - ToolValidator trait
```

**Step D: Move Tool Policy Types**
```bash
touch vtcode-tool-traits/src/policy.rs

# Copy from vtcode-core/src/tool_policy.rs
# Move: ToolPolicy trait (interface only, not implementation)
```

**Step E: Create lib.rs**
```rust
// vtcode-tool-traits/src/lib.rs
pub mod error;
pub mod types;
pub mod traits;
pub mod policy;

pub use error::{ToolError, ToolErrorKind, ToolResult};
pub use types::{ToolRequest, ToolResponse, ToolMetadata};
pub use traits::{Tool, ToolExecutor, ToolValidator};
pub use policy::ToolPolicy;
```

#### 4. Update vtcode-core
```toml
# vtcode-core/Cargo.toml
[dependencies]
vtcode-tool-traits = { path = "../vtcode-tool-traits", version = "0.43.6" }
```

#### 5. Update vtcode-tools
```toml
# vtcode-tools/Cargo.toml
[dependencies]
vtcode-tool-traits = { path = "../vtcode-tool-traits", version = "0.43.6" }

# Remove vtcode-core dependency
# [dependencies]
# vtcode-core = { path = "../vtcode-core" }  # DELETE THIS
```

#### 6. Update Imports
```bash
# In vtcode-core
cd vtcode-core
rg "use crate::tools::(traits|types|error)" -l
# Update to use vtcode-tool-traits

# In vtcode-tools
cd vtcode-tools
rg "use vtcode_core::tools" -l
# Update to use vtcode-tool-traits
```

#### 7. Verify & Test
```bash
cd vtcode-tool-traits && cargo build && cargo test
cd vtcode-tools && cargo build && cargo test  # Should work!
cd vtcode-core && cargo build && cargo test
```

### Success Criteria
- [ ] vtcode-tool-traits compiles independently
- [ ] vtcode-tools no longer depends on vtcode-core
- [ ] vtcode-core uses vtcode-tool-traits
- [ ] All tests pass
- [ ] No circular dependencies

### Files Affected
- **Created**: ~5 files in vtcode-tool-traits/
- **Modified**: ~25 files in vtcode-core/
- **Modified**: ~8 files in vtcode-tools/
- **LOC**: ~1,500 moved

---

## Branch 1.3: Enhance Commons

### Branch Name
```bash
refactor/enhance-commons-011CV<session-id>
```

### Checklist

#### 1. Move Safety Utilities to Commons

**Step A: Move safety.rs**
```bash
cd vtcode-commons
mkdir -p src/safety
touch src/safety/mod.rs
touch src/safety/path_validator.rs
touch src/safety/workspace_boundary.rs

# Copy from vtcode-core/src/utils/safety.rs
# Split into:
# - path_validator.rs: Path validation logic
# - workspace_boundary.rs: Workspace checks
```

**Step B: Move gitignore handling**
```bash
touch src/safety/gitignore.rs

# Copy from vtcode-core/src/utils/vtcodegitignore.rs
# Move: Gitignore parsing and checking
```

#### 2. Create Safety Traits

```rust
// vtcode-commons/src/safety/path_validator.rs
use std::path::Path;
use anyhow::Result;

/// Validates that paths are safe to access
pub trait PathValidator {
    /// Validates a path is safe and within bounds
    fn validate_path(&self, path: &Path) -> Result<()>;

    /// Checks if path is absolute
    fn is_absolute(&self, path: &Path) -> bool;

    /// Checks if path is within workspace
    fn is_in_workspace(&self, path: &Path) -> bool;
}

/// Checks workspace boundaries
pub trait WorkspaceBoundary {
    /// Returns the workspace root
    fn workspace_root(&self) -> &Path;

    /// Checks if path escapes workspace
    fn is_path_escape(&self, path: &Path) -> bool;

    /// Resolves path relative to workspace
    fn resolve_in_workspace(&self, path: &Path) -> Result<std::path::PathBuf>;
}
```

#### 3. Update lib.rs
```rust
// vtcode-commons/src/lib.rs
mod buffer;

pub mod errors;
pub mod paths;
pub mod telemetry;
pub mod safety;  // NEW

pub use errors::{DisplayErrorFormatter, ErrorFormatter, ErrorReporter, MemoryErrorReporter, NoopErrorReporter};
pub use paths::{PathResolver, PathScope, StaticWorkspacePaths, WorkspacePaths};
pub use telemetry::{MemoryTelemetry, NoopTelemetry, TelemetrySink};
pub use safety::{PathValidator, WorkspaceBoundary};  // NEW
```

#### 4. Update vtcode-core
```rust
// vtcode-core - Update imports
// Old: use crate::utils::safety::validate_path;
// New: use vtcode_commons::safety::PathValidator;
```

#### 5. Remove Old Files
```bash
cd vtcode-core/src/utils
git rm safety.rs
git rm vtcodegitignore.rs
```

#### 6. Update All Crates Using Safety
```bash
# Update vtcode-tools, vtcode-bash-runner, etc.
# All should use vtcode-commons::safety
```

#### 7. Verify & Test
```bash
cd vtcode-commons && cargo build && cargo test
cd vtcode-core && cargo build && cargo test
cargo test --workspace
```

### Success Criteria
- [ ] vtcode-commons has safety module
- [ ] vtcode-commons has no vtcode-core dependency
- [ ] All safety logic centralized
- [ ] All crates updated
- [ ] All tests pass

### Files Affected
- **Created**: ~4 files in vtcode-commons/src/safety/
- **Deleted**: ~2 files from vtcode-core/src/utils/
- **Modified**: ~20 files across all crates
- **LOC**: ~800 moved

---

## Phase 1 Integration

### Integration Branch
```bash
git checkout dedupe/main
git checkout -b refactor/phase1-foundation
```

### Merge Order
```bash
# 1. Merge commons enhancement first (foundation)
git merge refactor/enhance-commons-011CV<id>

# 2. Merge type extractions (parallel, no conflicts)
git merge refactor/extract-llm-types-011CV<id>
git merge refactor/extract-tool-traits-011CV<id>
```

### Integration Tests
```bash
# Verify workspace structure
cargo metadata --no-deps | jq '.workspace_members'

# Verify no circular dependencies
cargo tree | grep -E "vtcode-(core|llm|tools)"

# Full test suite
cargo test --workspace --all-features

# Check for any remaining circular deps
cargo build --workspace
```

### Final Verification Checklist
- [ ] All 3 branches merged successfully
- [ ] No merge conflicts
- [ ] Workspace compiles clean
- [ ] All tests pass (cargo test --workspace)
- [ ] No circular dependencies (cargo tree)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

### Update Workspace Cargo.toml
```toml
# Cargo.toml (root)
[workspace]
members = [
    "vtcode-acp-client",
    "vtcode-core",
    "vtcode-commons",
    "vtcode-config",
    "vtcode-llm",
    "vtcode-llm-types",        # NEW
    "vtcode-tool-traits",      # NEW
    "vtcode-markdown-store",
    "vtcode-indexer",
    "vtcode-tools",
    "vtcode-bash-runner",
    "vtcode-exec-events",
]

[patch.crates-io]
vtcode-commons = { path = "vtcode-commons" }
vtcode-llm-types = { path = "vtcode-llm-types" }      # NEW
vtcode-tool-traits = { path = "vtcode-tool-traits" }  # NEW
# ... rest
```

---

## Common Issues & Solutions

### Issue 1: Type Mismatches After Move
**Symptom**: Compilation errors about missing types
**Solution**:
```bash
# Find all usages
rg "crate::llm::types::LLMRequest"
# Replace with
# vtcode_llm_types::LLMRequest
```

### Issue 2: Trait Import Conflicts
**Symptom**: Multiple definitions of same trait
**Solution**:
```rust
// Only import from new crate
use vtcode_tool_traits::Tool;
// NOT: use crate::tools::Tool;
```

### Issue 3: Circular Dependency Still Present
**Symptom**: `cargo build` shows cycle
**Solution**:
```bash
# Find the offending import
cargo tree | grep cycle
# Remove the dependency from Cargo.toml
# Update imports to use types crate
```

### Issue 4: Tests Failing After Move
**Symptom**: Test compilation errors
**Solution**:
```rust
// Update test imports
#[cfg(test)]
mod tests {
    use vtcode_llm_types::*;  // NEW
    // NOT: use super::super::llm::*;
}
```

---

## Documentation Updates

### Update README.md
```markdown
## Architecture

vtcode now has a modular architecture:

- **vtcode-llm-types**: Shared LLM types and traits
- **vtcode-tool-traits**: Tool system abstractions
- **vtcode-commons**: Common utilities including safety
- **vtcode-core**: Core agent runtime
- **vtcode-llm**: LLM provider implementations
- **vtcode-tools**: Tool implementations
```

### Update ARCHITECTURE.md
Document the new dependency structure.

### Update CHANGELOG.md
```markdown
## [Unreleased]

### Changed
- **BREAKING**: Extracted LLM types into `vtcode-llm-types` crate
- **BREAKING**: Extracted tool traits into `vtcode-tool-traits` crate
- **BREAKING**: Moved safety utilities to `vtcode-commons`

### Fixed
- Removed circular dependencies between core, llm, and tools crates

### Migration Guide
See `MIGRATION.md` for updating imports.
```

---

## Success Metrics

### Before Phase 1
```
Dependency Graph:
vtcode-core ←→ vtcode-llm (CIRCULAR!)
vtcode-core ←→ vtcode-tools (CIRCULAR!)

Compilation:
❌ vtcode-llm cannot compile independently
❌ vtcode-tools cannot compile independently
```

### After Phase 1
```
Dependency Graph:
vtcode-llm-types ← vtcode-core ← vtcode-llm ✅
vtcode-tool-traits ← vtcode-core ← vtcode-tools ✅
vtcode-commons ← all crates ✅

Compilation:
✅ vtcode-llm-types compiles independently
✅ vtcode-tool-traits compiles independently
✅ vtcode-llm compiles without core
✅ vtcode-tools compiles without core
✅ All tests pass
```

---

## Ready for Phase 2

Once Phase 1 integration is complete:
1. Merge to dedupe/main
2. Create Phase 2 branches from dedupe/main
3. Begin large subsystem extractions (UI, Prompts, MCP, Execution)

**Phase 1 is the critical foundation - take time to get it right!**
