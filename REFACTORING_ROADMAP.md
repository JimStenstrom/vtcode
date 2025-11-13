# VTCode Enterprise Refactoring Roadmap

**Goal**: Transform vtcode-core from a 95K LOC monolith into a world-class, modular, domain-specific architecture following enterprise DRY principles.

**Strategy**: Extract cohesive subsystems into independent crates, eliminate circular dependencies, and establish clear architectural layers.

---

## Current State Analysis

### Problems
1. **Monolithic Core**: vtcode-core = 95,000 LOC with 60+ dependencies
2. **Circular Dependencies**: vtcode-llm and vtcode-tools depend on vtcode-core (should be reversed)
3. **Slow Compilation**: Large core means full rebuilds on any change
4. **Coupling**: UI, LLM, tools, config all tightly coupled in core
5. **Reusability**: Valuable subsystems (UI, prompts, MCP client) locked in core

### Strengths to Preserve
- ✅ Trait-based abstractions (Tool, AnyClient, McpToolExecutor)
- ✅ Event-driven execution (vtcode-exec-events)
- ✅ 6 properly extracted small crates (commons, config, bash-runner, etc.)
- ✅ Clear domain boundaries (LLM, tools, UI, execution)

---

## Target Architecture

```
vtcode (CLI - 1K LOC)
└── vtcode-agent (orchestration core - 8-10K LOC)
    ├── vtcode-llm (provider abstraction - 15K LOC)
    │   ├── vtcode-llm-types (shared types)
    │   ├── vtcode-llm-anthropic (optional feature)
    │   ├── vtcode-llm-openai (optional feature)
    │   ├── vtcode-llm-gemini (optional feature)
    │   ├── vtcode-llm-openrouter (optional feature)
    │   └── vtcode-llm-local (ollama, lmstudio - optional)
    ├── vtcode-tools (tool registry - 10K LOC)
    │   ├── vtcode-tool-traits (core abstractions)
    │   ├── vtcode-tree-sitter (optional code analysis - 2.7K)
    │   └── vtcode-patch (code modification - 1.5K)
    ├── vtcode-ui (TUI framework - 12K LOC)
    │   ├── vtcode-ui-components (reusable widgets)
    │   └── vtcode-ui-themes (theme system)
    ├── vtcode-mcp (MCP client - 3.5K LOC)
    ├── vtcode-prompts (prompt generation - 3K LOC)
    ├── vtcode-execution (sandboxed execution - 3K LOC)
    ├── vtcode-safety (validation utilities - merged from commons)
    ├── vtcode-metrics (telemetry - optional - 1.5K)
    ├── vtcode-config (enhanced)
    ├── vtcode-exec-events (exists)
    ├── vtcode-bash-runner (exists)
    ├── vtcode-indexer (exists)
    ├── vtcode-markdown-store (exists)
    └── vtcode-commons (base traits)
```

**Result**:
- Core reduced from 95K → 8-10K LOC (90% reduction)
- 10+ reusable, independently testable crates
- Optional features reduce default compilation
- Clear dependency hierarchy (no cycles)

---

## Execution Plan: 5 Phases with Parallel Branches

### 📋 Prerequisites
- All phases build on `dedupe/main` branch
- Each phase has multiple independent branches
- Branches within a phase can be executed in parallel
- Integration branch merges phase branches before moving to next phase

---

## Phase 1: Foundation - Break Circular Dependencies (CRITICAL)

**Duration**: 1-2 weeks
**Goal**: Establish proper dependency direction
**Parallel Execution**: 3 branches

### Branch 1.1: `refactor/extract-llm-types`
**Extract LLM types from vtcode-core into vtcode-llm-types**

**Tasks**:
1. Create `vtcode-llm-types/` crate
2. Move from `vtcode-core/src/llm/`:
   - `types.rs` → Core request/response types
   - `error.rs` → LLMError, LLMErrorKind
   - `token_metrics.rs` → Token counting types
   - `streaming.rs` → Streaming types
3. Update `vtcode-core` to depend on `vtcode-llm-types`
4. Update `vtcode-llm` to depend on `vtcode-llm-types`

**Success Criteria**:
- ✅ vtcode-llm-types compiles independently
- ✅ No circular dependencies
- ✅ All tests pass

**Files**: ~15 files, ~2K LOC

---

### Branch 1.2: `refactor/extract-tool-traits`
**Extract tool abstractions from vtcode-core into vtcode-tool-traits**

**Tasks**:
1. Create `vtcode-tool-traits/` crate
2. Move from `vtcode-core/src/tools/`:
   - `registry/traits.rs` → Tool, ToolExecutor traits
   - `types.rs` → ToolRequest, ToolResponse, ToolMetadata
   - `error.rs` → ToolError types
3. Move from `vtcode-core/src/`:
   - `tool_policy.rs` → ToolPolicy trait
4. Update `vtcode-core` to depend on `vtcode-tool-traits`
5. Update `vtcode-tools` to depend on `vtcode-tool-traits`

**Success Criteria**:
- ✅ vtcode-tool-traits compiles independently
- ✅ No circular dependencies
- ✅ All tests pass

**Files**: ~10 files, ~1.5K LOC

---

### Branch 1.3: `refactor/enhance-commons`
**Consolidate safety utilities into vtcode-commons**

**Tasks**:
1. Move from `vtcode-core/src/utils/`:
   - `safety.rs` → `vtcode-commons/src/safety.rs`
   - `vtcodegitignore.rs` → `vtcode-commons/src/gitignore.rs`
2. Add safety traits to vtcode-commons:
   - `PathValidator` trait
   - `WorkspaceBoundary` trait
3. Update all crates to use vtcode-commons for safety

**Success Criteria**:
- ✅ vtcode-commons has no vtcode-core dependency
- ✅ All safety logic centralized
- ✅ All tests pass

**Files**: ~5 files, ~800 LOC

---

### Phase 1 Integration: `refactor/phase1-foundation`
Merge branches 1.1, 1.2, 1.3 and verify:
- No circular dependencies
- All crates compile independently
- Full test suite passes
- Documentation updated

**Dependencies Established**:
```
vtcode-llm-types ← vtcode-core ← vtcode-llm (FIXED!)
vtcode-tool-traits ← vtcode-core ← vtcode-tools (FIXED!)
vtcode-commons (enhanced) ← all crates
```

---

## Phase 2: Large Subsystem Extraction (HIGH VALUE)

**Duration**: 2-3 weeks
**Goal**: Extract major independent subsystems
**Parallel Execution**: 4 branches
**Dependencies**: Requires Phase 1 complete

### Branch 2.1: `refactor/extract-ui`
**Extract TUI system into vtcode-ui crate**

**Tasks**:
1. Create `vtcode-ui/` crate
2. Move from `vtcode-core/src/ui/`:
   - `tui/session.rs` (4.7K LOC) → `vtcode-ui/src/session.rs`
   - `tui/session/modal.rs` → `vtcode-ui/src/modals/`
   - `tui/session/file_palette.rs` → `vtcode-ui/src/palettes/`
   - `markdown.rs` → `vtcode-ui/src/renderers/markdown.rs`
   - `diff_renderer.rs` → `vtcode-ui/src/renderers/diff.rs`
   - `theme_manager.rs` → `vtcode-ui/src/themes/`
3. Create event-driven interface:
   - `UIEvent` enum for user input
   - `UIState` for render state
   - `UICommand` for control flow
4. Update vtcode-core to use vtcode-ui via events

**Success Criteria**:
- ✅ vtcode-ui compiles independently
- ✅ No agent logic in UI (pure presentation)
- ✅ Event-driven architecture
- ✅ All UI tests pass
- ✅ Can be used by other projects

**Impact**:
- Remove 12K LOC from core
- Independent TUI framework
- Reusable for other Rust CLI apps

**Files**: ~45 files, ~12K LOC

---

### Branch 2.2: `refactor/extract-prompts`
**Extract prompt generation into vtcode-prompts crate**

**Tasks**:
1. Create `vtcode-prompts/` crate
2. Move from `vtcode-core/src/prompts/`:
   - `system.rs` → `vtcode-prompts/src/system.rs`
   - `custom.rs` → `vtcode-prompts/src/custom.rs`
   - `templates.rs` → `vtcode-prompts/src/templates/`
   - `context.rs` → `vtcode-prompts/src/context.rs`
   - `generator.rs` → `vtcode-prompts/src/generator.rs`
3. Create trait-based interface:
   - `PromptGenerator` trait
   - `TemplateRegistry` for custom prompts
   - `ContextBuilder` for dynamic injection
4. Update vtcode-core to use vtcode-prompts

**Success Criteria**:
- ✅ vtcode-prompts compiles independently
- ✅ Trait-based extensibility
- ✅ Template system documented
- ✅ All prompt tests pass

**Impact**:
- Remove 3K LOC from core
- Reusable prompt engineering patterns
- Shareable with other AI agent projects

**Files**: ~12 files, ~3K LOC

---

### Branch 2.3: `refactor/extract-mcp`
**Extract MCP client into vtcode-mcp crate**

**Tasks**:
1. Create `vtcode-mcp/` crate
2. Move from `vtcode-core/src/mcp/`:
   - `mod.rs` (2.4K LOC) → `vtcode-mcp/src/client.rs`
   - `tool_discovery.rs` → `vtcode-mcp/src/discovery.rs`
   - `enhanced_config.rs` → `vtcode-mcp/src/config.rs`
3. Create clean interface:
   - `McpClient` trait
   - `McpProvider` for server management
   - `McpToolExecutor` implementation
4. Update vtcode-core to use vtcode-mcp

**Success Criteria**:
- ✅ vtcode-mcp compiles independently
- ✅ No core dependencies beyond commons
- ✅ Can connect to any MCP server
- ✅ All MCP tests pass

**Impact**:
- Remove 3.5K LOC from core
- Reusable Rust MCP client
- Could be published as standalone crate

**Files**: ~15 files, ~3.5K LOC

---

### Branch 2.4: `refactor/extract-execution`
**Extract execution system into vtcode-execution crate**

**Tasks**:
1. Create `vtcode-execution/` crate
2. Move from `vtcode-core/src/`:
   - `exec/code_executor.rs` → `vtcode-execution/src/executor.rs`
   - `execpolicy/mod.rs` → `vtcode-execution/src/policy.rs`
   - `tool_policy.rs` → `vtcode-execution/src/tool_policy.rs`
   - `sandbox/` → `vtcode-execution/src/sandbox/`
3. Create security-focused interface:
   - `ExecutionPolicy` trait
   - `SandboxProfile` for Anthropic SRT
   - `ToolPolicy` for command filtering
4. Update vtcode-core to use vtcode-execution

**Success Criteria**:
- ✅ vtcode-execution compiles independently
- ✅ Security audit passes
- ✅ Sandbox integration works
- ✅ All policy tests pass

**Impact**:
- Remove 4.5K LOC from core
- Security-focused execution layer
- Reusable for other agent systems

**Files**: ~18 files, ~4.5K LOC

---

### Phase 2 Integration: `refactor/phase2-subsystems`
Merge branches 2.1, 2.2, 2.3, 2.4 and verify:
- vtcode-core reduced by ~23K LOC (95K → 72K)
- All extracted crates compile independently
- Full integration tests pass
- Documentation updated

---

## Phase 3: Provider Modularization (COMPILE SPEED)

**Duration**: 2-3 weeks
**Goal**: Make LLM providers optional features
**Parallel Execution**: 6 branches
**Dependencies**: Requires Phase 1 complete

### Branch 3.1: `refactor/extract-llm-anthropic`
**Extract Anthropic provider into vtcode-llm-anthropic**

**Tasks**:
1. Create `vtcode-llm-anthropic/` crate
2. Move from `vtcode-core/src/llm/providers/`:
   - `anthropic.rs` → `vtcode-llm-anthropic/src/client.rs`
   - `shared/` (anthropic-specific) → `vtcode-llm-anthropic/src/utils/`
3. Implement `AnyClient` trait from vtcode-llm-types
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-llm-anthropic compiles independently
- ✅ Can be used without vtcode-core
- ✅ All Anthropic tests pass

**Files**: ~8 files, ~1.5K LOC

---

### Branch 3.2: `refactor/extract-llm-openai`
**Extract OpenAI provider into vtcode-llm-openai**

**Tasks**:
1. Create `vtcode-llm-openai/` crate
2. Move from `vtcode-core/src/llm/providers/`:
   - `openai.rs` (2.6K LOC) → `vtcode-llm-openai/src/client.rs`
   - Include xAI, LMStudio, Ollama wrappers
3. Implement `AnyClient` trait
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-llm-openai compiles independently
- ✅ Supports all OpenAI-compatible providers
- ✅ All OpenAI tests pass

**Files**: ~12 files, ~3.5K LOC

---

### Branch 3.3: `refactor/extract-llm-gemini`
**Extract Gemini provider into vtcode-llm-gemini**

**Tasks**:
1. Create `vtcode-llm-gemini/` crate
2. Move from `vtcode-core/src/`:
   - `gemini/` entire module (2.5K LOC) → `vtcode-llm-gemini/src/`
   - `llm/providers/gemini.rs` → `vtcode-llm-gemini/src/provider.rs`
3. Implement `AnyClient` trait
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-llm-gemini compiles independently
- ✅ Gemini-specific features preserved
- ✅ All Gemini tests pass

**Files**: ~15 files, ~3.7K LOC

---

### Branch 3.4: `refactor/extract-llm-openrouter`
**Extract OpenRouter provider into vtcode-llm-openrouter**

**Tasks**:
1. Create `vtcode-llm-openrouter/` crate
2. Move from `vtcode-core/src/llm/providers/`:
   - `openrouter.rs` (2.2K LOC) → `vtcode-llm-openrouter/src/client.rs`
3. Implement `AnyClient` trait
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-llm-openrouter compiles independently
- ✅ All OpenRouter tests pass

**Files**: ~5 files, ~2.2K LOC

---

### Branch 3.5: `refactor/extract-llm-local`
**Extract local providers (Ollama, LMStudio) into vtcode-llm-local**

**Tasks**:
1. Create `vtcode-llm-local/` crate
2. Move from `vtcode-core/src/llm/providers/`:
   - `ollama.rs` → `vtcode-llm-local/src/ollama.rs`
   - `lmstudio.rs` → `vtcode-llm-local/src/lmstudio.rs`
3. Implement `AnyClient` trait for both
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-llm-local compiles independently
- ✅ Both providers work
- ✅ All local provider tests pass

**Files**: ~6 files, ~1.5K LOC

---

### Branch 3.6: `refactor/consolidate-llm-core`
**Create vtcode-llm core with provider registry**

**Tasks**:
1. Enhance `vtcode-llm/` crate:
   - Add `ProviderRegistry` for dynamic provider loading
   - Add `ProviderFactory` trait
   - Add provider discovery mechanism
2. Move from `vtcode-core/src/llm/`:
   - `client.rs` → `vtcode-llm/src/client.rs`
   - `provider.rs` → `vtcode-llm/src/provider.rs`
   - `error_display.rs` → `vtcode-llm/src/errors.rs`
3. Update to use optional provider crates

**Success Criteria**:
- ✅ vtcode-llm compiles with 0 providers
- ✅ Providers loaded via features
- ✅ Dynamic provider registration works

**Files**: ~20 files, ~8K LOC

---

### Phase 3 Integration: `refactor/phase3-providers`
Merge branches 3.1-3.6 and verify:
- vtcode-core no longer contains provider implementations
- Default build only includes Anthropic + OpenAI
- Feature flags work: `--features gemini,openrouter`
- All provider tests pass
- Compile time reduced by ~30-40%

**Feature Flags**:
```toml
[features]
default = ["anthropic", "openai"]
anthropic = ["vtcode-llm-anthropic"]
openai = ["vtcode-llm-openai"]
gemini = ["vtcode-llm-gemini"]
openrouter = ["vtcode-llm-openrouter"]
local = ["vtcode-llm-local"]
all-providers = ["anthropic", "openai", "gemini", "openrouter", "local"]
```

---

## Phase 4: Tool System Modularization (REUSABILITY)

**Duration**: 2 weeks
**Goal**: Extract specialized tool subsystems
**Parallel Execution**: 4 branches
**Dependencies**: Requires Phase 1 complete

### Branch 4.1: `refactor/extract-tree-sitter`
**Extract tree-sitter analysis into vtcode-tree-sitter crate**

**Tasks**:
1. Create `vtcode-tree-sitter/` crate
2. Move from `vtcode-core/src/tools/tree_sitter/`:
   - `analyzer.rs` → `vtcode-tree-sitter/src/analyzer.rs`
   - `languages.rs` → `vtcode-tree-sitter/src/languages/`
   - `analysis.rs` → `vtcode-tree-sitter/src/analysis.rs`
3. Create language-specific modules:
   - `rust.rs`, `python.rs`, `typescript.rs`, `go.rs`, `java.rs`
4. Make optional feature in vtcode-core

**Success Criteria**:
- ✅ vtcode-tree-sitter compiles independently
- ✅ Can be used by other code analysis tools
- ✅ All tree-sitter tests pass

**Impact**:
- Remove 2.7K LOC from core
- Reusable code analysis library
- Optional dependency (30% of users don't need it)

**Files**: ~15 files, ~2.7K LOC

---

### Branch 4.2: `refactor/extract-patch-system`
**Extract patch/diff system into vtcode-patch crate**

**Tasks**:
1. Create `vtcode-patch/` crate
2. Move from `vtcode-core/src/tools/editing/patch/`:
   - All patch application logic → `vtcode-patch/src/`
3. Create clean interface:
   - `PatchApplicator` trait
   - `DiffGenerator` for creating patches
   - `ConflictResolver` for merge conflicts

**Success Criteria**:
- ✅ vtcode-patch compiles independently
- ✅ Works with standard diff formats
- ✅ All patch tests pass

**Impact**:
- Remove 1.5K LOC from core
- Reusable for any code modification tools

**Files**: ~8 files, ~1.5K LOC

---

### Branch 4.3: `refactor/consolidate-tools`
**Consolidate tool implementations in vtcode-tools**

**Tasks**:
1. Enhance `vtcode-tools/` crate
2. Move from `vtcode-core/src/tools/`:
   - `file_ops.rs` (1.9K LOC) → `vtcode-tools/src/file_ops.rs`
   - `grep_file.rs` → `vtcode-tools/src/grep.rs`
   - `pty.rs` → `vtcode-tools/src/pty.rs`
   - `web_fetch/` → `vtcode-tools/src/web_fetch/`
3. Create tool categories:
   - `file/` - File operations
   - `search/` - Search and grep
   - `exec/` - Command execution
   - `web/` - HTTP tools

**Success Criteria**:
- ✅ vtcode-tools contains all tool implementations
- ✅ Depends on vtcode-tool-traits (Phase 1)
- ✅ No dependency on vtcode-core
- ✅ All tool tests pass

**Impact**:
- Remove 10K LOC from core
- Complete tool system extraction

**Files**: ~30 files, ~10K LOC

---

### Branch 4.4: `refactor/enhance-tool-registry`
**Create unified tool registry in vtcode-core**

**Tasks**:
1. Simplify `vtcode-core/src/tools/registry/`:
   - Remove implementations (moved to vtcode-tools)
   - Keep only registration and dispatch logic
   - Add plugin system for external tools
2. Create `ToolProvider` trait for dynamic loading
3. Add MCP tool integration via vtcode-mcp

**Success Criteria**:
- ✅ Registry is thin orchestration layer
- ✅ Tools loaded from vtcode-tools crate
- ✅ MCP tools work seamlessly
- ✅ Plugin system functional

**Files**: ~8 files, ~1K LOC (reduced from 3K)

---

### Phase 4 Integration: `refactor/phase4-tools`
Merge branches 4.1-4.4 and verify:
- Tool implementations extracted from core
- vtcode-tools is complete, independent crate
- Optional features work (tree-sitter, patch)
- Full tool integration tests pass

---

## Phase 5: Final Consolidation (POLISH)

**Duration**: 1-2 weeks
**Goal**: Polish architecture, consolidate utilities
**Parallel Execution**: 5 branches
**Dependencies**: All previous phases complete

### Branch 5.1: `refactor/extract-metrics`
**Extract metrics/telemetry into vtcode-metrics crate**

**Tasks**:
1. Create `vtcode-metrics/` crate
2. Move from `vtcode-core/src/metrics/`:
   - All metrics modules → `vtcode-metrics/src/`
3. Make optional feature
4. Add exporters (JSON, Prometheus)

**Success Criteria**:
- ✅ vtcode-metrics compiles independently
- ✅ Optional dependency
- ✅ All metric tests pass

**Files**: ~10 files, ~1.5K LOC

---

### Branch 5.2: `refactor/extract-ansi`
**Extract ANSI utilities into vtcode-ansi crate**

**Tasks**:
1. Create `vtcode-ansi/` crate
2. Move from `vtcode-core/src/utils/`:
   - `ansi.rs` (1K LOC) → `vtcode-ansi/src/ansi.rs`
   - `ansi_parser.rs` → `vtcode-ansi/src/parser.rs`
   - `anstyle_utils.rs` → `vtcode-ansi/src/style.rs`
3. Create clean interface for terminal styling

**Success Criteria**:
- ✅ vtcode-ansi compiles independently
- ✅ Can be used by other terminal apps

**Files**: ~6 files, ~1.5K LOC

---

### Branch 5.3: `refactor/simplify-agent-core`
**Simplify vtcode-agent (formerly vtcode-core)**

**Tasks**:
1. Rename `vtcode-core` → `vtcode-agent`
2. Remove all extracted code
3. Keep only:
   - Agent orchestration loop (`core/agent/`)
   - Configuration integration
   - Tool/LLM/UI coordination
4. Create clean `Agent` API

**Success Criteria**:
- ✅ vtcode-agent < 10K LOC (down from 95K)
- ✅ All dependencies external crates
- ✅ Clean separation of concerns

**Target LOC**: 8-10K LOC

---

### Branch 5.4: `refactor/enhance-config`
**Enhance vtcode-config with full extraction**

**Tasks**:
1. Move remaining config types from vtcode-agent
2. Add config validation layer
3. Add config migration system for breaking changes
4. Complete vtcode.toml schema documentation

**Success Criteria**:
- ✅ vtcode-config is complete
- ✅ No config types in vtcode-agent
- ✅ Schema documented

**Files**: ~15 files, ~3K LOC

---

### Branch 5.5: `refactor/update-workspace`
**Update workspace configuration and CI**

**Tasks**:
1. Update `Cargo.toml` workspace members
2. Add feature flags for optional crates
3. Update CI/CD for new crate structure
4. Update documentation
5. Create crate dependency graph diagram

**Success Criteria**:
- ✅ All crates in workspace
- ✅ CI passes for all crates
- ✅ Documentation complete

---

### Phase 5 Integration: `refactor/phase5-final`
Merge all Phase 5 branches and verify:
- Complete architecture transformation
- All 20+ crates compile independently
- Full integration tests pass
- Documentation complete
- Performance benchmarks show improvement

---

## Success Metrics

### Before Refactoring
- **vtcode-core**: 95,000 LOC
- **Crates**: 10 total (6 properly extracted + 4 with circular deps)
- **Compile time**: ~3-5 minutes (clean build)
- **Reusability**: Low (everything locked in core)
- **Maintainability**: Medium (monolithic)

### After Refactoring
- **vtcode-agent**: 8-10,000 LOC (90% reduction)
- **Crates**: 20+ total (all properly extracted)
- **Compile time**: ~1-2 minutes (clean build, 50-60% faster)
- **Reusability**: High (10+ independently useful crates)
- **Maintainability**: Excellent (clear boundaries)

### Additional Benefits
- ✅ Optional providers reduce default dependencies
- ✅ Tree-sitter optional (30% smaller for basic use)
- ✅ Metrics optional (telemetry-free builds)
- ✅ Each crate independently testable
- ✅ Parallel compilation of crates
- ✅ Incremental compilation benefits

---

## Execution Strategy

### Parallel Branch Management

Each phase has multiple branches that can be worked on simultaneously:

**Phase 1**: 3 parallel branches
- Developer 1: llm-types extraction
- Developer 2: tool-traits extraction
- Developer 3: commons enhancement
- Integration: Merge all 3 → phase1-foundation

**Phase 2**: 4 parallel branches
- Developer 1: UI extraction
- Developer 2: Prompts extraction
- Developer 3: MCP extraction
- Developer 4: Execution extraction
- Integration: Merge all 4 → phase2-subsystems

**Phase 3**: 6 parallel branches
- Developer 1: Anthropic provider
- Developer 2: OpenAI provider
- Developer 3: Gemini provider
- Developer 4: OpenRouter provider
- Developer 5: Local providers
- Developer 6: LLM core consolidation
- Integration: Merge all 6 → phase3-providers

**Phase 4**: 4 parallel branches
- Developer 1: Tree-sitter extraction
- Developer 2: Patch system
- Developer 3: Tools consolidation
- Developer 4: Registry enhancement
- Integration: Merge all 4 → phase4-tools

**Phase 5**: 5 parallel branches
- Developer 1: Metrics extraction
- Developer 2: ANSI extraction
- Developer 3: Agent core simplification
- Developer 4: Config enhancement
- Developer 5: Workspace updates
- Integration: Merge all 5 → phase5-final

### Branch Naming Convention
```
refactor/<phase>-<component>-<session-id>

Examples:
- refactor/extract-llm-types-011CVxxxx
- refactor/extract-ui-011CVxxxx
- refactor/extract-tree-sitter-011CVxxxx
```

### Integration Process
1. Create phase integration branch
2. Merge all phase branches
3. Run full test suite
4. Fix integration issues
5. Update documentation
6. Merge to main dedupe branch
7. Begin next phase

---

## Risk Mitigation

### High Risk Items
1. **Phase 1 circular dependency breaking**: Must be correct or blocks everything
   - **Mitigation**: Extensive testing, careful type extraction
2. **UI extraction**: Complex state management
   - **Mitigation**: Start with event-driven interface design
3. **Provider extraction**: Breaking changes to LLM interface
   - **Mitigation**: Maintain backward compatibility layer

### Testing Strategy
- **Per-branch**: Unit tests for extracted crate
- **Per-phase**: Integration tests for phase changes
- **Pre-merge**: Full integration test suite
- **Post-merge**: Regression test suite
- **Continuous**: CI runs on all branches

---

## Timeline Estimate

- **Phase 1**: 1-2 weeks (critical, careful)
- **Phase 2**: 2-3 weeks (large extractions)
- **Phase 3**: 2-3 weeks (6 parallel providers)
- **Phase 4**: 2 weeks (tools extraction)
- **Phase 5**: 1-2 weeks (polish)

**Total**: 8-12 weeks with parallel execution

**Accelerated** (with 6 developers): 4-6 weeks

---

## Next Steps

1. **Review this plan** - Discuss and refine with team
2. **Choose Phase 1 executor** - Critical foundation phase
3. **Set up branch infrastructure** - Naming, CI, automation
4. **Create branch templates** - Checklist for each extraction
5. **Begin Phase 1** - Start with 3 parallel branches

---

## Questions to Resolve

1. **Provider Priority**: Which providers should be core vs optional?
   - Recommendation: Core = Anthropic + OpenAI, Optional = rest
2. **Feature Flags**: Granularity of optional features?
   - Recommendation: Provider-level, not model-level
3. **API Stability**: Semver strategy during refactoring?
   - Recommendation: 0.x versions until stable
4. **Migration Path**: Support old imports during transition?
   - Recommendation: Yes, with deprecation warnings
5. **Documentation**: Update docs per-phase or at end?
   - Recommendation: Per-phase for integration branches

---

**Created**: 2025-11-13
**Status**: Proposed
**Complexity**: High
**Impact**: Transformational
**Maintainer**: TBD
