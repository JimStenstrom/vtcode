# 🏗️ Architectural Debt Analysis - Domain & DRY Violations

**Date:** 2025-11-15
**Focus:** Domain boundaries, Single Responsibility, DRY violations
**Severity:** **CRITICAL** - Major architectural refactoring needed

---

## 🚨 CRITICAL: vtcode-core is a MASSIVE Dumping Ground

### The Core Problem

**vtcode-core has become everything to everyone.** It violates every principle of good modular design.

#### Statistics:
- **247 Rust files** in vtcode-core alone
- **18 top-level modules** (should be <8)
- **Multiple competing domains** mixed together
- **31,557 total lines** across all files

#### What vtcode-core Claims to Do:

According to `lib.rs`, it's responsible for:
1. Provider abstraction (LLM)
2. Prompt caching
3. Semantic workspace model (tree-sitter)
4. Tool system
5. Configuration management
6. Safety & observability
7. Terminal integration (PTY)
8. Agent orchestration
9. UI utilities
10. File operations
11. Bash execution
12. ACP integration
13. MCP integration
14. Code analysis
15. And more...

### **This is INSANE. One crate cannot do all of this.**

---

## 💥 Domain Violations Breakdown

### 1. **"Core Within Core" Anti-Pattern**

**Location:** `vtcode-core/src/core/`

You have a module called `core` **inside** `vtcode-core`. This is a red flag that screams "we don't know where to put things anymore."

```
vtcode-core/
  src/
    core/          ← "Core" within "core" 🤦
      agent/
      prompt_caching.rs
      token_budget.rs
      error_recovery.rs
      context_pruner.rs
      ...
```

**What this tells me:** The original "core" became too bloated, so you created a "real core" inside it. This is architectural bankruptcy.

**Fix:** The `core/` subdirectory should be its own crate: `vtcode-agent-runtime` or similar.

---

### 2. **Utils is Not a Domain - It's a Code Smell**

**Location:** `vtcode-core/src/utils/` (6,457 lines across 18 files)

```
utils/
  ansi.rs                    (1,001 lines) - Terminal styling
  session_archive.rs         (705 lines)   - Session persistence
  dot_config.rs              (567 lines)   - User configuration
  ratatui_styles.rs          (481 lines)   - UI styling
  diff.rs                    (478 lines)   - Diff rendering
  vtcodegitignore.rs         (383 lines)   - Git integration
  transcript.rs              (342 lines)   - Transcript management
  color_utils.rs             (314 lines)   - Color manipulation
  ansi_parser.rs             (272 lines)   - ANSI parsing
  image_processing.rs        (263 lines)   - Image handling
  ...
```

### **STOP. UTILS IS NOT A DOMAIN.**

These belong in **separate crates**:
- `vtcode-terminal-styling` - ansi.rs, color_utils.rs, ansi_parser.rs, ratatui_styles.rs
- `vtcode-session` - session_archive.rs, transcript.rs
- `vtcode-config-user` - dot_config.rs (or merge with vtcode-config)
- `vtcode-git-integration` - vtcodegitignore.rs
- `vtcode-diff` - diff.rs, diff_styles.rs
- `vtcode-image` - image_processing.rs

**Current state:** 6,457 lines of unrelated code dumped in "utils" because you didn't know where else to put it.

---

### 3. **Duplication Across UI Styling**

**Files with overlapping responsibilities:**

```
vtcode-core/src/utils/ansi.rs              (1,001 lines) - ANSI codes & styling
vtcode-core/src/utils/anstyle_utils.rs     (198 lines)  - anstyle conversions
vtcode-core/src/utils/color_utils.rs       (314 lines)  - Color manipulation
vtcode-core/src/utils/colors.rs            (224 lines)  - Color definitions
vtcode-core/src/utils/ratatui_styles.rs    (481 lines)  - ratatui styling
vtcode-core/src/utils/style_helpers.rs     (173 lines)  - Style helpers
vtcode-core/src/utils/diff_styles.rs       (83 lines)   - Diff styling
vtcode-core/src/utils/cached_style_parser.rs (181 lines) - Style parsing cache
```

**Total: ~2,655 lines** doing variations of "terminal styling"

### **This is a DRY nightmare.**

You have **8 different files** all dealing with colors and styles in slightly different ways:
- ANSI codes
- anstyle conversion
- ratatui conversion
- Color utilities
- Style helpers
- Cached parsers

**Recommendation:**
Create **ONE** crate: `vtcode-terminal-style`
- Consolidate all styling logic
- Single source of truth for color/style conversions
- Proper abstraction over anstyle → ratatui → ANSI
- Estimated reduction: ~2,655 lines → ~800 lines (70% reduction)

---

### 4. **Tool System Domain Violation**

**Location:** `vtcode-core/src/tools/`

Your tool system is split across **multiple domains**:

```
vtcode-core/src/tools/
  file_ops.rs        (1,958 lines) - File operations
  pty.rs             (1,082 lines) - PTY/terminal
  grep_file.rs       (569 lines)   - File search
  command.rs         (548 lines)   - Command execution
  plan.rs            (427 lines)   - Task planning
  editing/           - File editing
  registry/          - Tool registry
  tree_sitter/       - Code analysis
  web_fetch/         - Web fetching
```

**Problems:**
1. **Mixing abstractions:** File ops, PTY, search, planning all in one module
2. **Large files:** file_ops.rs is 1,958 lines - another god object
3. **Unclear boundaries:** Is `tree_sitter` a tool or a code analysis engine?
4. **Planning logic:** Why is task planning in the tools module?

**Fix:**
```
vtcode-tools/          (Already exists - good!)
vtcode-tools-fs/       (File operations: file_ops, editing, grep_file)
vtcode-tools-shell/    (Shell: pty, command, bash integration)
vtcode-tools-net/      (Network: web_fetch)
vtcode-code-analysis/  (Tree-sitter, semantic analysis - NOT a "tool")
```

---

### 5. **Configuration Explosion**

**Location:** `vtcode-core/src/config/`

```
config/
  mod.rs
  core/             - Core config types
  defaults/         - Default values
  loader/           - Loading logic
  types/            - Type definitions
  acp.rs            - ACP config
  api_keys.rs       - API key management
  constants.rs      - Constants
  context.rs        - Context config
  hooks.rs          - Hook config
  mcp.rs            - MCP config
  models.rs         (2,168 lines!) - Model definitions
  router.rs         - Router config
  telemetry.rs      - Telemetry config
  validator.rs      - Validation logic
```

**You already have `vtcode-config` crate. Why is this still in core?**

**Problem:** Configuration is split between:
- `vtcode-config` crate
- `vtcode-core/src/config/` (massive)
- `vtcode-core/src/utils/dot_config.rs` (user prefs)

### **Pick ONE place for configuration.**

**Recommendation:**
Move everything to `vtcode-config`:
- Merge `vtcode-core/src/config/*` into `vtcode-config`
- Move `dot_config.rs` into `vtcode-config/user.rs`
- Single source of truth for ALL configuration

---

### 6. **Prompt/Instruction Duplication**

**Locations:**
- `vtcode-core/src/instructions.rs` (441 lines)
- `vtcode-core/src/prompts/` (separate module)
- `vtcode-prompts` (separate crate!)

### **You have THREE different places handling prompts!**

**Which one is canonical?** Nobody knows.

**Fix:**
- Delete `vtcode-core/src/instructions.rs`
- Delete `vtcode-core/src/prompts/` (if redundant)
- Use **ONLY** `vtcode-prompts` crate

---

### 7. **LLM Domain Leakage**

**The Good News:** You extracted providers to separate crates. Great!

**The Bad News:** `vtcode-core` still has massive LLM logic:

```
vtcode-core/src/llm/
  factory.rs         (437 lines)
  client.rs          (large)
  capabilities.rs
  provider.rs
  rig_adapter.rs
  token_metrics.rs
  ...
```

**Plus:**
- `vtcode-core/src/core/token_budget.rs` (810 lines)
- `vtcode-core/src/core/token_estimator.rs`
- `vtcode-core/src/core/prompt_caching.rs` (561 lines)

### **Token management is LLM domain, not "core" domain.**

**Recommendation:**
Create `vtcode-llm-orchestration` crate:
- Move `llm/factory.rs`, `llm/client.rs`
- Move token budget/estimator
- Move prompt caching
- Leave only interface traits in core

---

### 8. **UI Logic in Core**

**Location:** `vtcode-core/src/ui/`

**Why is UI rendering in the "core" library?**

You already have `vtcode-ui` crate!

**Files in vtcode-core/src/ui/:**
- `diff_renderer.rs`
- Other UI utilities (re-exported to vtcode-ui)

**Fix:** Move ALL UI to `vtcode-ui`. Core should not know about rendering.

---

### 9. **Bash Runner Duplication**

**Locations:**
- `vtcode-core/src/bash_runner.rs` (374 lines)
- `vtcode-bash-runner` (separate crate!)

### **Which one is used?**

This is confusing. Either:
1. Delete `vtcode-core/src/bash_runner.rs` and use the crate
2. Or delete the crate and keep it in core

**Don't have both.**

---

## 📊 Quantified DRY Violations

| Domain | Files | Total LoC | Should Be | Waste |
|--------|-------|-----------|-----------|-------|
| **Terminal Styling** | 8 files | ~2,655 | 1 crate (~800 LoC) | ~1,855 LoC |
| **Configuration** | 2 locations | ~3,500 | 1 crate | Confusion |
| **Prompts** | 3 locations | ~1,000 | 1 crate | Confusion |
| **Bash Execution** | 2 locations | ~500 | 1 crate | Duplication |
| **UI Rendering** | 2 locations | Unknown | 1 crate | Leakage |

**Estimated waste:** ~3,000+ lines of duplicated/misplaced code

---

## 🏛️ Proposed Architecture Refactoring

### Current State (Broken):
```
vtcode-core
  ├─ Everything under the sun (247 files, 18 modules)
  └─ God crate with no clear boundaries
```

### Proposed State (Fixed):

#### **Tier 1: Foundation**
```
vtcode-commons         ✅ (Already exists - good!)
vtcode-types           (Create - core types only)
```

#### **Tier 2: Configuration & Domain Models**
```
vtcode-config          ✅ (Consolidate ALL config here)
  ├─ Core config
  ├─ User preferences (from utils/dot_config)
  ├─ Model definitions (from config/models.rs)
  └─ Validation
```

#### **Tier 3: Domain-Specific Crates**
```
vtcode-terminal-style  (NEW - consolidate 8 styling files)
vtcode-session         (NEW - session management, transcript)
vtcode-git             (NEW - git integration)
vtcode-diff            (NEW - diff rendering)

vtcode-llm-types       ✅ (Already exists)
vtcode-llm-orchestration (NEW - factory, client, token mgmt)
vtcode-llm-*           ✅ (Providers already extracted)

vtcode-tools-core      (NEW - tool traits & registry)
vtcode-tools-fs        (NEW - file operations)
vtcode-tools-shell     (NEW - PTY, bash, command)
vtcode-tools-net       (NEW - web fetch)

vtcode-code-analysis   (NEW - tree-sitter, semantic parsing)

vtcode-agent-runtime   (NEW - agent loop, orchestration)
  ├─ From core/agent/
  ├─ From core/error_recovery.rs
  ├─ From core/timeout_detector.rs
  └─ From core/context_pruner.rs
```

#### **Tier 4: Integration & UI**
```
vtcode-ui              ✅ (Consolidate ALL UI here)
vtcode-mcp             ✅ (Already good)
vtcode-acp-client      ✅ (Already good)
```

#### **Tier 5: Application**
```
vtcode (binary)        ✅ (Main application)
vtcode-core            (Slim coordinator - <2,000 LoC total)
  ├─ Re-exports from domain crates
  ├─ High-level orchestration ONLY
  └─ No implementation details
```

---

## 🎯 Action Plan: Refactoring Priority

### Phase 1: Quick Wins (Low Risk, High Impact)
1. **Consolidate terminal styling** → Create `vtcode-terminal-style`
2. **Delete duplicate bash runner** → Pick one implementation
3. **Consolidate prompts** → Delete from core, use `vtcode-prompts` only
4. **Move UI logic** → Everything to `vtcode-ui`

**Impact:** ~3,000 lines removed, clear boundaries

---

### Phase 2: Configuration Consolidation (Medium Risk)
1. **Move `vtcode-core/src/config/*`** → `vtcode-config`
2. **Move `utils/dot_config.rs`** → `vtcode-config/user.rs`
3. **Update imports across codebase**

**Impact:** Single source of truth for config

---

### Phase 3: Tool System Refactoring (Medium-High Risk)
1. **Extract file operations** → `vtcode-tools-fs`
2. **Extract shell tools** → `vtcode-tools-shell`
3. **Extract tree-sitter** → `vtcode-code-analysis`
4. **Keep registry** → `vtcode-tools` (slim coordinator)

**Impact:** Clear tool boundaries, easier testing

---

### Phase 4: LLM Orchestration (Medium Risk)
1. **Create `vtcode-llm-orchestration`**
2. **Move token management** from core/
3. **Move prompt caching** from core/
4. **Move factory & client** from llm/

**Impact:** LLM concerns separated from agent runtime

---

### Phase 5: Agent Runtime Extraction (High Risk - Do Last)
1. **Create `vtcode-agent-runtime`**
2. **Move `core/agent/*`**
3. **Move core/* modules** (error recovery, timeout, pruning)
4. **Slim down vtcode-core** to <2,000 LoC coordinator

**Impact:** Clean separation of agent runtime from application

---

## 🔍 Code Smell Indicators

### How to Spot Domain Violations:

1. **"Utils" directory** → Not a domain. Break it up.
2. **"Core within core"** → Original abstraction failed.
3. **Duplicate implementations** → Pick one, delete others.
4. **3+ files doing similar things** → DRY violation.
5. **Crate > 10,000 LoC** → Too big, split it.
6. **Module with >10 submodules** → Too complex, split it.

### Current Violations:
- ✅ utils/ directory (6,457 lines)
- ✅ core/ within vtcode-core
- ✅ 8 styling files doing similar things
- ✅ 3 prompt locations
- ✅ vtcode-core: 31,557 lines (should be <5,000)
- ✅ vtcode-core: 18 top-level modules (should be <8)

---

## 📈 Success Metrics

### Before Refactoring:
- **vtcode-core:** 247 files, 31,557 LoC
- **Top-level modules:** 18
- **Utils dumping ground:** 6,457 LoC across 18 files
- **Configuration locations:** 3
- **Prompt locations:** 3
- **Clear domain boundaries:** ❌

### After Refactoring (Target):
- **vtcode-core:** <50 files, <5,000 LoC
- **Top-level modules:** <8 (mostly re-exports)
- **Utils directory:** DELETED
- **Configuration locations:** 1 (`vtcode-config`)
- **Prompt locations:** 1 (`vtcode-prompts`)
- **Clear domain boundaries:** ✅

### Estimated Impact:
- **Lines of code reduction:** ~3,000-5,000 (duplication elimination)
- **Cognitive load reduction:** Massive (clear module boundaries)
- **Build time improvement:** Faster (smaller dependency graphs)
- **Testing improvement:** Easier (isolated domains)
- **Onboarding improvement:** Faster (clear architecture)

---

## 🚦 Risk Assessment

### Low Risk Refactorings:
- Terminal styling consolidation
- Duplicate elimination (bash runner, prompts)
- UI logic movement

### Medium Risk Refactorings:
- Configuration consolidation
- Tool system split
- LLM orchestration extraction

### High Risk Refactorings:
- Agent runtime extraction (session.rs is 4,762 lines!)
- Core slimming

**Recommendation:** Start with low-risk, high-impact changes first.

---

## 💬 Conclusion

**Your codebase suffers from severe domain boundary violations.**

The root cause: **vtcode-core became the dumping ground for everything.**

When you didn't know where to put code, you put it in:
1. `vtcode-core/src/utils/` (the junk drawer)
2. `vtcode-core/src/core/` (core within core)
3. Another top-level module in core (now at 18!)

**This is technical debt at the architectural level.** It's more serious than code quality issues because it affects:
- Developer productivity (hard to find code)
- Onboarding (steep learning curve)
- Build times (massive dependency graph)
- Testing (tight coupling)
- Maintenance (unclear ownership)

**The fix requires discipline:**
1. **Stop adding to vtcode-core**
2. **Extract domains to focused crates**
3. **Delete duplicates**
4. **Enforce single responsibility**

**This is a 6-12 month refactoring** if done carefully. But it's **essential** for long-term health.

---

## 🎓 Architectural Principles to Follow

1. **Single Responsibility:** Each crate should have ONE clear purpose
2. **Dependency Direction:** Dependencies flow inward (domain → foundation)
3. **No God Crates:** If a crate has >15 top-level modules, split it
4. **Utils is a Smell:** "Utils" means "I don't know where this goes"
5. **DRY:** If 3+ files do similar things, consolidate
6. **Bounded Contexts:** Each crate is a bounded domain context
7. **Slim Coordinators:** Top-level crates should be thin orchestration layers

---

**Final Grade: C- (Architectural Debt)**

You've built a powerful system, but the architecture is collapsing under its own weight. **Refactor now before it becomes unmaintainable.**
