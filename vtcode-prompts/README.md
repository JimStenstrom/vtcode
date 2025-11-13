# vtcode-prompts

**Prompt generation and management system for VTCode**

`vtcode-prompts` is a standalone crate that provides flexible, reusable prompt generation infrastructure for AI coding agents. It's designed to be provider-agnostic, returning raw prompt strings that can be used with any LLM provider.

## Features

- 🎯 **System Prompts** - Default, lightweight, and specialized system instructions
- 🔧 **Custom Prompts** - User-defined prompt templates with variable substitution
- 📚 **Built-in Documentation** - Embedded documentation for self-documentation queries
- ⚙️ **Flexible Configuration** - Rich configuration options for prompt customization
- 🔌 **Provider-Agnostic** - Works with OpenAI, Anthropic, Gemini, or any LLM provider
- 🧪 **Well-Tested** - Comprehensive test coverage for reliability

## Quick Start

```rust
use vtcode_prompts::{SystemPromptConfig, generate_system_instruction};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Generate a system instruction with default settings
    let config = SystemPromptConfig::default();
    let instruction = generate_system_instruction(&config).await;

    println!("{}", instruction);
    Ok(())
}
```

## Architecture

The crate is organized into focused modules:

```
vtcode-prompts/
├── config      - Configuration types (SystemPromptConfig, AgentPersonality, ResponseStyle)
├── context     - Context information (PromptContext, UserPreferences)
├── templates   - Reusable prompt templates
├── system      - System instruction generation (default, lightweight, specialized)
├── custom      - Custom prompt registry with variable substitution
└── generator   - High-level prompt generation interface
```

## System Prompts

vtcode-prompts provides three built-in system prompt variants:

### Default System Prompt

The full-featured system prompt for general-purpose coding tasks:

```rust
use vtcode_prompts::{SystemPromptConfig, generate_system_instruction};

let config = SystemPromptConfig::default();
let instruction = generate_system_instruction(&config).await;
```

### Lightweight Prompt

A concise prompt for simple operations with lower token usage:

```rust
use vtcode_prompts::generate_lightweight_instruction;

let instruction = generate_lightweight_instruction();
```

### Specialized Prompt

An advanced prompt for complex refactoring and multi-file changes:

```rust
use vtcode_prompts::generate_specialized_instruction;

let instruction = generate_specialized_instruction();
```

## Custom Prompts

Custom prompts allow users to define reusable templates with variable substitution:

### Creating a Custom Prompt

Create a markdown file `~/.vtcode/prompts/review.md`:

```markdown
---
description: Review code for issues
argument_hint: FILE=<path> FOCUS=<aspect>
---
Please review $FILE and focus on $FOCUS.

Additional context: $ARGUMENTS
```

### Using Custom Prompts

```rust
use vtcode_prompts::{CustomPromptRegistry, PromptInvocation};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load custom prompts from directories
    let registry = CustomPromptRegistry::load(None, Path::new(".")).await?;

    // Get a specific prompt
    if let Some(prompt) = registry.get("review") {
        // Parse invocation arguments
        let invocation = PromptInvocation::parse("security FILE=auth.rs FOCUS=vulnerabilities")?;

        // Expand the prompt with arguments
        let expanded = prompt.expand(&invocation)?;
        println!("{}", expanded);
    }

    Ok(())
}
```

### Variable Substitution

Custom prompts support several types of placeholders:

- `$1`, `$2`, ... - Positional arguments
- `$NAME` - Named arguments (e.g., `FILE=path`)
- `$ARGUMENTS` - All positional arguments joined
- `$TASK` - Automatically populated with all arguments
- `$$` - Literal dollar sign

## Prompt Context

Customize prompts with contextual information:

```rust
use vtcode_prompts::{PromptContext, SystemPromptConfig, generate_system_instruction_with_config};
use std::path::PathBuf;

let mut context = PromptContext::from_workspace(PathBuf::from("/workspace"));
context.add_language("Rust".to_string());
context.add_language("Python".to_string());
context.add_tool("read_file".to_string());
context.add_tool("grep_file".to_string());
context.set_project_type("Backend API".to_string());

let config = SystemPromptConfig::default();
let instruction = generate_system_instruction_with_config(&config, &context);
```

## Configuration

Customize prompt generation behavior:

```rust
use vtcode_prompts::{SystemPromptConfig, AgentPersonality, ResponseStyle};

let config = SystemPromptConfig {
    verbose: false,
    include_tools: true,
    include_workspace: true,
    custom_instruction: Some("Always explain your reasoning.".to_string()),
    personality: AgentPersonality::Technical,
    response_style: ResponseStyle::Detailed,
};
```

### Personality Options

- `Professional` - Focused, business-like approach
- `Friendly` - Encouraging and approachable
- `Technical` - Detailed technical explanations
- `Creative` - Innovative solutions and ideas

### Response Style Options

- `Concise` - Brief, to-the-point responses
- `Detailed` - Comprehensive explanations
- `Conversational` - Natural, easy-to-understand tone
- `Technical` - Precise technical language

## Built-in Documentation

Access embedded documentation for self-referencing:

```rust
use vtcode_prompts::BuiltinDocs;

let docs = BuiltinDocs::default();

// Check if documentation exists
if docs.contains("vtcode_docs_map") {
    // Get the documentation content
    let content = docs.get("vtcode_docs_map").unwrap();
    println!("{}", content);
}

// List all available documentation
for key in docs.keys() {
    println!("Available doc: {}", key);
}
```

## Integration with LLM Providers

Since vtcode-prompts returns raw strings, it's easy to integrate with any provider:

### OpenAI

```rust
let instruction = generate_system_instruction(&config).await;

// Use with OpenAI
let messages = vec![
    ChatMessage {
        role: "system".to_string(),
        content: instruction,
    },
    ChatMessage {
        role: "user".to_string(),
        content: "Hello!".to_string(),
    },
];
```

### Anthropic

```rust
let instruction = generate_system_instruction(&config).await;

// Use with Anthropic
let request = ClaudeRequest {
    system: Some(instruction),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: "Hello!".to_string(),
        },
    ],
    // ...
};
```

### Gemini

```rust
let instruction = generate_system_instruction(&config).await;

// Use with Gemini (via system instruction field)
let request = GeminiRequest {
    system_instruction: Some(SystemInstruction {
        parts: vec![Part::Text { text: instruction }],
    }),
    contents: vec![
        Content::user_text("Hello!"),
    ],
    // ...
};
```

## Testing

The crate includes comprehensive tests:

```bash
# Run all tests
cargo test -p vtcode-prompts

# Run specific test suite
cargo test -p vtcode-prompts --test integration_tests

# Run with output
cargo test -p vtcode-prompts -- --nocapture
```

## Design Principles

1. **Provider Agnostic** - Returns raw strings, not provider-specific types
2. **Minimal Dependencies** - Only essential dependencies included
3. **No Circular Dependencies** - Clean dependency graph
4. **Reusable** - Can be used in any Rust project needing prompt management
5. **Well-Tested** - High test coverage for reliability
6. **Documented** - Comprehensive documentation and examples

## Dependencies

- `anyhow` - Error handling
- `serde`, `serde_json`, `serde_yaml` - Serialization
- `tokio` - Async runtime
- `tracing` - Logging
- `dirs` - Home directory detection
- `shell-words` - Shell-style argument parsing
- `vtcode-commons` - Common utilities
- `vtcode-config` - Configuration types

## Use Cases

- **AI Coding Agents** - Provide system instructions for code-focused LLMs
- **Prompt Management** - Organize and reuse prompt templates
- **Multi-Provider Systems** - Generate prompts that work across providers
- **Custom Workflows** - Create domain-specific prompt variations
- **Testing** - Generate consistent prompts for testing LLM integrations

## Contributing

This crate is part of the VTCode project. Contributions are welcome!

## License

MIT License - See LICENSE file for details

## Related Crates

- `vtcode-core` - Main VTCode runtime (uses vtcode-prompts)
- `vtcode-config` - Configuration management
- `vtcode-commons` - Common utilities
- `vtcode-llm` - LLM provider integrations
