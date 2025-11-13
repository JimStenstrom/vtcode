# vtcode-ui Dependency Analysis

**Generated**: November 13, 2025
**Purpose**: Comprehensive analysis of vtcode-ui's dependencies on vtcode-core to guide Phase 2 integration
**Status**: ✅ Complete

---

## Executive Summary

**Total Files Analyzed**: 42 Rust source files (~19K LOC)
**External Dependencies Found**: 25 unique imports across 4 vtcode-core modules
**Integration Complexity**: Medium - Clean boundaries possible with proper abstraction

### Key Finding
vtcode-ui has relatively **clean, minimal dependencies** on vtcode-core compared to initial expectations. The dependencies fall into 4 clear categories, all of which can be resolved through established patterns.

---

## Detailed Dependency Breakdown

### 1. Configuration Dependencies (12 imports)

**Module**: `crate::config::*`

#### 1.1 Constants
```rust
// Used in: theme.rs, tui/session.rs, tui/types.rs, tui/tui.rs,
//          tui/session/input.rs, tui/session/header.rs,
//          tui/session/slash.rs, tui/session/modal.rs

use crate::config::constants::ui;           // 4 occurrences
use crate::config::constants::defaults;     // 1 occurrence
use crate::config::constants::prompts;      // 1 occurrence
```

**What they contain**:
- UI layout constants (padding, margins, sizes)
- Default theme ID
- Color contrast ratios
- Timing values
- Default prompt configurations

**Resolution Strategy**:
- ✅ **Move to `vtcode-commons`** - These are pure data constants
- Create `vtcode-commons/src/ui_constants.rs`
- Or enhance `vtcode-config` to export these

#### 1.2 Type Definitions
```rust
// Used in: tui.rs, tui/modern_tui.rs, tui/tui.rs, tui/modern_integration.rs

use crate::config::types::UiSurfacePreference;    // 3 occurrences
use crate::config::types::ReasoningEffortLevel;   // 1 occurrence
```

**What they contain**:
- `enum UiSurfacePreference { Terminal, ModernTui, Auto }`
- `enum ReasoningEffortLevel { Low, Medium, High }`

**Resolution Strategy**:
- ✅ **Move to `vtcode-commons`** or keep in `vtcode-config`
- These are pure data types with no behavior
- UI needs to know which mode to display

#### 1.3 Configuration Loaders
```rust
// Used in: markdown.rs

use crate::config::loader::SyntaxHighlightingConfig;  // 1 occurrence
```

**What it contains**:
- Configuration for syntax highlighting themes
- Theme loading logic

**Resolution Strategy**:
- ✅ **Pass as trait or concrete type**
- UI can receive this as a parameter
- Or vtcode-ui can re-export from vtcode-config

---

### 2. Prompts Dependencies (6 imports)

**Module**: `crate::prompts::*`

```rust
// Used in: tui/session.rs, tui/session/state.rs,
//          tui/session/prompt_palette.rs, tui/session/slash_palette.rs

use crate::prompts::CustomPrompt;              // 2 occurrences
use crate::prompts::CustomPromptRegistry;      // 4 occurrences
```

**What they contain**:
- `struct CustomPrompt` - Represents a custom prompt template
- `struct CustomPromptRegistry` - Registry of available prompts

**Data Structures**:
```rust
pub struct CustomPrompt {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    // ...
}

pub struct CustomPromptRegistry {
    prompts: HashMap<String, CustomPrompt>,
    // ...
}
```

**Resolution Strategy**:
- ✅ **Extract types to `vtcode-commons`**
- Keep registry logic in vtcode-core
- vtcode-ui only needs read-only access to display prompts
- Pass prompts as `&[CustomPrompt]` or similar

**Alternative**:
- Create trait `PromptProvider` in vtcode-ui
- vtcode-core implements it
- Inversion of control pattern

---

### 3. Tools Dependencies (5 imports)

**Module**: `crate::tools::*`

```rust
// Used in: tui/session.rs, tui/types.rs, tui/session/state.rs

use crate::tools::TaskPlan;      // 4 occurrences
use crate::tools::PlanSummary;   // 1 occurrence
```

**What they contain**:
- `struct TaskPlan` - Represents a plan with tasks
- `struct PlanSummary` - Summary of plan execution

**Data Structures** (estimated):
```rust
pub struct TaskPlan {
    pub id: String,
    pub tasks: Vec<Task>,
    pub status: PlanStatus,
    // ...
}

pub struct PlanSummary {
    pub total_tasks: usize,
    pub completed: usize,
    pub failed: usize,
    // ...
}
```

**Resolution Strategy**:
- ✅ **Extract to `vtcode-tool-traits`** (already exists!)
- These are data types that UI displays
- Phase 1 already created vtcode-tool-traits - use it
- Move TaskPlan/PlanSummary there

---

### 4. Utils Dependencies (4 imports)

**Module**: `crate::utils::*`

```rust
// Used in: theme_config.rs, tui/theme_parser.rs,
//          user_confirmation.rs, diff_renderer.rs

use crate::utils::CachedStyleParser;                     // 2 occurrences
use crate::utils::colors::style;                         // 1 occurrence
use crate::utils::style_helpers::style_from_color_name;  // 1 occurrence
```

**What they contain**:
- `CachedStyleParser` - Caches parsed ANSI styles
- `style()` - Helper to create styles
- `style_from_color_name()` - Parse color names

**Resolution Strategy**:
- ✅ **Move to `vtcode-ui` itself** - These are UI utilities
- Or move to `vtcode-commons/src/ui_utils.rs`
- Small utility functions, easy to relocate

---

## Internal Self-References (OK)

These are internal to vtcode-ui and don't need changes:
```rust
use crate::ui::FileColorizer;
use crate::ui::GitColorConfig;
use crate::ui::ThemeConfig;
use crate::ui::theme::{self, ThemeStyles};
use crate::ui::search::{fuzzy_match, normalize_query};
use crate::ui::slash::{SlashCommandInfo, suggestions_for};
use crate::ui::tui::types::{InlineCommand, InlineEvent, ...};
// ... many more
```

These will automatically work once we update `crate::ui::` to `crate::` (or `vtcode_ui::`).

---

## External Crate Dependencies (Already in Cargo.toml)

✅ All properly configured:
- **Terminal/TUI**: ratatui, crossterm, vt100, portable-pty
- **ANSI**: anstyle suite (anstyle, anstyle-parse, anstyle-crossterm, etc.)
- **Syntax**: syntect, pulldown-cmark
- **Theming**: catppuccin
- **Search**: perg, nucleo-matcher
- **Utilities**: unicode-width, unicode-segmentation, textwrap
- **Async**: tokio, futures

---

## Proposed Resolution Strategy

### Phase 2A: Move Pure Data Types (Week 1)

**To `vtcode-commons/src/ui/`**:
```rust
// vtcode-commons/src/ui/constants.rs
pub mod constants {
    pub mod ui { /* UI layout constants */ }
    pub mod defaults { /* Default values */ }
}

// vtcode-commons/src/ui/types.rs
pub enum UiSurfacePreference { Terminal, ModernTui, Auto }
pub enum ReasoningEffortLevel { Low, Medium, High }
```

**To `vtcode-tool-traits/src/types.rs`**:
```rust
// vtcode-tool-traits/src/types.rs
pub struct TaskPlan { /* ... */ }
pub struct PlanSummary { /* ... */ }
```

### Phase 2B: Extract Registry Interfaces (Week 2)

**Create traits in `vtcode-ui/src/interfaces.rs`**:
```rust
/// Trait for providing custom prompts to UI
pub trait PromptProvider {
    fn get_prompts(&self) -> Vec<PromptInfo>;
    fn get_prompt(&self, name: &str) -> Option<PromptInfo>;
}

pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    // ... minimal UI-needed fields
}
```

vtcode-core implements these traits.

### Phase 2C: Move Utility Functions (Week 2)

**To `vtcode-ui/src/utils.rs`**:
```rust
// Move these small utilities directly into vtcode-ui
pub struct CachedStyleParser { /* ... */ }
pub fn style(...) -> Style { /* ... */ }
pub fn style_from_color_name(...) -> Style { /* ... */ }
```

### Phase 2D: Update All Imports (Week 3)

**Search and replace**:
```bash
# Internal references
crate::ui::     → crate::
crate::ui::     → vtcode_ui::

# External references (from vtcode-core perspective)
crate::ui::     → vtcode_ui::
```

### Phase 2E: Integration (Week 4)

1. Update `vtcode-core/Cargo.toml`:
```toml
[dependencies]
vtcode-ui = { path = "../vtcode-ui", version = "0.43.6" }
```

2. Replace old imports in vtcode-core:
```rust
// Old
mod ui;
use crate::ui::*;

// New
use vtcode_ui::{ThemeManager, Session, ...};
```

3. Remove `vtcode-core/src/ui/` directory

---

## Coordination with Other Teams

### For vtcode-prompts Team
**Impact**: Medium
- `CustomPrompt` and `CustomPromptRegistry` types need to be:
  - Option A: Moved to vtcode-commons (if prompts team hasn't started)
  - Option B: Re-exported from vtcode-prompts (if they have the types)
  - Option C: UI defines minimal `PromptInfo` type

**Recommendation**: Wait for prompts team to structure their crate, then UI imports from there.

### For vtcode-mcp Team
**Impact**: None
- No direct dependencies on MCP found in vtcode-ui

### For vtcode-execution Team
**Impact**: Low
- UI displays TaskPlan/PlanSummary but doesn't execute
- These types should live in vtcode-tool-traits (Phase 1 crate)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Type conflicts when merging | Medium | Medium | Use semantic versioning, coordinate moves |
| Breaking API changes | Low | Medium | Types are simple data structures |
| Circular dependencies | Very Low | High | All moves are unidirectional (core → commons) |
| Merge conflicts | Low | Low | Different modules being worked on |

---

## Integration Checklist

### Before Integration
- [ ] Coordinate with prompts team on CustomPrompt types
- [ ] Move constants to vtcode-commons
- [ ] Move TaskPlan/PlanSummary to vtcode-tool-traits
- [ ] Move utility functions to vtcode-ui
- [ ] Update all imports in vtcode-ui

### During Integration
- [ ] Add vtcode-ui dependency to vtcode-core
- [ ] Update vtcode-core imports
- [ ] Remove vtcode-core/src/ui/ directory
- [ ] Run tests
- [ ] Fix compilation errors

### After Integration
- [ ] Verify all tests pass
- [ ] Benchmark compilation time (expect 10-15% improvement)
- [ ] Update documentation
- [ ] Create migration guide for downstream users

---

## Files Requiring Import Updates

### High Priority (Core UI)
1. `src/theme.rs` - 2 imports (config constants)
2. `src/tui/session.rs` - 4 imports (config, prompts, tools)
3. `src/tui/types.rs` - 2 imports (config, tools)
4. `src/tui/session/state.rs` - 2 imports (tools, prompts)

### Medium Priority (Features)
5. `src/markdown.rs` - 2 imports (config, theme)
6. `src/tui/modern_integration.rs` - 3 imports (session, types, config)
7. `src/tui/session/modal.rs` - 3 imports (config, search, types)
8. `src/tui/session/slash_palette.rs` - 3 imports (prompts, search, slash)

### Low Priority (Utilities)
9. `src/theme_config.rs` - 1 import (CachedStyleParser)
10. `src/diff_renderer.rs` - 2 imports (git_config, style_helpers)
11. `src/user_confirmation.rs` - 1 import (colors)

Total: 11 primary files + 31 files with internal references

---

## Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Total external dependencies | 25 imports | 0 imports |
| Modules depended on | 4 | 0 |
| Files requiring changes | 42 | 0 |
| Estimated effort (hours) | 40-60 | N/A |
| Estimated timeline (weeks) | 3-4 | 4 |
| Risk level | 🟡 Medium | 🟢 Low |

---

## Recommended Next Steps

### This Week (Nov 13-17)
1. ✅ Complete this analysis (DONE)
2. Share with other Phase 2 teams
3. Coordinate on shared types (prompts, tools)
4. Begin moving constants to vtcode-commons

### Next Week (Nov 18-24)
5. Move TaskPlan/PlanSummary to vtcode-tool-traits
6. Create trait interfaces for registries
7. Move utility functions into vtcode-ui

### Week 3 (Nov 25-Dec 1)
8. Update all imports systematically
9. Verify vtcode-ui compiles independently
10. Create integration PR

### Week 4 (Dec 2-8)
11. Integrate with vtcode-core
12. Run full test suite
13. Benchmark compilation improvements
14. Document changes

---

## Conclusion

vtcode-ui has **clean, well-defined dependencies** on vtcode-core. The 25 external imports fall into clear categories that can be resolved through established patterns:

1. **Data types** → Move to vtcode-commons or vtcode-tool-traits
2. **Registries** → Create trait interfaces
3. **Utilities** → Move into vtcode-ui
4. **Constants** → Extract to vtcode-commons

**Confidence Level**: 🟢 **HIGH** - Clear path forward with low risk

**Recommendation**: Proceed with Phase 2A (move pure data types) immediately. This work will not conflict with other teams and provides immediate value.

---

**Document Version**: 1.0
**Last Updated**: November 13, 2025
**Next Review**: After coordination with other Phase 2 teams
