# Phase 8: Wrapper Provider Extraction Report

**Date**: 2025-11-15
**Task**: Extract Minimax and xAI wrapper providers from vtcode-core to standalone crates
**Status**: COMPLETED

## Overview

Successfully extracted two wrapper providers (Minimax and xAI) from vtcode-core into standalone crates. These providers are unique because they wrap other provider implementations rather than implementing direct API integrations.

## Extracted Crates

### 1. vtcode-llm-minimax

**Location**: `/home/jim/github/vtcode/vtcode-llm-minimax/`

**Description**: Minimax provider that wraps AnthropicProvider with custom tool call parsing

**Files Created**:
- `Cargo.toml` (903 bytes)
- `src/lib.rs` (3 lines)
- `src/minimax.rs` (438 lines)

**Key Dependencies**:
- `vtcode-llm-anthropic` - Wraps Anthropic provider
- `vtcode-llm-common` - Common utilities
- `vtcode-llm-types` - LLM trait definitions
- `vtcode-config` - Configuration and model constants
- `regex` - XML-like tool call parsing

**Special Features**:
- Custom XML-like tool call format: `<minimax:tool_call>` blocks
- Parses `<invoke>` and `<parameter>` blocks from response text
- Type conversion for parameters (string, integer, number, boolean, object, array)
- Post-processing to extract tool calls from text content

**Tests**: 6 tests, all passing
- `parse_handles_missing_blocks`
- `post_processing_infers_tool_calls`
- `parse_complex_tool_call_parameters`
- `parse_multiple_tool_calls_in_sequence`
- `parses_minimax_tool_calls_and_arguments`
- `supported_models_returns_non_empty_list`

### 2. vtcode-llm-xai

**Location**: `/home/jim/github/vtcode/vtcode-llm-xai/`

**Description**: xAI (Grok) provider that wraps OpenAIProvider for OpenAI-compatible API

**Files Created**:
- `Cargo.toml` (753 bytes)
- `src/lib.rs` (3 lines)
- `src/xai.rs` (170 lines)

**Key Dependencies**:
- `vtcode-llm-openai` - Wraps OpenAI provider
- `vtcode-llm-common` - Common utilities
- `vtcode-llm-types` - LLM trait definitions
- `vtcode-config` - Configuration and model constants

**Special Features**:
- Supports reasoning models (Grok-4, Grok-4-Code)
- Prompt caching support (platform-managed)
- Custom base URL and environment variable support

**Tests**: 5 tests, all passing
- `with_model_creates_provider_with_custom_model`
- `new_creates_provider_with_default_model`
- `from_config_uses_defaults_when_none`
- `supported_models_returns_non_empty_list`
- `wraps_openai_provider`

## Import Transformations

### Common Pattern Applied

**Old (vtcode-core)**:
```rust
use crate::config::constants::*;
use crate::llm::provider::*;
use super::common::*;
```

**New (standalone crate)**:
```rust
use vtcode_config::constants::*;
use vtcode_llm_types::*;
use vtcode_llm_common::*;
```

### Minimax-Specific Changes

```rust
// OLD
use super::AnthropicProvider;
use crate::config::constants::models;
use crate::config::core::PromptCachingConfig;
use crate::llm::provider::{...};

// NEW
use vtcode_llm_anthropic::AnthropicProvider;
use vtcode_config::constants::models;
use vtcode_config::core::PromptCachingConfig;
use vtcode_llm_types::{...};
```

### xAI-Specific Changes

```rust
// OLD
use crate::config::constants::{env_vars, models, urls};
use crate::llm::error_display;
use super::common::{forward_prompt_cache_with_state, ...};

// NEW
use vtcode_config::constants::{env_vars, models, urls};
use vtcode_llm_common::{format_llm_error, forward_prompt_cache_with_state, ...};
```

## vtcode-llm-common Enhancement

Added missing `forward_prompt_cache_with_state` function to vtcode-llm-common:

**Function**: `forward_prompt_cache_with_state`
- Returns `(bool, Option<PromptCachingConfig>)` tuple
- Used by wrapper providers to determine if caching should be enabled
- Applies predicate to config and returns state + optional config

**Location**: `/home/jim/github/vtcode/vtcode-llm-common/src/config.rs` (lines 94-117)

**Export**: Added to `vtcode-llm-common/src/lib.rs` public API

## Workspace Updates

**File**: `/home/jim/github/vtcode/Cargo.toml`

**Changes**: Added to workspace members list:
```toml
[workspace]
members = [
    # ... existing members ...
    "vtcode-llm-minimax",
    "vtcode-llm-xai",
    # ... more members ...
]
```

## Compilation Verification

**Command**: `cargo check -p vtcode-llm-minimax -p vtcode-llm-xai`

**Result**: SUCCESS

**Warnings** (non-critical):
- `vtcode-llm-types`: unexpected cfg condition value: `schema`
- `vtcode-config`: unused import: `crate::constants::reasoning`

Both warnings are pre-existing and unrelated to the extraction.

## Test Execution Results

### vtcode-llm-minimax
```
running 6 tests
test minimax::tests::parse_handles_missing_blocks ... ok
test minimax::tests::post_processing_infers_tool_calls ... ok
test minimax::tests::parse_complex_tool_call_parameters ... ok
test minimax::tests::parse_multiple_tool_calls_in_sequence ... ok
test minimax::tests::parses_minimax_tool_calls_and_arguments ... ok
test minimax::tests::supported_models_returns_non_empty_list ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

### vtcode-llm-xai
```
running 5 tests
test xai::tests::with_model_creates_provider_with_custom_model ... ok
test xai::tests::new_creates_provider_with_default_model ... ok
test xai::tests::from_config_uses_defaults_when_none ... ok
test xai::tests::supported_models_returns_non_empty_list ... ok
test xai::tests::wraps_openai_provider ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

## Code Statistics

**Total Lines Extracted**: 614 lines
- vtcode-llm-minimax: 441 lines (438 + 3)
- vtcode-llm-xai: 173 lines (170 + 3)

**Lines Added to vtcode-llm-common**: 24 lines
- `forward_prompt_cache_with_state` function + documentation

## Architecture Benefits

### Separation of Concerns
- Wrapper providers now isolated from core implementation
- Clear dependency chain: wrapper → base provider → types/config
- Each wrapper can evolve independently

### Dependency Reduction
- vtcode-core no longer needs Minimax/xAI-specific code
- Wrapper providers pull in only what they need
- Smaller compilation units for faster builds

### Maintainability
- Wrapper logic isolated in dedicated crates
- Easy to add new wrapper providers following this pattern
- Tests co-located with implementation

## Wrapper Provider Pattern

Both crates demonstrate the wrapper provider pattern:

1. **Struct wraps base provider**:
   ```rust
   pub struct MinimaxProvider {
       inner: AnthropicProvider,
   }
   ```

2. **Delegate most functionality**:
   ```rust
   fn supports_streaming(&self) -> bool {
       self.inner.supports_streaming()
   }
   ```

3. **Add custom logic where needed**:
   - Minimax: Post-process responses to extract custom tool calls
   - xAI: Configure base URL and prompt caching for xAI-specific API

4. **Implement LLMProvider trait**:
   - Override specific methods
   - Delegate others to wrapped provider

## Next Steps

Phase 8 is complete. The remaining provider extraction tasks are:

- None - All providers have been extracted from vtcode-core

All 11 LLM providers are now in standalone crates:
1. vtcode-llm-anthropic ✓
2. vtcode-llm-openai ✓
3. vtcode-llm-gemini ✓
4. vtcode-llm-openrouter ✓
5. vtcode-llm-directline ✓
6. vtcode-llm-ollama ✓
7. vtcode-llm-zai ✓
8. vtcode-llm-deepseek ✓
9. vtcode-llm-moonshot ✓
10. vtcode-llm-lmstudio ✓
11. vtcode-llm-minimax ✓ (wrapper)
12. vtcode-llm-xai ✓ (wrapper)

## Verification Checklist

- [x] Created vtcode-llm-minimax crate structure
- [x] Created vtcode-llm-xai crate structure
- [x] Updated workspace Cargo.toml
- [x] Added missing `forward_prompt_cache_with_state` to vtcode-llm-common
- [x] Updated imports from crate-relative to absolute
- [x] All tests passing (11 total)
- [x] Compilation successful with no errors
- [x] Documentation complete

## Conclusion

Phase 8 successfully extracted the two wrapper providers (Minimax and xAI) from vtcode-core into standalone crates. Both crates compile cleanly, all tests pass, and the workspace is properly configured. The extraction demonstrates the wrapper provider pattern that can be used for future providers that need to wrap or adapt existing provider implementations.

The key insight from this phase is that wrapper providers need an additional dependency on the base provider they wrap, but otherwise follow the same extraction pattern as direct-implementation providers.
