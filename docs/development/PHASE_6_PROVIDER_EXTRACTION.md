# Phase 6: Provider Extraction Status & Roadmap

## Current State (After Phase 5)

### Completed Extractions ✅
- `vtcode-llm-gemini` - Fully extracted, uses `vtcode_llm_types::LLMProvider`
- `vtcode-llm-openrouter` - Fully extracted, uses `vtcode_llm_types::LLMProvider`

### Incomplete Extractions ⚠️
These crates exist but have problems:

1. **vtcode-llm-anthropic** - Has its own `LLMProvider` trait instead of using `vtcode_llm_types`
2. **vtcode-llm-openai** - Has its own `LLMProvider` trait instead of using `vtcode_llm_types`
3. **vtcode-llm-microsoft** - Just a wrapper that re-exports from vtcode-core (circular dependency)

### Providers Still in vtcode-core
These need to be extracted:

| Provider | Size | Dependencies | Complexity |
|----------|------|--------------|------------|
| anthropic.rs | 45KB | None | Medium |
| openai.rs | 98KB | None | High |
| microsoft.rs | 17KB | None | Medium |
| deepseek.rs | 29KB | None | Medium |
| moonshot.rs | 20KB | None | Medium |
| ollama.rs | 30KB | None | Medium |
| minimax.rs | 15KB | Wraps Anthropic | Medium |
| zai.rs | 28KB | None | Medium |
| xai.rs | 5KB | Wraps OpenAI | Low |
| lmstudio.rs | 8KB | Wraps OpenAI | Low |

**Total**: ~300KB of provider code in vtcode-core

## Problems Blocking Extraction

### 1. Duplicate LLMProvider Trait Definitions

vtcode-llm-anthropic and vtcode-llm-openai define their own `LLMProvider` trait instead of using the one from `vtcode_llm_types`. This makes them incompatible with vtcode-core.

**Fix Required**:
- Remove local `LLMProvider` trait definitions
- Import from `vtcode_llm_types::LLMProvider`
- Update all trait method signatures to match
- Fix any type mismatches

### 2. Provider Dependencies

Some providers wrap others:
- `minimax.rs` wraps `AnthropicProvider`
- `xai.rs` wraps `OpenAIProvider`  
- `lmstudio.rs` wraps `OpenAIProvider`

**Fix Required**:
- Extract wrapper providers AFTER their dependencies
- Or make them depend on the standalone crate (e.g., `vtcode-llm-xai` depends on `vtcode-llm-openai`)

### 3. Circular Dependencies

`vtcode-llm-microsoft` depends on `vtcode-core` to re-export `MicrosoftProvider`, creating a circular dependency.

**Fix Required**:
- Actually move the `microsoft.rs` implementation into `vtcode-llm-microsoft`
- Remove the re-export hack

## Phase 6 Extraction Roadmap

### Phase 6A: Fix Existing Standalone Crates ⏳

**Goal**: Make vtcode-llm-anthropic and vtcode-llm-openai actually work

Tasks:
1. Fix vtcode-llm-anthropic
   - Remove local `LLMProvider` trait
   - Use `vtcode_llm_types::LLMProvider`
   - Remove local type definitions that duplicate vtcode_llm_types
   - Verify compilation

2. Fix vtcode-llm-openai
   - Remove local `LLMProvider` trait  
   - Use `vtcode_llm_types::LLMProvider`
   - Remove duplicate types
   - Verify compilation

3. Fix vtcode-llm-microsoft
   - Move actual `MicrosoftProvider` implementation from vtcode-core
   - Remove circular dependency
   - Verify compilation

4. Remove duplicates from vtcode-core
   - Delete `anthropic.rs`, `openai.rs`, `microsoft.rs` from vtcode-core/src/llm/providers/
   - Import from standalone crates in providers/mod.rs
   - Update wrapper providers (minimax, xai, lmstudio) to use standalone crates
   - Verify compilation

**Estimated Time**: 2-3 hours
**LOC Reduction**: ~160KB

### Phase 6B: Extract Standalone Providers

Extract the 5 standalone providers that don't wrap others:

1. **vtcode-llm-deepseek** (29KB)
2. **vtcode-llm-moonshot** (20KB)  
3. **vtcode-llm-ollama** (30KB)
4. **vtcode-llm-zai** (28KB)

For each provider:
1. Create new crate: `mkdir vtcode-llm-{name}`
2. Copy Cargo.toml template from vtcode-llm-gemini
3. Move provider.rs implementation
4. Add to workspace in root Cargo.toml
5. Update vtcode-core to import from new crate
6. Delete from vtcode-core/src/llm/providers/
7. Verify compilation

**Estimated Time**: 3-4 hours (30-60 min per provider)
**LOC Reduction**: ~107KB

### Phase 6C: Extract Wrapper Providers

Extract the wrapper providers (depend on standalone crates):

1. **vtcode-llm-minimax** (15KB) - depends on vtcode-llm-anthropic
2. **vtcode-llm-xai** (5KB) - depends on vtcode-llm-openai
3. **vtcode-llm-lmstudio** (8KB) - depends on vtcode-llm-openai

For each:
1. Create new crate
2. Add dependency on wrapped provider crate
3. Move implementation
4. Update vtcode-core
5. Delete from vtcode-core
6. Verify compilation

**Estimated Time**: 1-2 hours
**LOC Reduction**: ~28KB

### Phase 6D: Final Cleanup

1. Update vtcode-llm aggregator crate to re-export all providers
2. Update factory.rs to use standalone crates
3. Remove vtcode-core/src/llm/providers/ directory entirely
4. Update documentation
5. Run full test suite
6. Update ARCHITECTURE_TRANSFORMATION.md

**Estimated Time**: 1 hour

## Total Phase 6 Effort

**Time**: 7-10 hours
**LOC Reduction**: ~295KB removed from vtcode-core
**New Crates**: 7 additional provider crates
**Result**: Complete provider modularization, vtcode-core reduced to ~50KB

## Breaking Changes

### For vtcode CLI Users
**Impact**: None - CLI interface unchanged

### For vtcode-core Library Users
**Impact**: High - Import paths change

**Before**:
```rust
use vtcode_core::llm::providers::{
    AnthropicProvider, OpenAIProvider, DeepSeekProvider
};
```

**After**:
```rust
use vtcode_llm_anthropic::AnthropicProvider;
use vtcode_llm_openai::OpenAIProvider;
use vtcode_llm_deepseek::DeepSeekProvider;
```

### For Extension Developers
**Impact**: Positive - Easier to create custom providers

## Success Criteria

- ✅ All 12 providers in standalone crates
- ✅ Zero circular dependencies
- ✅ vtcode-core/src/llm/providers/ directory deleted
- ✅ All providers use `vtcode_llm_types::LLMProvider`
- ✅ Full test suite passes
- ✅ Documentation updated

## Next Steps

Run:
```bash
git add docs/development/PHASE_6_PROVIDER_EXTRACTION.md
git commit -m "docs: Add Phase 6 provider extraction roadmap"
```

Then proceed with Phase 6A tasks.
