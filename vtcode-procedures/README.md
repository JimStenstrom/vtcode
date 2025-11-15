# vtcode-procedures

Standard Operating Procedures system for VTCode agents.

## Overview

`vtcode-procedures` provides functionality for loading, indexing, and retrieving Standard Operating Procedures (SOPs) to guide LLM behavior on common workflows.

## Features

- **Markdown-based Procedures** - Write procedures in simple markdown with optional YAML frontmatter
- **Semantic Search** - Find relevant procedures using RAG (Retrieval-Augmented Generation)
- **Configurable Directories** - Load procedures from multiple locations
- **Project & User Procedures** - Support for both version-controlled and user-specific procedures

## Usage

```rust
use vtcode_procedures::ProcedureManager;
use vtcode_config::ProceduresConfig;

#[tokio::main]
async fn main() {
    let config = ProceduresConfig::default();
    let manager = ProcedureManager::new(config).await.unwrap();

    // Retrieve relevant procedures
    let procedures = manager.get_relevant_procedures("how to edit files", 3).await.unwrap();
    for proc in procedures {
        println!("{}", proc);
    }
}
```

## Procedure Format

Procedures are markdown files with optional YAML frontmatter:

```markdown
---
type: standard-operating-procedure
id: file-editing
---

# Effective File Editing

## When to Use

When you need to modify file contents...

## Best Practices

### 1. Read Before Edit

**Always** read a file before editing it...
```

## Configuration

Configure procedure directories in `vtcode.toml`:

```toml
[memory.procedures]
enabled = true
paths = [
    "docs/procedures",        # Project-level procedures (version controlled)
    ".vtcode/procedures"      # User-specific procedures (gitignored)
]
```

## Default Procedures

VTCode includes several built-in procedures:

- **file-reading.md** - Effective file reading strategies
- **file-editing.md** - File editing best practices
- **code-search.md** - How to search code effectively
- **error-handling.md** - Handling errors properly
- **git-commit.md** - Git commit procedures
- **pty-sessions.md** - PTY/terminal session management
- **test-before-commit.md** - Testing procedures

## Architecture

```
vtcode-procedures
├── loader.rs          # Load markdown procedures from directories
└── manager.rs         # Procedure indexing and retrieval via RAG
```

The procedure system uses:
- **vtcode-vectordb** - Vector storage for semantic search
- **vtcode-rag** - Document chunking and embedding
- **vtcode-config** - Configuration management

## Development

Run tests:

```bash
cargo test -p vtcode-procedures
```

## License

MIT
