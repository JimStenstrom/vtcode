# Phase 2: vtcode-ui Extraction - Progress Report

**Started**: November 13, 2025
**Branch**: `claude/vtcode-ui-phase-2-011CV61uxFU7JtyYwLxQbLk6`
**Status**: 🔄 IN PROGRESS - Initial Structure Created

---

## Overview

This document tracks the progress of extracting the `vtcode-ui` subsystem (approximately 18K LOC) from `vtcode-core` into an independent crate, as outlined in Phase 2 of the ARCHITECTURE_TRANSFORMATION.md plan.

---

## Work Completed

### 1. ✅ Phase 1 Foundation Merged
- Merged `claude/deduplicate-crates-011CV5uvkR9chFAHikx2brdh` branch
- Gained access to Phase 1 improvements:
  - `vtcode-llm-types` crate
  - `vtcode-tool-traits` crate
  - Enhanced `vtcode-commons` with safety utilities
  - Circular dependency fixes

### 2. ✅ vtcode-ui Crate Structure Created
Created new crate at `/vtcode-ui/` with:
- **Cargo.toml**: Configured with necessary UI dependencies
  - Terminal/TUI: crossterm, ratatui, vt100, portable-pty
  - ANSI styling: anstyle suite
  - Syntax: syntect, pulldown-cmark
  - Theming: catppuccin
  - Search: perg, nucleo-matcher
  - VTCode dep: vtcode-commons

### 3. ✅ UI Code Copied
Copied all UI code from `vtcode-core/src/ui/` to `vtcode-ui/src/`:
- 16 Rust source files (~18K LOC)
- Complete TUI directory with session management
- Files include:
  - `lib.rs` (was mod.rs)
  - `diff_renderer.rs`
  - `file_colorizer.rs`
  - `git_config.rs`
  - `markdown.rs`
  - `search.rs`
  - `slash.rs`
  - `styled.rs`
  - `terminal.rs`
  - `theme.rs`
  - `theme_config.rs`
  - `theme_manager.rs`
  - `tui.rs`
  - `user_confirmation.rs`
  - `tui/` directory with 20+ modules

### 4. ✅ Workspace Configuration Updated
Modified root `Cargo.toml`:
- Added `vtcode-ui` to workspace members
- Added `vtcode-ui` to patch.crates-io section

---

## Challenges Discovered

### 1. Deep vtcode-core Dependencies
The UI code has extensive internal dependencies on `vtcode-core`:

**Example imports found**:
```rust
use crate::config::constants::{defaults, ui}
use crate::agent::runloop::*
use crate::acp::permissions::*
use crate::hooks::lifecycle::*
```

These dependencies span:
- Configuration constants
- Agent runloop modules
- ACP (Agent Client Protocol) modules
- Lifecycle hooks
- Tool management
- MCP support

### 2. Required Refactoring Strategy
To properly extract vtcode-ui, we need to:

1. **Identify Shared Dependencies**
   - Configuration constants needed by UI
   - Types/traits from agent modules
   - ACP integration points

2. **Choose Extraction Approach**
   - Option A: Move shared code to vtcode-commons
   - Option B: Create intermediate types crate (vtcode-types?)
   - Option C: Pass dependencies as traits/generics
   - Option D: Keep some circular dependency temporarily

3. **Update All Imports**
   - Change `crate::` imports to `vtcode_core::`
   - Re-export necessary types
   - Create adapter layer if needed

---

## Next Steps

### Immediate (This Session)
1. Create detailed dependency analysis
2. Document all `crate::` references
3. Commit current work as WIP
4. Push to branch

### Short Term (Next Session)
1. Analyze which vtcode-core dependencies are essential
2. Choose refactoring strategy
3. Create vtcode-ui-core or vtcode-types if needed
4. Begin systematic import fixes

### Medium Term (Weeks)
1. Extract configuration dependencies
2. Create trait-based abstractions for agent integration
3. Update vtcode-core to use vtcode-ui
4. Ensure compilation passes
5. Run test suite
6. Update documentation

---

## File Structure

### Current vtcode-ui Structure
```
vtcode-ui/
├── Cargo.toml (configured)
└── src/
    ├── lib.rs (module exports)
    ├── diff_renderer.rs
    ├── file_colorizer.rs
    ├── git_config.rs
    ├── markdown.rs
    ├── search.rs
    ├── slash.rs
    ├── styled.rs
    ├── terminal.rs
    ├── theme.rs
    ├── theme_config.rs
    ├── theme_manager.rs
    ├── tui.rs
    ├── user_confirmation.rs
    └── tui/
        ├── log.rs
        ├── modern_tui.rs
        ├── panic_hook.rs
        ├── session.rs
        ├── style.rs
        ├── theme_parser.rs
        ├── tui.rs
        ├── types.rs
        └── session/
            ├── config.rs
            ├── error.rs
            ├── header.rs
            ├── message.rs
            ├── modal.rs
            ├── navigation.rs
            ├── performance.rs
            ├── queue.rs
            ├── slash.rs
            └── state.rs (planned refactoring)
```

---

## Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| vtcode-core LOC reduction | -12K | 0 | ⏳ Pending |
| vtcode-ui crate created | Yes | ✅ Yes | ✅ Complete |
| Compilation passing | Yes | ❌ No | 🔄 In Progress |
| Dependencies resolved | 100% | ~30% | 🔄 In Progress |
| Tests passing | 100% | N/A | ⏳ Pending |

---

## Dependencies Analysis

### vtcode-ui Dependencies on vtcode-core Modules

Based on initial analysis, the UI code references:

1. **Config Module** (`crate::config::*`)
   - Constants (defaults, ui values)
   - Configuration types

2. **Agent Module** (`crate::agent::*`)
   - Runloop types and functions
   - Context management
   - Tool execution

3. **ACP Module** (`crate::acp::*`)
   - Permissions
   - Workspace trust
   - Tooling

4. **Hooks Module** (`crate::hooks::*`)
   - Lifecycle events
   - Hook messages

5. **External Crates**
   - agent-client-protocol
   - chrono
   - clap
   - Other UI deps already in Cargo.toml

---

## Recommended Approach

### Phase 2A: Preparation (Current)
1. ✅ Create vtcode-ui structure
2. ✅ Copy code
3. 🔄 Document all dependencies
4. Create dependency resolution plan

### Phase 2B: Foundation Types
1. Create shared types crate or enhance vtcode-commons
2. Move constants needed by UI
3. Define trait boundaries

### Phase 2C: Gradual Migration
1. Fix imports one module at a time
2. Start with leaf modules (no internal dependencies)
3. Work up to complex modules (tui/session/)

### Phase 2D: Integration
1. Update vtcode-core to import vtcode-ui
2. Remove old ui/ directory from vtcode-core
3. Update all references across codebase

### Phase 2E: Testing & Documentation
1. Ensure all tests pass
2. Update architecture docs
3. Create migration guide
4. Benchmark compilation time improvement

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking API changes | High | High | Careful trait design, version management |
| Compilation issues | High | Medium | Incremental approach, frequent testing |
| Performance regression | Low | Medium | Benchmarking, profiling |
| Circular dependencies | Medium | High | Careful architecture review |

---

## Timeline Estimate

Based on complexity discovered:

- **Week 1** (Current): Initial setup + dependency analysis ← WE ARE HERE
- **Week 2**: Foundation types + begin migration
- **Week 3**: Complete UI module migration
- **Week 4**: vtcode-core integration + testing
- **Week 5**: Documentation + cleanup

**Total**: 4-5 weeks for complete vtcode-ui extraction

---

## Notes

- This extraction is more complex than initially anticipated due to deep integration
- Phase 1's work on breaking circular dependencies provides a good foundation
- The modular architecture will make future phases (Phase 3-5) significantly easier
- Consider creating vtcode-ui-types or enhancing vtcode-commons for shared types

---

## Next Session Plan

1. Create comprehensive dependency map
2. Identify minimum viable interface
3. Design trait-based architecture
4. Begin systematic refactoring

---

**Status**: 🟡 PARTIALLY COMPLETE - Foundation laid, implementation in progress
**Confidence**: 🟢 HIGH - Clear path forward identified
**Next Update**: After dependency analysis complete

---

Version: 1.0
Created: November 13, 2025
