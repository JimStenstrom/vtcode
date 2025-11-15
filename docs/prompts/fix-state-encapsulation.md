# Fix State Encapsulation Compilation Errors

## Context

The `vtcode-ui/src/tui/session/state.rs` file has been refactored to properly encapsulate all state structs by making all fields private and adding accessor methods. This follows best practices but has introduced ~45 compilation errors throughout the codebase that need to be fixed.

## What Was Changed

All 5 state structs now have **private fields with accessor methods**:

1. **DisplayState** - theme, labels, lines, revision counter, code fence flag
2. **PromptState** - prefix, style, placeholder, status left/right
3. **UIState** - boolean flags, dimensions (view_rows, input_height, etc.)
4. **PaletteState** - slash/file/prompt palettes and their active states
5. **RenderState** - header, caches, modal, plan, navigation state

## Accessor Pattern Reference

### Immutable Accessors
```rust
// Returns immutable reference
pub fn theme(&self) -> &InlineTheme { &self.theme }
```

### Mutable Accessors
```rust
// Returns mutable reference
pub fn theme_mut(&mut self) -> &mut InlineTheme { &mut self.theme }
```

### Setters
```rust
// Takes ownership and sets value
pub fn set_theme(&mut self, theme: InlineTheme) { self.theme = theme; }
```

### Convenience Methods
```rust
// Boolean predicates use `is_` prefix
pub fn is_input_enabled(&self) -> bool { self.input_enabled }

// State mutation methods
pub fn request_redraw(&mut self) { self.needs_redraw = true; }
pub fn clear_redraw_flag(&mut self) { self.needs_redraw = false; }
pub fn take_redraw(&mut self) -> bool {
    let needs = self.needs_redraw;
    self.needs_redraw = false;
    needs
}
```

## Current Compilation Status

```bash
cargo check --package vtcode-ui
```

**Errors remaining**: ~45 errors

**Error types**:
- `E0616`: 24 errors - Private field access (need to use accessor methods)
- `E0596`: 8 errors - Cannot borrow as mutable (need `_mut()` accessors)
- `E0599`: 6 errors - No method found (might need different accessor)
- `E0716`: 3 errors - Temporary value dropped (need to bind to variable)
- `E0594`: 3 errors - Cannot assign (need setter methods)
- `E0521`: 1 error - Borrowed data escapes

## Fix Strategy

### Step 1: Fix Direct Field Access (E0616 errors)

**Pattern**: Replace direct field access with accessor methods

**Common conversions**:
```rust
// BEFORE
self.ui.input_enabled         // ERROR: field is private
self.ui.should_exit          // ERROR: field is private
self.display.lines           // ERROR: field is private
self.render.modal            // ERROR: field is private
self.palette.slash_palette   // ERROR: field is private

// AFTER
self.ui.is_input_enabled()   // ✓ Use boolean accessor
self.ui.should_exit()        // ✓ Use accessor
self.display.lines()         // ✓ Use accessor
self.render.modal()          // ✓ Use accessor
self.palette.slash_palette() // ✓ Use accessor
```

### Step 2: Fix Assignment Errors (E0594, E0070)

**Pattern**: Replace `accessor() = value` with setter methods

```rust
// BEFORE
self.ui.should_exit() = true;              // ERROR: invalid left-hand side
self.ui.needs_redraw() = false;            // ERROR: invalid left-hand side
self.prompt.prefix() = prefix;             // ERROR: invalid left-hand side
self.display.theme() = theme;              // ERROR: invalid left-hand side

// AFTER
self.ui.request_exit();                    // ✓ Use convenience method
self.ui.clear_redraw_flag();               // ✓ Use convenience method
self.prompt.set_prefix(prefix);            // ✓ Use setter
self.display.set_theme(theme);             // ✓ Use setter
```

### Step 3: Fix Mutable Borrow Errors (E0596)

**Pattern**: Use `_mut()` accessors when calling mutable methods

```rust
// BEFORE
self.render.navigation_state().select(None);           // ERROR: cannot borrow as mutable
self.palette.file_palette().as_mut()                   // ERROR: cannot borrow as mutable
self.display.lines().last_mut()                        // ERROR: cannot borrow as mutable

// AFTER
self.render.navigation_state_mut().select(None);       // ✓ Use mutable accessor
self.palette.file_palette_mut()                        // ✓ Use mutable accessor
self.display.lines_mut().last_mut()                    // ✓ Use mutable accessor
```

### Step 4: Fix Temporary Value Errors (E0716)

**Pattern**: Bind temporary to variable before pattern matching

```rust
// BEFORE
let Some(palette) = self.palette.file_palette().as_mut() else {
    // ERROR: temporary value dropped while borrowed
};

// AFTER - Option 1: Use mutable accessor directly
let Some(palette) = self.palette.file_palette_mut() else {
    return false;
};

// AFTER - Option 2: Bind to variable first
let binding = self.palette.file_palette_mut();
let Some(palette) = binding else {
    return false;
};
```

## Affected Files

Run this to see which files have errors:
```bash
cargo check --package vtcode-ui 2>&1 | grep "error\[E" | grep "\.rs:" | cut -d: -f1 | sort | uniq -c | sort -rn
```

**Primary files** with errors:
- `vtcode-ui/src/tui/session.rs` (largest file, most errors)
- `vtcode-ui/src/tui/session/input.rs`
- `vtcode-ui/src/tui/session/slash.rs`
- `vtcode-ui/src/tui/session/navigation.rs`

## Complete Accessor Reference

See `vtcode-ui/src/tui/session/state.rs` for all available methods. Here's a quick reference:

### UIState
```rust
// Getters
is_input_enabled() -> bool
is_cursor_visible() -> bool
needs_redraw() -> bool
needs_full_clear() -> bool
should_exit() -> bool
view_rows() -> u16
input_height() -> u16
transcript_rows() -> u16
transcript_width() -> u16
transcript_view_top() -> usize
show_timeline_pane() -> bool

// Setters
set_input_enabled(bool)
set_cursor_visible(bool)
set_needs_full_clear(bool)
set_view_rows(u16)
set_input_height(u16)
set_transcript_rows(u16)
set_transcript_width(u16)
set_transcript_view_top(usize)
set_show_timeline_pane(bool)

// Convenience
request_redraw()
clear_redraw_flag()
take_redraw() -> bool
request_exit()
```

### PromptState
```rust
// Getters
prefix() -> &str
style() -> &InlineTextStyle
placeholder() -> Option<&str>
placeholder_style() -> Option<&InlineTextStyle>
status_left() -> Option<&str>
status_right() -> Option<&str>

// Setters
set_prefix(String)
set_style(InlineTextStyle)
set_placeholder(Option<String>)
set_placeholder_style(Option<InlineTextStyle>)
set_status_left(Option<String>)
set_status_right(Option<String>)

// Convenience
has_left_status() -> bool
has_right_status() -> bool
has_any_status() -> bool
```

### DisplayState
```rust
// Getters
theme() -> &InlineTheme
theme_mut() -> &mut InlineTheme
labels() -> &MessageLabels
labels_mut() -> &mut MessageLabels
lines() -> &[MessageLine]
lines_mut() -> &mut Vec<MessageLine>
current_revision() -> u64
is_in_tool_code_fence() -> bool

// Setters
set_theme(InlineTheme)
set_labels(MessageLabels)
set_in_tool_code_fence(bool)

// Convenience
next_revision() -> u64
```

### PaletteState
```rust
// Getters
slash_palette() -> &SlashPalette
slash_palette_mut() -> &mut SlashPalette
file_palette() -> Option<&FilePalette>
file_palette_mut() -> Option<&mut FilePalette>
is_file_palette_active() -> bool
has_deferred_file_browser_trigger() -> bool
prompt_palette() -> Option<&PromptPalette>
prompt_palette_mut() -> Option<&mut PromptPalette>
is_prompt_palette_active() -> bool
has_deferred_prompt_browser_trigger() -> bool
custom_prompts() -> Option<&CustomPromptRegistry>
custom_prompts_mut() -> Option<&mut CustomPromptRegistry>

// Setters
set_file_palette(Option<FilePalette>)
set_file_palette_active(bool)
set_deferred_file_browser_trigger(bool)
set_prompt_palette(Option<PromptPalette>)
set_prompt_palette_active(bool)
set_deferred_prompt_browser_trigger(bool)
set_custom_prompts(Option<CustomPromptRegistry>)
```

### RenderState
```rust
// Getters
header_context() -> &InlineHeaderContext
header_context_mut() -> &mut InlineHeaderContext
header_rows() -> u16
transcript_cache() -> Option<&TranscriptReflowCache>
transcript_cache_mut() -> &mut Option<TranscriptReflowCache>
queued_inputs() -> &[String]
queued_inputs_mut() -> &mut Vec<String>
queue_overlay_cache() -> Option<&QueueOverlay>
queue_overlay_version() -> u64
modal() -> Option<&ModalState>
modal_mut() -> Option<&mut ModalState>
plan() -> &TaskPlan
plan_mut() -> &mut TaskPlan
navigation_state() -> &ListState
navigation_state_mut() -> &mut ListState

// Setters
set_header_context(InlineHeaderContext)
set_header_rows(u16)
set_transcript_cache(Option<TranscriptReflowCache>)
set_queue_overlay_cache(Option<QueueOverlay>)
set_queue_overlay_version(u64)
set_modal(Option<ModalState>)
set_plan(TaskPlan)

// Convenience
take_transcript_cache() -> Option<TranscriptReflowCache>
take_modal() -> Option<ModalState>
```

## Task Instructions

**Your goal**: Fix all ~45 compilation errors in vtcode-ui by converting direct field access to proper accessor method calls.

**Approach**:
1. Run `cargo check --package vtcode-ui` to see current errors
2. Group errors by type (E0616, E0596, E0594, etc.)
3. Fix them systematically by file, starting with the most common patterns
4. Use the accessor reference above to find the right method
5. Test frequently with `cargo check --package vtcode-ui`
6. When you get to 0 errors, run full check: `cargo check`

**Success criteria**:
- `cargo check --package vtcode-ui` shows 0 errors
- All state struct fields remain private
- All access goes through accessor methods

## Tips

- Use `sed` or similar tools for bulk replacements where patterns are consistent
- For complex cases, read the specific lines and fix manually
- Some accessors return `Option<&T>`, some return `&T` - check the type
- Boolean fields use `is_` prefix: `is_input_enabled()`, not `input_enabled()`
- When in doubt, check `vtcode-ui/src/tui/session/state.rs` for the exact method signature

Good luck!
