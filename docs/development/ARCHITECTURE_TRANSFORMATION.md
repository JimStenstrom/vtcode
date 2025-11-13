# VTCode Architecture Transformation - Visual Guide

## Current Architecture (Before)

```
┌─────────────────────────────────────────────────────────────────┐
│                         vtcode (CLI)                             │
│                            1K LOC                                 │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                      vtcode-core                                 │
│                     MONOLITH: 95,000 LOC                         │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  UI (12K) │ LLM (15K) │ Tools (13K) │ MCP (3.5K)         │  │
│  │  Prompts (3K) │ Execution (4.5K) │ Config (5.5K)        │  │
│  │  Gemini (2.5K) │ Metrics (1.5K) │ Agent (8.5K)          │  │
│  │  Tree-sitter (2.7K) │ Utils (5K) │ Sandbox (0.8K)       │  │
│  └──────────────────────────────────────────────────────────┘  │
└──────────┬──────────────────┬──────────────────┬───────────────┘
           │                  │                  │
    ┌──────▼──────┐    ┌──────▼──────┐   ┌──────▼──────┐
    │ vtcode-llm  │    │vtcode-tools │   │vtcode-config│
    │  (circular!)│    │  (circular!)│   │   (partial) │
    └─────────────┘    └─────────────┘   └─────────────┘

Dependencies (good):
┌───────────────┐  ┌──────────────┐  ┌─────────────────┐
│vtcode-commons │  │vtcode-bash-  │  │vtcode-exec-     │
│   (traits)    │  │   runner     │  │    events       │
└───────────────┘  └──────────────┘  └─────────────────┘

┌──────────────┐  ┌──────────────┐  ┌─────────────────┐
│vtcode-indexer│  │vtcode-markdown│ │vtcode-acp-client│
│              │  │    -store     │  │                 │
└──────────────┘  └──────────────┘  └─────────────────┘
```

**Problems:**
- ❌ 95K LOC monolith
- ❌ Circular dependencies (llm, tools)
- ❌ 3-5 minute compile time
- ❌ Tight coupling
- ❌ Low reusability
- ❌ Hard to test independently

---

## Target Architecture (After)

```
┌─────────────────────────────────────────────────────────────────┐
│                         vtcode (CLI)                             │
│                           1K LOC                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                      vtcode-agent                                │
│              ORCHESTRATION CORE: 8-10K LOC                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Agent Loop │ Coordination │ Context │ State Management │  │
│  └──────────────────────────────────────────────────────────┘  │
└──┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┘
   │    │    │    │    │    │    │    │    │    │    │    │
   │    │    │    │    │    │    │    │    │    │    │    │
┌──▼──┐ │    │    │    │    │    │    │    │    │    │    │
│ UI  │ │    │    │    │    │    │    │    │    │    │    │
│12K  │ │    │    │    │    │    │    │    │    │    │    │
└─────┘ │    │    │    │    │    │    │    │    │    │    │
        │    │    │    │    │    │    │    │    │    │    │
     ┌──▼──┐ │    │    │    │    │    │    │    │    │    │
     │ LLM │ │    │    │    │    │    │    │    │    │    │
     │15K  │ │    │    │    │    │    │    │    │    │    │
     └──┬──┘ │    │    │    │    │    │    │    │    │    │
        │    │    │    │    │    │    │    │    │    │    │
   ┌────┴───┬┴───┬┴───┬┴───┐│    │    │    │    │    │    │
   │Anthropic│OpenAI│Gemini│OR││  │    │    │    │    │    │
   │  (opt) │(opt) │(opt) │  ││  │    │    │    │    │    │
   └────────┴────┴────┴────┘ │  │    │    │    │    │    │
                              │  │    │    │    │    │    │
                          ┌───▼┐ │    │    │    │    │    │
                          │Tools│    │    │    │    │    │
                          │ 10K│ │    │    │    │    │    │
                          └──┬─┘ │    │    │    │    │    │
                             │   │    │    │    │    │    │
                        ┌────┴──┬▼───┐│    │    │    │    │
                        │Tree-  │Patch││   │    │    │    │
                        │sitter │(opt)││   │    │    │    │
                        │(opt)  │     ││   │    │    │    │
                        └───────┴─────┘│   │    │    │    │
                                       │   │    │    │    │
                                  ┌────▼─┐ │    │    │    │
                                  │ MCP  │ │    │    │    │
                                  │ 3.5K │ │    │    │    │
                                  └──────┘ │    │    │    │
                                           │    │    │    │
                                      ┌────▼──┐ │    │    │
                                      │Prompts│ │    │    │
                                      │  3K   │ │    │    │
                                      └───────┘ │    │    │
                                                │    │    │
                                           ┌────▼──┐ │    │
                                           │Execute│ │    │
                                           │  3K   │ │    │
                                           └───────┘ │    │
                                                     │    │
                                              ┌──────▼─┐  │
                                              │Metrics │  │
                                              │ (opt)  │  │
                                              └────────┘  │
                                                          │
                                                    ┌─────▼┐
                                                    │ ANSI │
                                                    │(opt) │
                                                    └──────┘

Foundation Layer (used by all):
┌─────────────┐ ┌────────────┐ ┌───────────────┐ ┌─────────────┐
│vtcode-      │ │vtcode-llm- │ │vtcode-tool-   │ │vtcode-      │
│ commons     │ │   types    │ │   traits      │ │  config     │
└─────────────┘ └────────────┘ └───────────────┘ └─────────────┘

Execution Layer:
┌─────────────┐ ┌────────────┐ ┌───────────────┐
│vtcode-bash- │ │vtcode-exec-│ │vtcode-        │
│  runner     │ │  events    │ │ execution     │
└─────────────┘ └────────────┘ └───────────────┘

Storage Layer:
┌─────────────┐ ┌────────────┐
│vtcode-      │ │vtcode-     │
│ indexer     │ │ markdown-  │
│             │ │   store    │
└─────────────┘ └────────────┘

Integration Layer:
┌─────────────┐
│vtcode-acp-  │
│  client     │
└─────────────┘
```

**Benefits:**
- ✅ 8-10K LOC core (90% reduction)
- ✅ No circular dependencies
- ✅ 1-2 minute compile time (50-60% faster)
- ✅ Clear separation of concerns
- ✅ 10+ independently reusable crates
- ✅ Easy to test each component

---

## Transformation Phases

### Phase 1: Break Circular Dependencies (Foundation)
```
Before:                          After:
vtcode-core ←──→ vtcode-llm     vtcode-llm-types ← vtcode-core
vtcode-core ←──→ vtcode-tools   vtcode-tool-traits ← vtcode-core
vtcode-commons (basic)          vtcode-commons (enhanced with safety)

Result: Proper dependency hierarchy established
```

### Phase 2: Extract Large Subsystems
```
vtcode-core (95K LOC)
    │
    ├─→ vtcode-ui (12K) ────────────────┐
    ├─→ vtcode-prompts (3K) ────────────┤
    ├─→ vtcode-mcp (3.5K) ──────────────┤
    └─→ vtcode-execution (4.5K) ────────┤
                                         ▼
                              vtcode-core (72K LOC)

Result: 23K LOC extracted, 4 new reusable crates
```

### Phase 3: Modularize Providers
```
vtcode-core with embedded providers (72K LOC)
    │
    ├─→ vtcode-llm-anthropic (opt) ──────┐
    ├─→ vtcode-llm-openai (opt) ─────────┤
    ├─→ vtcode-llm-gemini (opt) ─────────┤
    ├─→ vtcode-llm-openrouter (opt) ─────┤
    └─→ vtcode-llm-local (opt) ──────────┤
                                          ▼
                              vtcode-core (60K LOC)
                              + vtcode-llm (15K)

Result: 12K LOC extracted, 6 optional provider crates
```

### Phase 4: Modularize Tools
```
vtcode-core with embedded tools (60K LOC)
    │
    ├─→ vtcode-tree-sitter (opt, 2.7K) ──┐
    ├─→ vtcode-patch (1.5K) ──────────────┤
    └─→ vtcode-tools (enhanced, 10K) ─────┤
                                           ▼
                              vtcode-core (45K LOC)

Result: 14K LOC extracted, 3 new tool crates
```

### Phase 5: Final Polish
```
vtcode-core (45K LOC)
    │
    ├─→ vtcode-metrics (opt, 1.5K) ──────┐
    ├─→ vtcode-ansi (opt, 1.5K) ─────────┤
    ├─→ vtcode-config (enhanced, +2K) ───┤
    └─→ Code cleanup & simplification ────┤
                                           ▼
                              vtcode-agent (8-10K LOC)

Result: Final architecture, 20+ independent crates
```

---

## Dependency Flow (After Refactoring)

### Layer 1: Foundation (No dependencies on other vtcode crates)
```
vtcode-commons
vtcode-llm-types
vtcode-tool-traits
vtcode-exec-events
```

### Layer 2: Utilities (Depend only on Layer 1)
```
vtcode-config ──→ vtcode-commons
vtcode-bash-runner ──→ vtcode-commons, vtcode-exec-events
vtcode-indexer ──→ (no vtcode deps)
vtcode-markdown-store ──→ (no vtcode deps)
vtcode-ansi ──→ (no vtcode deps)
```

### Layer 3: Domain Implementations (Depend on Layers 1-2)
```
vtcode-llm-anthropic ──→ vtcode-llm-types
vtcode-llm-openai ──→ vtcode-llm-types
vtcode-llm-gemini ──→ vtcode-llm-types
vtcode-tree-sitter ──→ vtcode-tool-traits
vtcode-patch ──→ vtcode-tool-traits
vtcode-execution ──→ vtcode-commons, vtcode-bash-runner
vtcode-safety ──→ vtcode-commons
```

### Layer 4: Subsystems (Depend on Layers 1-3)
```
vtcode-llm ──→ vtcode-llm-types, [provider crates]
vtcode-tools ──→ vtcode-tool-traits, vtcode-bash-runner, [tool crates]
vtcode-ui ──→ vtcode-commons, vtcode-ansi
vtcode-prompts ──→ vtcode-config, vtcode-commons
vtcode-mcp ──→ vtcode-tool-traits, vtcode-commons
vtcode-metrics ──→ vtcode-commons
```

### Layer 5: Integration (Depends on all layers)
```
vtcode-acp-client ──→ vtcode-tool-traits, vtcode-llm-types
vtcode-agent ──→ ALL subsystems from Layer 4
vtcode (CLI) ──→ vtcode-agent, vtcode-acp-client
```

**Key Properties:**
- ✅ Acyclic dependency graph
- ✅ Clear layering
- ✅ Each layer can be tested independently
- ✅ Optional features don't affect core layers

---

## Compilation Impact

### Before Refactoring
```
Clean Build Timeline:
├─ vtcode-commons ────── 10s
├─ vtcode-config ─────── 15s
├─ vtcode-core ───────── 180s ◄── BOTTLENECK
│  └─ Includes: UI, LLM providers, tools, MCP, everything
├─ vtcode (CLI) ───────── 5s
└─ Total: ~3-5 minutes

Change Impact:
- Any change to core → Rebuild everything
- Cannot parallelize compilation
- Large binary size (~80MB)
```

### After Refactoring
```
Clean Build Timeline (Parallel):
├─ Layer 1 (parallel) ─── 20s
│  ├─ vtcode-commons ──── 10s
│  ├─ vtcode-llm-types ── 5s
│  └─ vtcode-tool-traits─ 5s
├─ Layer 2 (parallel) ─── 30s
│  ├─ vtcode-config ───── 15s
│  ├─ vtcode-bash-runner─ 10s
│  └─ others ──────────── 20s
├─ Layer 3 (parallel) ─── 45s
│  ├─ vtcode-llm-* ────── 30s (6 crates)
│  ├─ vtcode-tools ────── 25s
│  └─ others ──────────── 20s
├─ Layer 4 (parallel) ─── 35s
│  ├─ vtcode-ui ──────── 20s
│  ├─ vtcode-prompts ─── 10s
│  └─ others ──────────── 15s
├─ Layer 5 ────────────── 20s
│  ├─ vtcode-agent ────── 15s ◄── MUCH SMALLER
│  └─ vtcode (CLI) ────── 5s
└─ Total: ~1-2 minutes (with parallelization)

Change Impact:
- Change to UI → Only rebuild ui + agent + cli
- Change to LLM provider → Only that provider + llm + agent + cli
- Parallel compilation of independent crates
- Smaller binary with optional features (~40MB base)
- Feature-gated builds (e.g., --no-default-features)
```

**Speedup**: 50-60% faster clean builds, 70-80% faster incremental builds

---

## Feature Flags

### Default Build (Minimal)
```toml
[dependencies]
vtcode-agent = { version = "0.43", default-features = false, features = [
    "anthropic",  # Core provider
    "openai",     # Core provider
] }
```

**Result**: ~40MB binary, 1 minute build

### Full Featured Build
```toml
[dependencies]
vtcode-agent = { version = "0.43", features = [
    "all-providers",  # All LLM providers
    "tree-sitter",    # Code analysis
    "metrics",        # Telemetry
    "mcp",            # MCP protocol
] }
```

**Result**: ~80MB binary, 2 minutes build

### Custom Build Examples
```toml
# Anthropic-only build for Claude users
features = ["anthropic", "ui"]

# Local-only build (no API keys needed)
features = ["local", "ui"]

# Headless agent (no UI, for API use)
features = ["anthropic", "openai", "no-ui"]

# Minimal CLI tool user
features = ["anthropic", "no-mcp", "no-metrics"]
```

---

## Testing Strategy

### Before Refactoring
```
Test Approach:
├─ Integration tests in vtcode-core
├─ Hard to isolate components
├─ Full core compilation required for any test
└─ ~5 minute test cycle

Test Coverage:
├─ Core: ~60% coverage
└─ Total: ~60% coverage
```

### After Refactoring
```
Test Approach:
├─ Unit tests per crate (isolated)
├─ Integration tests in vtcode-agent
├─ Can test components independently
└─ ~30 second test cycle per crate

Test Coverage Strategy:
├─ vtcode-commons: 95%+ (pure logic)
├─ vtcode-llm-types: 95%+ (data types)
├─ vtcode-llm-anthropic: 85%+ (provider logic)
├─ vtcode-ui: 80%+ (UI components)
├─ vtcode-tools: 90%+ (tool implementations)
├─ vtcode-agent: 70%+ (integration)
└─ Total: 85%+ coverage

Benefits:
✅ Faster test feedback
✅ Better test isolation
✅ Higher confidence per component
✅ Easier to maintain tests
```

---

## Reusability Matrix

### Current (Before)
| Component | Reusable? | Reason |
|-----------|-----------|--------|
| UI System | ❌ No | Locked in vtcode-core |
| LLM Providers | ❌ No | Tightly coupled to core |
| Tool System | ❌ No | Circular dependency |
| MCP Client | ❌ No | Embedded in core |
| Prompts | ❌ No | Uses core types |
| Tree-sitter | ❌ No | Part of core |
| Execution | ❌ No | Embedded in core |

### After Refactoring
| Component | Reusable? | Use Cases |
|-----------|-----------|-----------|
| vtcode-ui | ✅ Yes | Any Rust TUI app |
| vtcode-llm-anthropic | ✅ Yes | Any Anthropic integration |
| vtcode-llm-openai | ✅ Yes | Any OpenAI integration |
| vtcode-tools | ✅ Yes | Any tool-based agent |
| vtcode-mcp | ✅ Yes | Any MCP client in Rust |
| vtcode-prompts | ✅ Yes | Any LLM agent project |
| vtcode-tree-sitter | ✅ Yes | Code analysis tools |
| vtcode-execution | ✅ Yes | Sandboxed execution systems |
| vtcode-patch | ✅ Yes | Code modification tools |
| vtcode-ansi | ✅ Yes | Terminal applications |

**Potential Users**:
- Other AI agent frameworks
- Code analysis tools
- Terminal UI applications
- LLM integration libraries
- Sandbox execution systems

---

## Migration Path for Users

### For vtcode CLI Users
**Impact**: Minimal to none
- CLI command remain the same
- Configuration file format unchanged
- Behavior identical
- Optional: Faster compilation with feature flags

### For vtcode-core Library Users
**Impact**: Moderate (breaking changes)

**Old**:
```rust
use vtcode_core::{Agent, LLMClient, ToolRegistry};
```

**New**:
```rust
use vtcode_agent::{Agent};
use vtcode_llm::{LLMClient};
use vtcode_tools::{ToolRegistry};
```

**Migration Steps**:
1. Update imports to use specific crates
2. Add crate dependencies explicitly
3. Enable needed feature flags
4. Update to new trait interfaces

**Compatibility Layer** (optional):
```rust
// Re-exports for backward compatibility (deprecated)
pub use vtcode_agent::Agent;
pub use vtcode_llm::LLMClient;
pub use vtcode_tools::ToolRegistry;
```

### For Extension Developers
**Impact**: Positive (easier to extend)

**Before**:
```rust
// Hard to add custom provider
impl CustomProvider {
    // Must understand core internals
}
```

**After**:
```rust
// Clean trait implementation
use vtcode_llm_types::{AnyClient, LLMRequest, LLMResponse};

impl AnyClient for CustomProvider {
    async fn send(&self, req: LLMRequest) -> Result<LLMResponse> {
        // Simple interface
    }
}
```

---

## Success Criteria Summary

### Phase 1 Success
- ✅ No circular dependencies
- ✅ All crates compile independently
- ✅ Foundation traits established

### Phase 2 Success
- ✅ vtcode-core reduced by 23K LOC
- ✅ 4 major subsystems extracted
- ✅ Each subsystem independently useful

### Phase 3 Success
- ✅ LLM providers are optional features
- ✅ Compilation time reduced 30-40%
- ✅ Provider crates can be used standalone

### Phase 4 Success
- ✅ Tool system fully extracted
- ✅ Optional code analysis features
- ✅ Plugin system functional

### Phase 5 Success
- ✅ vtcode-core → vtcode-agent (8-10K LOC)
- ✅ 20+ independent crates
- ✅ 50-60% faster compilation
- ✅ Architecture documented
- ✅ Migration guide complete

---

**Final Result**: World-class, modular, domain-driven architecture ready for enterprise adoption and open-source collaboration.
