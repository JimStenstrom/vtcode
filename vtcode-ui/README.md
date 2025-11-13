# vtcode-ui

**Terminal UI components for VTCode**

[![Crate](https://img.shields.io/badge/crates.io-vtcode--ui-orange)](https://crates.io/crates/vtcode-ui)
[![Documentation](https://img.shields.io/badge/docs-vtcode--ui-blue)](https://docs.rs/vtcode-ui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🚧 Work In Progress

**Status**: Phase 2 Extraction - Foundation Complete

This crate is being extracted from `vtcode-core` as part of Phase 2 of the VTCode architecture transformation. The extraction is in progress and the crate is not yet ready for independent use.

**Current State**:
- ✅ Crate structure created
- ✅ UI code migrated (~19K LOC, 42 files)
- ✅ Dependencies analyzed and documented
- 🔄 Import refactoring in progress
- ⏳ Integration pending

See [PHASE_2_VTCODE_UI_PROGRESS.md](../PHASE_2_VTCODE_UI_PROGRESS.md) for detailed status.

---

## Overview

`vtcode-ui` provides terminal user interface components for VTCode, a Rust-based AI coding agent. It includes:

- **TUI Framework**: Full-featured terminal UI built with `ratatui`
- **Theme System**: Customizable themes with Catppuccin support
- **Markdown Rendering**: Syntax-highlighted markdown with code blocks
- **Diff Rendering**: Git-style diff visualization
- **Search Components**: Fuzzy search with nucleo-matcher
- **File Browser**: Interactive file selection palette
- **Slash Commands**: Command palette UI
- **Session Management**: Conversation UI with message history

---

## Architecture

### Module Structure

```
vtcode-ui/
├── src/
│   ├── lib.rs                   # Public API exports
│   │
│   ├── Core Rendering
│   ├── theme.rs                 # Theme system
│   ├── theme_config.rs          # Theme configuration
│   ├── theme_manager.rs         # Theme management
│   ├── markdown.rs              # Markdown rendering
│   ├── diff_renderer.rs         # Diff visualization
│   ├── styled.rs                # Styled text utilities
│   │
│   ├── Utilities
│   ├── search.rs                # Fuzzy search
│   ├── slash.rs                 # Slash command helpers
│   ├── file_colorizer.rs        # File syntax coloring
│   ├── git_config.rs            # Git color config
│   ├── terminal.rs              # Terminal utilities
│   ├── user_confirmation.rs     # User prompts
│   │
│   └── tui/                     # Terminal UI (main interface)
│       ├── modern_tui.rs        # Modern TUI implementation
│       ├── session.rs           # Session management
│       ├── types.rs             # TUI type definitions
│       ├── style.rs             # Style system
│       ├── theme_parser.rs      # Theme parsing
│       │
│       └── session/             # Session components
│           ├── state.rs         # Session state
│           ├── message.rs       # Message rendering
│           ├── input.rs         # Input handling
│           ├── header.rs        # Header rendering
│           ├── modal.rs         # Modal dialogs
│           ├── file_palette.rs  # File picker
│           ├── prompt_palette.rs # Prompt picker
│           ├── slash_palette.rs # Slash command picker
│           ├── transcript.rs    # Message transcript
│           ├── scroll.rs        # Scroll management
│           ├── navigation.rs    # Keyboard navigation
│           └── ...              # Additional components
```

### Key Components

#### 1. Theme System
Provides dynamic theming with contrast adjustment:
- Base themes: Catppuccin variants (Mocha, Macchiato, Frappé, Latte)
- Custom theme support via configuration
- Automatic contrast adjustment for readability
- ANSI style generation

#### 2. TUI Session
Full-featured terminal interface:
- Message history with syntax highlighting
- Real-time streaming support
- Keyboard navigation (vim-style and arrow keys)
- Modal dialogs and overlays
- File/prompt/slash command palettes
- Progress indicators

#### 3. Markdown Renderer
Sophisticated markdown rendering:
- Syntax highlighting via `syntect`
- Code fence support with language detection
- Inline formatting (bold, italic, code)
- Heading hierarchy
- List rendering

#### 4. Search & Palettes
Interactive selection components:
- Fuzzy matching with `nucleo-matcher`
- File system browsing
- Custom prompt selection
- Slash command suggestions

---

## Dependencies

### Core UI
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal control
- **anstyle** suite - ANSI styling

### Rendering
- **syntect** - Syntax highlighting
- **pulldown-cmark** - Markdown parsing
- **catppuccin** - Theme palettes

### Search & Matching
- **perg** - Pattern matching
- **nucleo-matcher** - Fuzzy matching

### Utilities
- **unicode-width** - Unicode string width calculation
- **unicode-segmentation** - Grapheme cluster handling
- **textwrap** - Text wrapping

See [Cargo.toml](./Cargo.toml) for complete dependency list.

---

## Usage

> **Note**: vtcode-ui is currently being refactored and is not yet ready for standalone use. The API is subject to change.

### Basic Example (Planned API)

```rust
use vtcode_ui::{ThemeManager, Session};

// Create theme manager
let theme_manager = ThemeManager::new();
theme_manager.set_theme("catppuccin-mocha")?;

// Create TUI session
let session = Session::new()?;
session.run()?;
```

### Rendering Markdown

```rust
use vtcode_ui::render_markdown;

let markdown = "# Hello\n\nThis is **bold** and this is *italic*.";
let rendered = render_markdown(markdown);
println!("{}", rendered);
```

### Theme Customization

```rust
use vtcode_ui::{ThemeManager, ThemePalette};
use anstyle::RgbColor;

let palette = ThemePalette {
    primary_accent: RgbColor(137, 220, 235),   // Sky blue
    background: RgbColor(24, 24, 37),          // Dark background
    foreground: RgbColor(205, 214, 244),       // Light text
    secondary_accent: RgbColor(203, 166, 247), // Purple
    alert: RgbColor(243, 139, 168),            // Pink
    logo_accent: RgbColor(166, 227, 161),      // Green
};

let theme_manager = ThemeManager::new();
theme_manager.set_custom_palette(palette);
```

---

## Integration Status

### Current Dependencies (Being Resolved)

vtcode-ui currently depends on vtcode-core for:

1. **Configuration** (`crate::config::*`)
   - Constants (UI layout values)
   - Types (UiSurfacePreference, ReasoningEffortLevel)
   - Loaders (SyntaxHighlightingConfig)

2. **Prompts** (`crate::prompts::*`)
   - CustomPrompt type
   - CustomPromptRegistry

3. **Tools** (`crate::tools::*`)
   - TaskPlan type
   - PlanSummary type

4. **Utils** (`crate::utils::*`)
   - Style parsing utilities
   - Color helpers

**Resolution Plan**: See [DEPENDENCY_ANALYSIS.md](./DEPENDENCY_ANALYSIS.md) for complete strategy.

### Target State (Post-Refactoring)

```rust
// vtcode-ui will depend only on:
vtcode-commons     // Shared types and constants
vtcode-tool-traits // Tool-related types (TaskPlan, etc.)
vtcode-config      // Configuration (optional)
```

---

## Development

### Building

```bash
# From repository root
cargo build -p vtcode-ui

# With all features
cargo build -p vtcode-ui --all-features
```

### Testing

```bash
# Run tests
cargo test -p vtcode-ui

# Run with output
cargo test -p vtcode-ui -- --nocapture
```

### Documentation

```bash
# Generate and open docs
cargo doc -p vtcode-ui --open
```

---

## Features

Currently, vtcode-ui does not have feature flags. All functionality is included by default.

**Planned Features** (post-extraction):
- `themes` - Theme system (default)
- `tui` - Terminal UI components (default)
- `markdown` - Markdown rendering (default)
- `search` - Search components (default)

---

## Contributing

vtcode-ui is part of the VTCode project. Contributions are welcome!

**Phase 2 Status**: We are currently in the extraction phase. If you'd like to contribute:

1. Check [PHASE_2_VTCODE_UI_PROGRESS.md](../PHASE_2_VTCODE_UI_PROGRESS.md) for status
2. Review [DEPENDENCY_ANALYSIS.md](./DEPENDENCY_ANALYSIS.md) for integration plan
3. See open issues on GitHub

---

## Documentation

### Primary Documents
- [README.md](./README.md) - This file
- [DEPENDENCY_ANALYSIS.md](./DEPENDENCY_ANALYSIS.md) - Detailed dependency analysis
- [DEPENDENCY_MAP.md](./DEPENDENCY_MAP.md) - Visual dependency reference
- [PHASE_2_VTCODE_UI_PROGRESS.md](../PHASE_2_VTCODE_UI_PROGRESS.md) - Extraction progress

### Code Documentation
- Module docs: See individual module headers
- API docs: Generated via `cargo doc`

---

## Architecture Principles

vtcode-ui follows these design principles:

1. **Modularity**: Components are self-contained and composable
2. **Flexibility**: Support multiple UI modes (terminal, modern TUI)
3. **Performance**: Efficient rendering with minimal redraws
4. **Accessibility**: Clear visual hierarchy and keyboard navigation
5. **Themability**: All colors configurable via theme system

---

## Performance Considerations

### Rendering Optimization
- Incremental rendering (only changed regions)
- Cached style parsing
- Lazy message reflow
- Viewport culling

### Memory Management
- Message transcript with LRU cache
- Syntax highlighting cache
- Theme instance reuse

---

## Compatibility

### Terminal Requirements
- UTF-8 support
- 256-color or truecolor support (recommended)
- Minimum 80x24 terminal size

### Platform Support
- ✅ Linux
- ✅ macOS
- ✅ Windows (via crossterm)
- ✅ WSL

---

## Related Crates

Part of the VTCode ecosystem:

- **vtcode-core** - Core agent logic
- **vtcode-commons** - Shared utilities and types
- **vtcode-config** - Configuration management
- **vtcode-tool-traits** - Tool system traits
- **vtcode-llm** - LLM client implementations
- **vtcode-tools** - Tool implementations

---

## License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

---

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Themes from [Catppuccin](https://github.com/catppuccin/catppuccin)
- Syntax highlighting via [syntect](https://github.com/trishume/syntect)
- Fuzzy matching via [nucleo-matcher](https://github.com/helix-editor/nucleo)

---

## Status & Roadmap

### Current Phase: Extraction (Phase 2)
- [x] Create crate structure
- [x] Migrate code from vtcode-core
- [x] Analyze dependencies
- [ ] Resolve external dependencies
- [ ] Update imports
- [ ] Integration testing
- [ ] Independent compilation

### Future: Enhancement (Phase 3+)
- [ ] Web UI support (wasm)
- [ ] Plugin system for custom components
- [ ] Alternative theme formats
- [ ] Improved accessibility
- [ ] Performance benchmarks

---

## Contact & Support

- **Repository**: https://github.com/vinhnx/vtcode
- **Issues**: https://github.com/vinhnx/vtcode/issues
- **Documentation**: https://docs.rs/vtcode-ui (coming soon)

---

**Version**: 0.43.6 (Pre-release)
**Status**: 🚧 Work In Progress
**Last Updated**: November 13, 2025
