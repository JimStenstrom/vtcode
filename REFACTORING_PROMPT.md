# High Priority Refactoring Tasks for vtcode

## Context
You are working on vtcode, a Rust-based terminal coding agent with 30 crates and 623 files. A code review identified several critical technical debt items that need immediate attention. Your task is to systematically fix these issues while maintaining all existing functionality and passing all tests.

## High Priority Fixes

### Priority 1: Eliminate `BuiltinProvider` Boilerplate in factory.rs
**File:** `vtcode-core/src/llm/factory.rs` (lines 223-437)

**Problem:**
There are 12 nearly identical `BuiltinProvider` implementations consuming ~200 lines of pure boilerplate code.

**Task:**
1. Create a declarative macro `impl_builtin_provider!` that generates the boilerplate
2. Replace all 12 manual implementations with macro invocations
3. The macro should handle the pattern:
   - Destructure `ProviderConfig`
   - Call provider's `from_config()` method
   - Box the result
4. Ensure all providers continue to work identically
5. Run tests to verify no regressions

**Expected Result:** Reduce ~200 lines to ~20 lines while maintaining identical behavior.

**Example:**
```rust
macro_rules! impl_builtin_provider {
    ($provider:ty) => {
        impl BuiltinProvider for $provider {
            fn build_from_config(config: ProviderConfig) -> Box<dyn LLMProvider> {
                let ProviderConfig { api_key, base_url, model, prompt_cache } = config;
                Box::new(<$provider>::from_config(api_key, model, base_url, prompt_cache))
            }
        }
    };
}

// Then use it:
impl_builtin_provider!(GeminiProvider);
impl_builtin_provider!(OpenAIProvider);
// ... etc for all 12 providers
```

---

### Priority 2: Move Regex Compilation to Static in Minimax Provider
**File:** `vtcode-llm-minimax/src/minimax.rs` (lines 145-160)

**Problem:**
Regex patterns are compiled on EVERY function call in the hot path, causing significant performance overhead.

**Task:**
1. Convert the three regex patterns to `static` using `LazyLock`
2. Replace inline `Regex::new()` calls with static references
3. Update error handling since regexes are now infallible (use `.expect()` in static init)
4. Run performance benchmarks if available to measure improvement

**Current Code (lines 149-159):**
```rust
fn parse_minimax_tool_calls(text: &str, tools: Option<&[ToolDefinition]>) -> (Vec<ToolCall>, String) {
    let tool_call_regex = Regex::new(TOOL_CALL_BLOCK_PATTERN).ok();
    let invoke_regex = Regex::new(INVOKE_BLOCK_PATTERN).ok();
    let parameter_regex = Regex::new(PARAMETER_BLOCK_PATTERN).ok();

    if tool_call_regex.is_none() || invoke_regex.is_none() || parameter_regex.is_none() {
        return (Vec::new(), text.to_string());
    }

    let tool_call_regex = tool_call_regex.unwrap();  // Line 157
    let invoke_regex = invoke_regex.unwrap();        // Line 158
    let parameter_regex = parameter_regex.unwrap();  // Line 159
    // ...
}
```

**Expected Code:**
```rust
use std::sync::LazyLock;

static TOOL_CALL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(TOOL_CALL_BLOCK_PATTERN).expect("valid tool_call regex"));
static INVOKE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(INVOKE_BLOCK_PATTERN).expect("valid invoke regex"));
static PARAMETER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PARAMETER_BLOCK_PATTERN).expect("valid parameter regex"));

fn parse_minimax_tool_calls(text: &str, tools: Option<&[ToolDefinition]>) -> (Vec<ToolCall>, String) {
    // Use &*TOOL_CALL_REGEX instead of runtime compilation
    // ...
}
```

---

### Priority 3: Remove Unnecessary Mutex from Global Factory
**File:** `vtcode-core/src/llm/factory.rs` (lines 173-181)

**Problem:**
The global factory uses `Mutex<LLMFactory>` but the factory is read-only after initialization. This adds unnecessary synchronization overhead and potential contention.

**Current Code:**
```rust
static FACTORY: LazyLock<Mutex<LLMFactory>> = LazyLock::new(|| Mutex::new(LLMFactory::new()));

pub fn get_factory() -> &'static Mutex<LLMFactory> {
    &FACTORY
}
```

**Task:**
1. Change `LazyLock<Mutex<LLMFactory>>` to `LazyLock<LLMFactory>`
2. Update `get_factory()` to return `&'static LLMFactory`
3. Find all call sites that do `.lock().unwrap()` and remove the locking
4. Update `LLMFactory` methods to take `&self` instead of `&mut self` where appropriate
5. Verify `register_provider()` is not called after initialization (should only be called in `new()`)
6. Run all tests to ensure thread safety is maintained

**Expected Impact:** Eliminate global mutex contention, improve concurrent performance.

---

### Priority 4: Fix `.unwrap()` Violations
**Files:** Multiple (117 instances found)

**Problem:**
The workspace Cargo.toml declares `unwrap_used = "deny"` but there are 117 violations in the codebase.

**Task:**
1. Run `cargo clippy --workspace -- -D clippy::unwrap_used` to find all violations
2. For each violation, replace with proper error handling:
   - Use `?` operator where Result propagation is appropriate
   - Use `if let Some()` or `match` for Options
   - Use `.expect()` with descriptive messages ONLY for infallible cases (like static initialization)
   - Use `.unwrap_or()`, `.unwrap_or_else()`, or `.unwrap_or_default()` where fallbacks are acceptable
3. Focus on production code first, tests can use `.unwrap()` where appropriate
4. Pay special attention to:
   - `vtcode-llm-minimax/src/minimax.rs:157-159` (these are already fixed by Priority 2)
   - `vtcode-llm-common/src/config.rs:12` (HTTP client creation)
   - Any unwraps in error handling paths
5. Run full test suite after fixes

**Key Files with Violations:**
- `vtcode-llm-zai/src/zai.rs` (4 instances)
- `vtcode-llm-minimax/src/minimax.rs` (4 instances)
- `vtcode-llm-common/src/reasoning.rs` (2 instances)
- `vtcode-llm-moonshot/src/moonshot.rs` (1 instance)
- `vtcode-llm-anthropic/src/anthropic.rs` (1 instance)
- `vtcode-llm-gemini/src/provider.rs` (1 instance)

**Decision Framework:**
- **Infallible operations** → Keep `.unwrap()` but document WHY it's safe with a comment, or use `.expect("why this is safe")`
- **Recoverable errors** → Propagate with `?` or handle explicitly
- **Optional values** → Use pattern matching or `unwrap_or_*` variants
- **Test code** → Can keep `.unwrap()` but consider `.expect()` for better test failure messages

---

### Priority 5: Refactor session.rs God Object
**File:** `vtcode-ui/src/tui/session.rs` (4,762 lines, 32 fields)

**Problem:**
This is a massive "god object" that violates Single Responsibility Principle and is unmaintainable.

**Task:**
This is the most complex refactoring. Break into phases:

**Phase 1: Extract State Objects**
1. Create `session/state.rs` with distinct state structs:
   - `MessageState` (lines, theme, labels, header_context)
   - `RenderState` (transcript_cache, queue_overlay_cache, modal)
   - `InputState` (already have InputManager, consolidate related fields)
   - `PaletteState` (file_palette, prompt_palette, slash_palette, custom_prompts)
   - `LayoutState` (view_rows, input_height, transcript_rows, etc.)

**Phase 2: Extract Managers**
2. Create `session/event_handler.rs` for event processing logic
3. Create `session/renderer.rs` for rendering logic (likely 1000+ lines)
4. Keep existing managers (InputManager, ScrollManager)

**Phase 3: Refactor Main Session**
5. The main `Session` struct becomes a coordinator:
   ```rust
   pub struct Session {
       state: SessionState,
       event_handler: EventHandler,
       renderer: SessionRenderer,
   }
   ```

**Phase 4: Module Organization**
6. Create `session/` directory with:
   - `mod.rs` - Main Session struct (coordinator)
   - `state.rs` - State objects
   - `event_handler.rs` - Event processing
   - `renderer.rs` - Rendering logic
   - Keep existing submodules (file_palette, input_manager, etc.)

**Constraints:**
- Maintain 100% backward compatibility with public API
- All existing tests must pass
- No changes to external interfaces
- Use type-state pattern where appropriate
- Each file should be <500 lines

**Note:** This is the largest refactoring. Consider doing it last after other high-priority items are complete and tests are green.

---

## Testing Strategy

After EACH priority fix:

1. **Run unit tests:**
   ```bash
   cargo test --workspace
   ```

2. **Run clippy:**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

3. **Check for regressions:**
   ```bash
   cargo build --release
   ./target/release/vtcode --help
   ```

4. **Run integration tests:**
   ```bash
   cargo test --test integration_tests
   ```

## Success Criteria

- [ ] All tests pass (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Build succeeds in release mode
- [ ] No performance regressions (especially for Priority 2)
- [ ] Code is more maintainable (subjective but measurable by LoC reduction)
- [ ] No `.unwrap()` in production code (Priority 4)
- [ ] Factory.rs reduced from 437 to <250 lines (Priority 1)
- [ ] Session.rs broken into modules <500 lines each (Priority 5)

## Recommended Order

1. **Start with Priority 1** (BuiltinProvider macro) - Low risk, high impact, easy to verify
2. **Then Priority 2** (Regex static) - Low risk, performance improvement, isolated change
3. **Then Priority 3** (Remove Mutex) - Medium risk, requires careful review of call sites
4. **Then Priority 4** (Fix unwraps) - Medium risk, requires case-by-case analysis
5. **Finally Priority 5** (Session refactor) - High risk, largest change, do last

## Git Strategy

Create separate commits for each priority:
- `refactor: eliminate BuiltinProvider boilerplate with macro`
- `perf: move minimax regex compilation to static initialization`
- `refactor: remove unnecessary Mutex from global LLMFactory`
- `fix: replace unwrap() calls with proper error handling`
- `refactor: break up session.rs god object into modules`

This allows easy bisecting if issues arise and clear change history.

## Notes

- The codebase uses edition 2024 - ensure you're using a compatible Rust toolchain
- LazyLock is stable in Rust 1.80+
- The workspace has strict clippy lints - leverage them during refactoring
- There are 81 test files - use them to verify changes

---

## Additional Context

- This is a production codebase with 30 crates
- Recent work (Phase 8) successfully extracted providers to standalone crates
- The team values clean architecture and modularity
- Performance matters - this is a terminal UI application
- All changes should maintain backward compatibility
