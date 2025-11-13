# vtcode-ui Testing Plan

**Status**: 🔄 Prepared - Awaiting Dependency Resolution
**Created**: November 13, 2025

---

## Current Testing Status

### ❌ Tests Cannot Run Yet

vtcode-ui cannot be compiled independently yet due to unresolved dependencies on vtcode-core:
- `crate::config::*` imports (12 occurrences)
- `crate::prompts::*` imports (6 occurrences)
- `crate::tools::*` imports (5 occurrences)
- `crate::utils::*` imports (4 occurrences)

**Once dependencies are resolved**, we can run the full test suite.

### ✅ Existing Test

One test exists in `src/lib.rs`:
```rust
#[test]
fn test_render_markdown() {
    // Tests that markdown rendering doesn't panic
}
```

**Status**: Will work once dependencies resolved

---

## Testing Strategy

### Phase 1: Unit Tests (Per Module)

Test each module in isolation:

#### 1. Theme System Tests
**Module**: `theme.rs`, `theme_config.rs`, `theme_manager.rs`

```rust
// tests/theme_tests.rs
#[test]
fn test_theme_palette_creation() { }

#[test]
fn test_contrast_adjustment() { }

#[test]
fn test_theme_switching() { }

#[test]
fn test_custom_theme_loading() { }
```

**Coverage Target**: >90%

#### 2. Markdown Rendering Tests
**Module**: `markdown.rs`

```rust
// tests/markdown_tests.rs
#[test]
fn test_render_headings() { }

#[test]
fn test_render_code_blocks() { }

#[test]
fn test_render_inline_code() { }

#[test]
fn test_render_lists() { }

#[test]
fn test_render_emphasis() { }

#[test]
fn test_syntax_highlighting() { }
```

**Coverage Target**: >85%

#### 3. Search Tests
**Module**: `search.rs`

```rust
// tests/search_tests.rs
#[test]
fn test_fuzzy_match() { }

#[test]
fn test_normalize_query() { }

#[test]
fn test_empty_query() { }

#[test]
fn test_unicode_search() { }
```

**Coverage Target**: >95%

#### 4. Diff Renderer Tests
**Module**: `diff_renderer.rs`

```rust
// tests/diff_tests.rs
#[test]
fn test_render_addition() { }

#[test]
fn test_render_deletion() { }

#[test]
fn test_render_modification() { }

#[test]
fn test_color_application() { }
```

**Coverage Target**: >80%

#### 5. TUI Component Tests
**Module**: `tui/*.rs`

```rust
// tests/tui_tests.rs
#[test]
fn test_session_creation() { }

#[test]
fn test_message_rendering() { }

#[test]
fn test_input_handling() { }

#[test]
fn test_modal_display() { }

#[test]
fn test_palette_filtering() { }
```

**Coverage Target**: >75%

---

### Phase 2: Integration Tests

Test how components work together:

#### 1. Theme + Markdown Integration
```rust
#[test]
fn test_themed_markdown_rendering() {
    // Apply theme to markdown output
}
```

#### 2. Search + File Palette Integration
```rust
#[test]
fn test_file_search_in_palette() {
    // Test fuzzy search within file palette
}
```

#### 3. Session + Message Integration
```rust
#[test]
fn test_session_message_flow() {
    // Test adding messages to session
}
```

---

### Phase 3: Visual Regression Tests

Test UI rendering output (future):

```rust
#[test]
fn test_terminal_output_snapshot() {
    // Snapshot testing for terminal output
}
```

---

## Test Infrastructure

### Directory Structure

```
vtcode-ui/
├── src/
│   └── lib.rs (with inline tests)
├── tests/
│   ├── theme_tests.rs
│   ├── markdown_tests.rs
│   ├── search_tests.rs
│   ├── diff_tests.rs
│   ├── tui_tests.rs
│   └── integration/
│       ├── theme_markdown_tests.rs
│       ├── search_palette_tests.rs
│       └── session_flow_tests.rs
└── benches/
    └── rendering_benchmarks.rs
```

### Test Dependencies (in Cargo.toml)

```toml
[dev-dependencies]
assert_fs = "1.1"      # Already present
tempfile = "3.0"       # Already present
insta = "1.39"         # For snapshot testing (add)
criterion = "0.7"      # For benchmarks (add)
proptest = "1.4"       # For property testing (add)
```

---

## Testing Checklist

### Before Tests Can Run
- [ ] Resolve all `crate::config::*` dependencies
- [ ] Resolve all `crate::prompts::*` dependencies
- [ ] Resolve all `crate::tools::*` dependencies
- [ ] Resolve all `crate::utils::*` dependencies
- [ ] Verify vtcode-ui compiles independently
- [ ] Add test dependencies to Cargo.toml

### Unit Test Implementation
- [ ] Theme system tests (theme.rs, theme_config.rs, theme_manager.rs)
- [ ] Markdown rendering tests (markdown.rs)
- [ ] Search tests (search.rs)
- [ ] Diff renderer tests (diff_renderer.rs)
- [ ] File colorizer tests (file_colorizer.rs)
- [ ] Git config tests (git_config.rs)
- [ ] Slash command tests (slash.rs)
- [ ] TUI session tests (tui/session.rs)
- [ ] TUI types tests (tui/types.rs)
- [ ] TUI modal tests (tui/session/modal.rs)
- [ ] TUI palette tests (tui/session/*_palette.rs)

### Integration Test Implementation
- [ ] Theme + Markdown integration
- [ ] Search + Palette integration
- [ ] Session + Message flow
- [ ] Input + Navigation integration

### Coverage Verification
- [ ] Run coverage report: `cargo tarpaulin -p vtcode-ui`
- [ ] Verify >85% overall coverage
- [ ] Document any uncovered edge cases

### Performance Testing
- [ ] Benchmark markdown rendering
- [ ] Benchmark theme switching
- [ ] Benchmark search performance
- [ ] Benchmark TUI rendering

---

## Test Execution Commands

### Run All Tests
```bash
cargo test -p vtcode-ui
```

### Run Specific Test Module
```bash
cargo test -p vtcode-ui --test theme_tests
```

### Run With Output
```bash
cargo test -p vtcode-ui -- --nocapture --test-threads=1
```

### Run Integration Tests Only
```bash
cargo test -p vtcode-ui --test '*' --tests
```

### Generate Coverage Report
```bash
cargo tarpaulin -p vtcode-ui --out Html --output-dir coverage/
```

### Run Benchmarks
```bash
cargo bench -p vtcode-ui
```

---

## Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| theme.rs | >90% | Core functionality, pure logic |
| markdown.rs | >85% | Complex rendering, many branches |
| search.rs | >95% | Simple algorithms, easy to test |
| diff_renderer.rs | >80% | Visual output, hard to assert |
| tui/session.rs | >75% | Complex state management |
| tui/types.rs | >90% | Data structures, simple logic |
| Overall | >85% | Industry standard for libraries |

---

## Known Testing Challenges

### 1. Terminal Output Testing
**Challenge**: Hard to assert visual output
**Solution**:
- Use snapshot testing (insta crate)
- Test data structures rather than rendered output
- Mock terminal for deterministic output

### 2. Interactive Components
**Challenge**: Testing user input and navigation
**Solution**:
- Test event handlers in isolation
- Mock keyboard events
- Test state changes, not UI updates

### 3. Theme System
**Challenge**: Color contrast calculations
**Solution**:
- Test with known good/bad color combinations
- Property testing for edge cases
- Visual inspection for final verification

### 4. Async Components
**Challenge**: TUI session with async operations
**Solution**:
- Use tokio test runtime
- Mock async dependencies
- Test state transitions

---

## Test Examples

### Example: Theme Test

```rust
#[test]
fn test_theme_contrast_adjustment() {
    let palette = ThemePalette {
        primary_accent: RgbColor(137, 220, 235),
        background: RgbColor(24, 24, 37),
        foreground: RgbColor(205, 214, 244),
        secondary_accent: RgbColor(203, 166, 247),
        alert: RgbColor(243, 139, 168),
        logo_accent: RgbColor(166, 227, 161),
    };

    let styles = palette.build_styles();

    // Verify contrast ratios meet WCAG standards
    let contrast = calculate_contrast(
        styles.primary.fg_color().unwrap(),
        palette.background
    );

    assert!(contrast >= 4.5, "Contrast ratio too low: {}", contrast);
}
```

### Example: Markdown Test

```rust
#[test]
fn test_render_code_block() {
    let markdown = r#"
```rust
fn main() {
    println!("Hello, world!");
}
```
"#;

    let rendered = render_markdown(markdown);

    // Verify syntax highlighting was applied
    assert!(rendered.contains("\x1b["), "No ANSI codes found");
    // Verify content preserved
    assert!(rendered.contains("println!"));
}
```

### Example: Search Test

```rust
#[test]
fn test_fuzzy_match_scoring() {
    let results = fuzzy_match("mdwn", &[
        "markdown.rs",
        "readme.md",
        "main.rs",
        "modern_tui.rs"
    ]);

    // "markdown.rs" should score highest
    assert_eq!(results[0].0, "markdown.rs");
    assert!(results[0].1 > 0.5, "Score too low");
}
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: vtcode-ui tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test -p vtcode-ui
      - name: Check coverage
        run: cargo tarpaulin -p vtcode-ui --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

---

## Documentation Tests

All public API examples in documentation will be tested:

```rust
/// Renders markdown with syntax highlighting
///
/// # Example
/// ```
/// use vtcode_ui::render_markdown;
///
/// let markdown = "# Hello\n\nThis is **bold**.";
/// let output = render_markdown(markdown);
/// assert!(!output.is_empty());
/// ```
pub fn render_markdown(input: &str) -> String {
    // ...
}
```

Run with: `cargo test -p vtcode-ui --doc`

---

## Test Maintenance

### Adding New Tests
1. Create test file in appropriate directory
2. Follow naming convention: `test_<feature>_<scenario>`
3. Add to test list in this document
4. Update coverage targets if needed

### Updating Tests
1. When API changes, update affected tests
2. Update snapshots if using insta
3. Document breaking changes
4. Update examples in README

### Deprecating Tests
1. Don't delete, mark as `#[ignore]` with reason
2. Remove after 2 releases if no longer relevant

---

## Success Criteria

Before merging to main:
- [ ] All tests pass
- [ ] Coverage >85%
- [ ] No failing doc tests
- [ ] Benchmarks show acceptable performance
- [ ] CI pipeline green

---

## Future Enhancements

### Phase 4: Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_theme_contrast_always_readable(
        fg: (u8, u8, u8),
        bg: (u8, u8, u8)
    ) {
        let palette = ThemePalette::with_colors(fg, bg);
        let styles = palette.build_styles();
        assert!(verify_contrast(styles.primary, palette.background));
    }
}
```

### Phase 5: Fuzz Testing
```rust
#[test]
fn fuzz_markdown_rendering() {
    // Test that arbitrary input never crashes
}
```

### Phase 6: Visual Regression
- Automated screenshot comparison
- Terminal output diffing
- Theme preview generation

---

## Notes

- Tests will be added incrementally during dependency resolution
- Mock implementations will be created for vtcode-core dependencies
- Integration tests will verify vtcode-ui works with vtcode-core
- Performance benchmarks will establish baselines

---

**Status**: 📋 Plan Ready - Implementation Pending Dependency Resolution
**Next Step**: Resolve dependencies, then implement tests module by module
**Target**: >85% coverage before merge to main

---

**Version**: 1.0
**Last Updated**: November 13, 2025
