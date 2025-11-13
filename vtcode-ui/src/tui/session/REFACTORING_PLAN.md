# Session.rs Refactoring Plan

## Current State
- **File size:** 4,762 lines
- **Methods:** 170 methods in Session impl
- **Fields:** 55+ fields in Session struct
- **Status:** Partially modularized (some sub-modules exist)

## Problem Statement
The Session struct is a "God Object" that manages too many responsibilities:
- UI state management
- Event handling (keyboard, mouse, scroll)
- Rendering (transcript, palettes, modals, headers)
- Input management
- Style and formatting
- Layout calculations

This makes the code difficult to:
- Understand and maintain
- Test individual components
- Extend with new features
- Debug issues

## Existing Modules (Already Extracted)
✅ `file_palette.rs` (27KB) - File browser palette
✅ `input.rs` (14KB) - Input handling primitives
✅ `input_manager.rs` (9KB) - Input manager
✅ `modal.rs` (49KB) - Modal dialogs (still needs splitting)
✅ `prompt_palette.rs` (17KB) - Prompt browser
✅ `scroll.rs` (7KB) - Scroll management
✅ `slash_palette.rs` (22KB) - Slash command palette
✅ `transcript.rs` (8KB) - Transcript rendering

## New Modules Created

### ✅ state.rs (230 lines)
Organizes Session fields into 5 logical groups:
1. **DisplayState** - Theme, messages, labels, revision tracking
2. **PromptState** - Prompt display configuration
3. **UIState** - UI flags and dimensions
4. **PaletteState** - File/prompt/slash palette state
5. **RenderState** - Rendering caches and overlays

**Benefits:**
- Reduces Session struct complexity
- Groups related fields logically
- Each state struct is self-contained and testable
- Clear ownership boundaries

## Proposed Additional Modules

### 📋 event_handler.rs (~500 lines)
**Responsibilities:**
- Handle keyboard events
- Handle mouse events
- Handle scroll events
- Emit inline events
- Command dispatch

**Methods to extract:**
- `handle_event()`
- `handle_command()`
- `handle_scroll_down()`
- `handle_scroll_up()`
- `emit_inline_event()`
- All palette event handlers
- Modal interaction handlers

**Benefits:**
- Isolates event handling logic
- Easier to test input/event flows
- Clear separation from rendering

### 📋 renderer.rs (~1000 lines)
**Responsibilities:**
- Main render coordination
- Transcript rendering
- Header rendering
- Input rendering
- Queue overlay rendering

**Methods to extract:**
- `render()`
- `render_transcript()`
- `render_message_spans()`
- `render_tool_segments()`
- `render_tool_header_line()`
- Layout calculation methods

**Benefits:**
- Separates rendering from state management
- Easier to optimize rendering performance
- Clear rendering pipeline

### 📋 palette_renderer.rs (~300 lines)
**Responsibilities:**
- File palette rendering
- Prompt palette rendering
- Slash palette rendering
- Palette instructions rendering

**Methods to extract:**
- `render_file_palette()`
- `render_prompt_palette()`
- `file_palette_instructions()`
- `prompt_palette_instructions()`

**Benefits:**
- Consolidates palette rendering logic
- Reduces duplication
- Easier to add new palettes

### 📋 style_formatter.rs (~400 lines)
**Responsibilities:**
- Tool name normalization
- Tool styling (colors, borders)
- Format tool parameters
- Style calculations
- Theme application

**Methods to extract:**
- `normalize_tool_name()`
- `tool_inline_style()`
- `tool_border_style()`
- `format_tool_parameters()`
- `default_style()`
- `accent_inline_style()`
- `border_inline_style()`

**Benefits:**
- Centralizes styling logic
- Consistent tool appearance
- Easy to customize themes
- Reusable styling functions

### 📋 layout_calculator.rs (~200 lines)
**Responsibilities:**
- Calculate view dimensions
- Compute layout constraints
- Manage responsive sizing
- Handle resize events

**Methods to extract:**
- `apply_view_rows()`
- `force_view_rows()`
- `apply_transcript_rows()`
- `apply_transcript_width()`
- `recalculate_transcript_rows()`
- `header_reserved_rows()`
- `input_reserved_rows()`
- `input_block_height_for_lines()`

**Benefits:**
- Clear layout calculation logic
- Easier to implement responsive design
- Testable dimension calculations

## Migration Strategy

### Phase 1: Non-Breaking Additions ✅
1. ✅ Create state.rs with new state structs
2. Add new modules alongside existing code
3. Document the new architecture

### Phase 2: Gradual Migration (Future)
1. Update Session struct to use new state structs
2. Move methods to appropriate modules
3. Add tests for extracted modules
4. Update imports and visibility

### Phase 3: Cleanup (Future)
1. Remove duplicated code
2. Optimize module boundaries
3. Update documentation
4. Performance profiling

## Expected Improvements

### Metrics Before Refactoring
- Session struct: ~55 fields
- Session impl: ~170 methods
- session.rs: 4,762 lines
- Cognitive complexity: Very High

### Metrics After Refactoring (Target)
- Session struct: ~10-15 fields (state containers + managers)
- session.rs: ~500-800 lines (coordination only)
- Average module size: ~300 lines
- Cognitive complexity: Medium

### Benefits
1. **Maintainability:** Smaller, focused modules are easier to understand
2. **Testability:** Each module can be tested independently
3. **Extensibility:** New features can be added to appropriate modules
4. **Performance:** Easier to optimize specific areas
5. **Collaboration:** Multiple developers can work on different modules
6. **Documentation:** Each module has clear responsibility
7. **DRY Principle:** Reduces duplication (especially in styling/rendering)

## Implementation Notes

### Backward Compatibility
- All existing tests must pass
- Public API should remain unchanged
- Internal refactoring only

### Testing Strategy
- Unit tests for each new module
- Integration tests for Session coordination
- Property-based tests for layout calculations
- Regression tests for rendering

### Performance Considerations
- Rendering optimizations should be preserved
- Cache invalidation logic must be maintained
- No additional allocations in hot paths

## References
- Original analysis: Project-wide codebase refactoring analysis
- session.rs lines: 77-132 (state struct), 226-323 (command handling), 426-500+ (rendering)
- Existing good patterns: InputManager, ScrollManager (already extracted)
