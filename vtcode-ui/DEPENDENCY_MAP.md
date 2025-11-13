# vtcode-ui Dependency Map - Visual Reference

**Purpose**: Quick visual reference for vtcode-ui's dependencies
**Generated**: November 13, 2025

---

## Current Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                          vtcode-ui                               │
│                        (19K LOC, 42 files)                       │
└────────┬──────────────────┬──────────────┬────────────┬─────────┘
         │                  │              │            │
         │                  │              │            │
    ┌────▼─────┐       ┌────▼────┐   ┌────▼───┐  ┌────▼────┐
    │ Config   │       │ Prompts │   │ Tools  │  │  Utils  │
    │ (12 dep) │       │ (6 dep) │   │(5 dep) │  │ (4 dep) │
    └────┬─────┘       └────┬────┘   └────┬───┘  └────┬────┘
         │                  │              │           │
         │                  │              │           │
    vtcode-core::config     │              │           │
         │                  │              │           │
         │         vtcode-core::prompts    │           │
         │                  │              │           │
         │                  │      vtcode-core::tools  │
         │                  │              │           │
         │                  │              │    vtcode-core::utils
         │                  │              │           │
         └──────────────────┴──────────────┴───────────┘
                                  │
                     ┌────────────▼───────────┐
                     │      vtcode-core       │
                     │    (All Modules)       │
                     └────────────────────────┘
```

**Status**: 🔴 Tightly Coupled - Needs Refactoring

---

## Target Dependency Graph (After Refactoring)

```
┌─────────────────────────────────────────────────────────────────┐
│                          vtcode-ui                               │
│                    (Independent Crate)                           │
└────────┬──────────────────┬──────────────┬────────────┬─────────┘
         │                  │              │            │
         │                  │              │            │
    ┌────▼─────┐       ┌────▼────┐   ┌────▼───────┐  ┌────▼────┐
    │UI Consts │       │UI Types │   │Tool Traits │  │Built-in │
    └────┬─────┘       └────┬────┘   └────┬───────┘  └─────────┘
         │                  │              │
         │                  │              │
    ┌────▼──────────────────▼──────────────▼──────────────────┐
    │              vtcode-commons                              │
    │    (Shared types, constants, traits)                     │
    └──────────────────────────────────────────────────────────┘
                                  │
                     ┌────────────▼───────────┐
                     │  vtcode-tool-traits    │
                     │   (Tool types)         │
                     └────────────────────────┘

         ┌──────────────────────────────────────┐
         │         vtcode-core                   │
         │   (Imports vtcode-ui, not reverse)   │
         └──────────────────────────────────────┘
```

**Status**: 🟢 Loosely Coupled - Clean Architecture

---

## Dependency Resolution Roadmap

### 1. Config Module Dependencies (12 imports)

**Current**:
```
vtcode-ui → crate::config::constants::{ui, defaults, prompts}
vtcode-ui → crate::config::types::{UiSurfacePreference, ReasoningEffortLevel}
vtcode-ui → crate::config::loader::SyntaxHighlightingConfig
```

**After Phase 2A**:
```
vtcode-ui → vtcode-commons::ui::constants::{ui, defaults, prompts}
vtcode-ui → vtcode-commons::ui::types::{UiSurfacePreference, ReasoningEffortLevel}
vtcode-ui → vtcode-config::SyntaxHighlightingConfig
```

**Action Items**:
- [ ] Move constants to vtcode-commons/src/ui/constants.rs
- [ ] Move types to vtcode-commons/src/ui/types.rs
- [ ] Keep loader in vtcode-config (already independent)

---

### 2. Prompts Module Dependencies (6 imports)

**Current**:
```
vtcode-ui → crate::prompts::CustomPrompt
vtcode-ui → crate::prompts::CustomPromptRegistry
```

**After Phase 2B** (Coordinate with prompts team):
```
vtcode-ui → vtcode-prompts::CustomPrompt       (if prompts crate exists)
         OR
vtcode-ui defines: trait PromptProvider        (UI defines interface)
vtcode-core implements: PromptProvider          (core provides data)
```

**Action Items**:
- [ ] Wait for prompts team to create vtcode-prompts crate
- [ ] Or create trait interface in vtcode-ui
- [ ] Coordinate on type definitions

---

### 3. Tools Module Dependencies (5 imports)

**Current**:
```
vtcode-ui → crate::tools::TaskPlan
vtcode-ui → crate::tools::PlanSummary
```

**After Phase 2A**:
```
vtcode-ui → vtcode-tool-traits::TaskPlan
vtcode-ui → vtcode-tool-traits::PlanSummary
```

**Action Items**:
- [ ] Move TaskPlan to vtcode-tool-traits/src/types.rs
- [ ] Move PlanSummary to vtcode-tool-traits/src/types.rs
- [ ] Update imports in vtcode-ui

**Rationale**: Phase 1 already created vtcode-tool-traits - perfect home!

---

### 4. Utils Module Dependencies (4 imports)

**Current**:
```
vtcode-ui → crate::utils::CachedStyleParser
vtcode-ui → crate::utils::colors::style
vtcode-ui → crate::utils::style_helpers::style_from_color_name
```

**After Phase 2C**:
```
vtcode-ui/src/utils.rs contains:
  - CachedStyleParser (moved from core)
  - style functions (moved from core)
  - style_from_color_name (moved from core)
```

**Action Items**:
- [ ] Copy utils into vtcode-ui/src/utils.rs
- [ ] Update internal imports to use local utils
- [ ] Remove crate::utils:: references

**Rationale**: These are UI-specific utilities, belong in vtcode-ui

---

## Files Requiring Updates

### Critical Path Files (Update First)

```
Priority 1: Core Data Flows
├── src/theme.rs                     (2 deps: config constants)
├── src/tui/session.rs               (4 deps: config, prompts, tools)
├── src/tui/types.rs                 (2 deps: config, tools)
└── src/tui/session/state.rs         (2 deps: tools, prompts)

Priority 2: Feature Implementations
├── src/markdown.rs                  (2 deps: config, theme)
├── src/tui/modern_integration.rs    (3 deps: session, types, config)
├── src/tui/session/modal.rs         (3 deps: config, search, types)
└── src/tui/session/slash_palette.rs (3 deps: prompts, search, slash)

Priority 3: Utilities
├── src/theme_config.rs              (1 dep: CachedStyleParser)
├── src/diff_renderer.rs             (2 deps: git_config, style_helpers)
└── src/user_confirmation.rs         (1 dep: colors)
```

### Bulk Update Files (Update Last)

```
Remaining 31 files:
└── Internal crate::ui::* references
    Action: Simple search/replace
    Pattern: s/crate::ui::/crate::/g  (or vtcode_ui::)
```

---

## Integration Timeline

### Week 1: Foundation (Nov 13-17)
```
Day 1-2: ✅ Dependency analysis (DONE)
Day 3-4: Move constants to vtcode-commons
Day 5:   Move types to vtcode-commons
```

### Week 2: Types & Traits (Nov 18-24)
```
Day 1-2: Move TaskPlan/PlanSummary to vtcode-tool-traits
Day 3-4: Coordinate with prompts team OR create traits
Day 5:   Move utils into vtcode-ui
```

### Week 3: Import Updates (Nov 25-Dec 1)
```
Day 1-2: Update Priority 1 files (critical path)
Day 3-4: Update Priority 2 files (features)
Day 5:   Update Priority 3 + bulk files
```

### Week 4: Integration (Dec 2-8)
```
Day 1-2: Verify vtcode-ui compiles independently
Day 3-4: Integrate with vtcode-core
Day 5:   Testing & documentation
```

---

## Dependency Metrics by Category

### 1. Pure Data (Safe to Move)
| Type | Count | Target | Effort |
|------|-------|--------|--------|
| Constants | 7 | vtcode-commons | 2 hours |
| Type Enums | 2 | vtcode-commons | 1 hour |
| Structs | 3 | vtcode-tool-traits | 2 hours |

**Total**: 12 items, ~5 hours effort

### 2. Behavioral (Need Abstraction)
| Type | Count | Strategy | Effort |
|------|-------|----------|--------|
| Registries | 2 | Trait interfaces | 4 hours |
| Loaders | 1 | Pass as param | 1 hour |

**Total**: 3 items, ~5 hours effort

### 3. Utilities (Move to vtcode-ui)
| Type | Count | Action | Effort |
|------|-------|--------|--------|
| Style utils | 3 | Copy to vtcode-ui | 2 hours |
| Parser | 1 | Copy to vtcode-ui | 1 hour |

**Total**: 4 items, ~3 hours effort

### 4. Internal References (Bulk Update)
| Type | Count | Action | Effort |
|------|-------|--------|--------|
| crate::ui:: | ~100+ | Search/replace | 2 hours |

**Total**: ~2 hours effort

---

## Grand Total

| Phase | Effort (hours) | Risk |
|-------|----------------|------|
| Phase 2A: Move data | 5 | Low |
| Phase 2B: Abstractions | 5 | Medium |
| Phase 2C: Utilities | 3 | Low |
| Phase 2D: Imports | 2 | Low |
| Phase 2E: Integration | 8 | Medium |
| **Total** | **23 hours** | **Low-Medium** |

**Timeline**: 3-4 weeks (part-time work)

---

## Coordination Points

### With vtcode-prompts Team
**Topic**: CustomPrompt and CustomPromptRegistry types
**Options**:
1. They create vtcode-prompts with these types
2. We create trait in vtcode-ui for them to implement
3. We move types to vtcode-commons temporarily

**Status**: ⏸️ Waiting for coordination

### With vtcode-execution Team
**Topic**: TaskPlan and PlanSummary types
**Options**:
1. Move to vtcode-tool-traits (RECOMMENDED)
2. Move to vtcode-commons
3. Keep in vtcode-core (NOT RECOMMENDED)

**Status**: ✅ Can proceed independently

### With vtcode-mcp Team
**Topic**: No direct dependencies
**Status**: ✅ No coordination needed

---

## Success Criteria

### Phase 2A Success
- [ ] vtcode-commons has ui/constants.rs
- [ ] vtcode-commons has ui/types.rs
- [ ] vtcode-tool-traits has TaskPlan/PlanSummary
- [ ] vtcode-ui/src/utils.rs exists

### Phase 2B Success
- [ ] Prompts integration decided (trait or import)
- [ ] All data dependencies resolved
- [ ] No more crate::config:: imports

### Phase 2C Success
- [ ] No more crate::prompts:: imports
- [ ] No more crate::tools:: imports
- [ ] No more crate::utils:: imports

### Phase 2D Success
- [ ] All crate::ui:: changed to crate::
- [ ] vtcode-ui compiles independently
- [ ] Tests pass

### Phase 2E Success
- [ ] vtcode-core imports vtcode-ui
- [ ] vtcode-core/src/ui/ removed
- [ ] All tests pass
- [ ] Compilation time improved

---

## Quick Reference Commands

### Find all crate:: imports
```bash
find vtcode-ui/src -name "*.rs" -exec grep -H "^use crate::" {} \;
```

### Count dependencies by module
```bash
grep -r "^use crate::" vtcode-ui/src | cut -d: -f2 | sort | uniq -c | sort -rn
```

### List files needing updates
```bash
grep -rl "^use crate::config::" vtcode-ui/src
grep -rl "^use crate::prompts::" vtcode-ui/src
grep -rl "^use crate::tools::" vtcode-ui/src
grep -rl "^use crate::utils::" vtcode-ui/src
```

---

## Visual Dependency Heat Map

```
Files by Dependency Count:
src/tui/session.rs            ████████ 4 deps
src/tui/session/modal.rs      ██████ 3 deps
src/tui/modern_integration.rs ██████ 3 deps
src/tui/session/slash_palette.rs ██████ 3 deps
src/theme.rs                  ████ 2 deps
src/tui/types.rs              ████ 2 deps
src/tui/session/state.rs      ████ 2 deps
src/markdown.rs               ████ 2 deps
src/diff_renderer.rs          ████ 2 deps
[... 33 more files with 0-1 deps]
```

**Hotspots**: Focus refactoring effort on top 4 files first

---

## Next Actions (Prioritized)

### 🔥 Do Now (No Conflicts)
1. Create vtcode-commons/src/ui/ module structure
2. Copy constants to vtcode-commons/src/ui/constants.rs
3. Copy types to vtcode-commons/src/ui/types.rs
4. Document in this repo (won't conflict)

### ⏳ Do Soon (Coordinate First)
5. Discuss prompts approach with prompts team
6. Agree on TaskPlan location with all teams
7. Schedule integration sprint

### 📋 Do Later (After Foundation)
8. Update imports systematically
9. Integration testing
10. Performance benchmarking

---

**Document Version**: 1.0
**Companion To**: DEPENDENCY_ANALYSIS.md
**Last Updated**: November 13, 2025
