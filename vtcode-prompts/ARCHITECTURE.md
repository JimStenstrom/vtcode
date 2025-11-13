# vtcode-prompts Architecture

## Overview

`vtcode-prompts` is a standalone crate that provides prompt generation and management infrastructure for AI coding agents. It follows a modular, layered architecture with clear separation of concerns.

## Design Principles

1. **Provider Agnostic** - Returns raw strings, not provider-specific types
2. **Minimal Dependencies** - Only essential crates included
3. **No Circular Dependencies** - Clean, acyclic dependency graph
4. **Reusable** - Can be used in any Rust project
5. **Well-Tested** - High test coverage for reliability
6. **Documented** - Comprehensive documentation and examples

## Module Structure

```
vtcode-prompts/
├── src/
│   ├── lib.rs           - Main exports and documentation
│   ├── config.rs        - Configuration types
│   ├── context.rs       - Context information
│   ├── templates.rs     - Reusable templates
│   ├── system.rs        - System instruction generation
│   ├── custom.rs        - Custom prompt registry
│   └── generator.rs     - High-level generation interface
├── examples/
│   ├── basic_usage.rs       - Basic prompt generation
│   ├── custom_prompts.rs    - Custom prompt templates
│   └── configuration.rs     - Configuration options
├── tests/
│   └── integration_tests.rs - Integration tests
└── README.md            - User documentation
```

## Component Responsibilities

### config.rs
- Defines `SystemPromptConfig` for prompt generation settings
- Defines `AgentPersonality` enum for personality variants
- Defines `ResponseStyle` enum for response style variants
- Provides sensible defaults

**Key Types:**
- `SystemPromptConfig` - Main configuration struct
- `AgentPersonality` - Professional, Friendly, Technical, Creative
- `ResponseStyle` - Concise, Detailed, Conversational, Technical

### context.rs
- Defines `PromptContext` for contextual information
- Defines `UserPreferences` for user customization
- Provides builder-style API for context construction

**Key Types:**
- `PromptContext` - Workspace, languages, tools, project type
- `UserPreferences` - User-specific preferences

### templates.rs
- Provides reusable prompt template fragments
- Static string templates for common patterns
- Personality and response style templates

**Key Functions:**
- `base_system_prompt()` - Core system prompt
- `personality_prompt()` - Personality-specific additions
- `response_style_prompt()` - Style-specific additions
- `tool_usage_prompt()` - Tool usage guidelines
- `workspace_context_prompt()` - Workspace-aware prompts
- `safety_guidelines_prompt()` - Safety instructions

### system.rs
- Core system instruction generation
- Three prompt variants: default, lightweight, specialized
- File-based prompt loading with fallback
- Provider-agnostic output

**Key Functions:**
- `default_system_prompt()` - Returns default prompt string
- `generate_system_instruction()` - Async generation from config
- `generate_lightweight_instruction()` - Concise variant
- `generate_specialized_instruction()` - Advanced variant
- `read_system_prompt_from_md()` - Load from file
- `compose_system_instruction_text()` - Compose full instruction

### custom.rs
- Custom prompt template system
- Variable substitution ($1, $NAME, $ARGUMENTS)
- YAML frontmatter support
- Built-in prompt registry
- File-based prompt loading

**Key Types:**
- `CustomPromptRegistry` - Registry of custom prompts
- `CustomPrompt` - Individual prompt template
- `PromptInvocation` - Parsed invocation with arguments
- `BuiltinDocs` - Embedded documentation registry

**Variable Syntax:**
- `$1`, `$2`, ... - Positional arguments
- `$NAME` - Named arguments
- `$ARGUMENTS` - All positional arguments
- `$TASK` - Auto-populated with all arguments
- `$$` - Escape for literal $

### generator.rs
- High-level prompt generation interface
- Combines configuration and context
- Composable prompt generation

**Key Types:**
- `SystemPromptGenerator` - Stateful generator
- `generate_system_instruction_with_config()` - Helper function

## Data Flow

### System Prompt Generation

```
┌─────────────────────────────────────────────┐
│ User Application                             │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ SystemPromptConfig + PromptContext          │
│ - Personality, ResponseStyle                │
│ - Languages, Tools, Workspace               │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ SystemPromptGenerator                       │
│ - Combines templates                        │
│ - Applies configuration                     │
│ - Injects context                           │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ Raw String Output                           │
│ (Provider-agnostic)                         │
└─────────────────────────────────────────────┘
```

### Custom Prompt Expansion

```
┌─────────────────────────────────────────────┐
│ Custom Prompt File (.md)                    │
│ - YAML frontmatter (optional)               │
│ - Template body with variables              │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ CustomPromptRegistry::load()                │
│ - Scans directories                         │
│ - Parses frontmatter                        │
│ - Extracts variables                        │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ PromptInvocation::parse()                   │
│ - Shell-style argument parsing              │
│ - Named and positional args                 │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ CustomPrompt::expand()                      │
│ - Substitute variables                      │
│ - Validate required args                    │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ Expanded Prompt String                      │
└─────────────────────────────────────────────┘
```

## Dependencies

### Direct Dependencies
- `anyhow` - Error handling
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tokio` - Async runtime (for file I/O)
- `tracing` - Logging
- `dirs` - Home directory detection
- `shell-words` - Shell-style argument parsing

### Workspace Dependencies
- `vtcode-commons` - Common utilities
- `vtcode-config` - Configuration types

### Why These Dependencies?

- **anyhow**: Provides ergonomic error handling without boilerplate
- **serde**: Industry-standard serialization, needed for config
- **tokio**: Used only for async file I/O, minimal feature set
- **tracing**: Standard logging, integrates with parent applications
- **dirs**: Cross-platform home directory detection
- **shell-words**: Proper shell-style quoting for arguments
- **vtcode-commons**: Shared utilities to avoid duplication
- **vtcode-config**: Configuration types for consistency

## Integration Patterns

### With vtcode-core

vtcode-core uses vtcode-prompts as a library and adds core-specific extensions:

```rust
// In vtcode-core/src/prompts/mod.rs
pub use vtcode_prompts::*;  // Re-export all types

pub mod core_extensions;  // Core-specific additions:
                          // - Gemini Content integration
                          // - AGENTS.md instructions
                          // - Config awareness
```

This pattern allows:
- Clean separation of generic and specific functionality
- No changes to vtcode-prompts for core-specific features
- Backward compatibility through re-exports

### With Other Projects

Other projects can use vtcode-prompts directly:

```rust
use vtcode_prompts::{
    SystemPromptConfig,
    generate_system_instruction,
};

// Generate prompts without vtcode-core dependency
```

## Testing Strategy

### Unit Tests
- Located inline with source files (`#[cfg(test)] mod tests`)
- Test individual functions and types in isolation
- Fast, no I/O or external dependencies

### Integration Tests
- Located in `tests/` directory
- Test complete workflows end-to-end
- Include file I/O and async operations
- Verify real-world usage patterns

### Examples as Tests
- Examples in `examples/` directory serve dual purpose
- Demonstrate usage patterns
- Act as smoke tests for basic functionality
- Can be run with `cargo run --example <name>`

## Future Enhancements

### Planned Features
1. **Build-time Embedding** - Embed built-in prompts at compile time
2. **Prompt Versioning** - Track and migrate prompt versions
3. **Template Validation** - Validate template syntax at load time
4. **Performance** - Optimize for large prompt registries
5. **More Templates** - Additional built-in prompt templates

### Non-Goals
- Provider-specific formatting (kept in provider crates)
- Complex template engines (keep it simple)
- Runtime prompt compilation
- Network-based prompt fetching

## Security Considerations

### File Size Limits
- Custom prompts have configurable size limits
- Default 64KB prevents memory exhaustion
- Validates file size before reading content

### Path Safety
- All paths validated before file operations
- Uses tokio's safe file I/O
- No arbitrary path traversal

### Input Validation
- Prompt names validated (no whitespace, no colons)
- Shell argument parsing uses battle-tested `shell-words`
- YAML parsing errors handled gracefully

### No Code Execution
- Templates are pure text substitution
- No eval, no scripting, no code execution
- Variables are simple string replacement

## Performance Characteristics

### Memory Usage
- Prompts loaded on-demand
- Registry uses BTreeMap for O(log n) lookup
- Deduplication prevents multiple copies
- File size limits prevent unbounded growth

### CPU Usage
- Parsing is linear in prompt size
- Variable substitution is linear in template size
- No complex regex or parsing
- Caching at call site recommended for hot paths

### I/O Patterns
- Async file I/O with tokio
- Reads entire file at once (limited by size check)
- No buffering needed for small files
- Directory scanning is one-time at load

## Versioning and Compatibility

### Semantic Versioning
- Major: Breaking API changes
- Minor: New features, backward compatible
- Patch: Bug fixes, no API changes

### Stability Guarantees
- Public API is stable within major version
- Internal implementation may change
- Examples kept up-to-date with API

### Migration Path
- Breaking changes documented in CHANGELOG
- Migration guide provided for major versions
- Deprecation warnings before removal
