# vtcode-execution

[![Crates.io](https://img.shields.io/crates/v/vtcode-execution.svg)](https://crates.io/crates/vtcode-execution)
[![Documentation](https://docs.rs/vtcode-execution/badge.svg)](https://docs.rs/vtcode-execution)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Code execution, sandbox management, and execution policy for VTCode.

## Overview

`vtcode-execution` provides the execution layer for VTCode, extracted as part of the Phase 2 architecture transformation. It handles:

- **Code Execution**: Run Python/JavaScript snippets with MCP tool integration
- **Async Commands**: Execute system commands with streaming output
- **Sandbox Management**: Configure secure execution environments
- **Execution Policy**: Validate commands against security allow-lists
- **Skills Management**: Save and reuse code functions across sessions
- **PII Detection**: Identify and tokenize sensitive information
- **Agent Optimization**: Analyze and improve agent behavior patterns
- **Tool Versioning**: Manage compatibility and migrations

## Architecture

This crate depends only on foundation layers, making it independently reusable:

```
vtcode-execution
    ├─→ vtcode-commons (utilities, safety validation)
    ├─→ vtcode-bash-runner (command execution)
    ├─→ vtcode-tool-traits (MCP tool executor trait)
    └─→ vtcode-exec-events (execution event types)
```

## Features

### Code Execution with MCP Integration

Execute code that can call MCP tools as library functions, reducing round-trips to the LLM:

```rust
use vtcode_execution::{CodeExecutor, Language};
use std::sync::Arc;
use std::path::PathBuf;

let executor = CodeExecutor::new(
    Language::Python3,
    sandbox_profile,
    Arc::new(mcp_client),
    PathBuf::from("/workspace"),
);

let code = r#"
# Call MCP tools directly in code
files = list_files(path="/workspace", recursive=True)
filtered = [f for f in files if "test" in f]
result = {"count": len(filtered), "files": filtered[:10]}
"#;

let result = executor.execute(code).await?;
```

### Sandbox Environment Management

Configure sandboxed execution with fine-grained permissions:

```rust
use vtcode_execution::sandbox::{SandboxEnvironment, SandboxRuntimeKind};

let mut environment = SandboxEnvironment::builder("./workspace")
    .sandbox_root("./.vtcode/sandbox")
    .runtime_kind(SandboxRuntimeKind::AnthropicSrt)
    .build();

// Configure permissions
environment.allow_domain("api.example.com")?;
environment.allow_path("logs")?;
environment.deny_path("secrets")?;

// Write settings for sandbox runtime
environment.write_settings()?;

// Create profile for execution
let profile = environment.create_profile("/usr/local/bin/srt");
```

### Execution Policy Validation

Enforce security policies on command execution:

```rust
use vtcode_execution::policy::{validate_command, sanitize_working_dir};
use std::path::Path;

let workspace = Path::new("/workspace");
let working_dir = sanitize_working_dir(workspace, Some("./project")).await?;

let command = vec!["git".to_string(), "status".to_string()];
validate_command(&command, workspace, &working_dir).await?;

// Only allowed commands pass validation
// Prevents workspace breakout and destructive operations
```

### Async Process Execution

Stream output from long-running processes:

```rust
use vtcode_execution::exec::async_command::{AsyncProcessRunner, ProcessOptions};

let options = ProcessOptions {
    command: vec!["cargo".to_string(), "build".to_string()],
    working_dir: Some("/workspace".into()),
    timeout: Some(Duration::from_secs(300)),
    ..Default::default()
};

let mut runner = AsyncProcessRunner::new(options);
let mut output_stream = runner.spawn().await?;

while let Some(output) = output_stream.next().await {
    println!("Output: {}", output);
}
```

### Skills Management

Save and reuse code functions:

```rust
use vtcode_execution::exec::{SkillManager, SkillMetadata};

let manager = SkillManager::new("/workspace/.vtcode/skills");

// Save a skill
let metadata = SkillMetadata {
    name: "analyze_logs".to_string(),
    description: "Parse and analyze application logs".to_string(),
    language: "python3".to_string(),
    // ... other metadata
};

manager.save_skill(metadata, code_content).await?;

// Load and use later
let skill = manager.load_skill("analyze_logs").await?;
```

### PII Detection

Detect and tokenize personally identifiable information:

```rust
use vtcode_execution::exec::{PiiTokenizer, PiiType};

let tokenizer = PiiTokenizer::new();
let text = "Contact John at john.doe@example.com or call 555-123-4567";

let detected = tokenizer.detect(text);
for pii in detected {
    match pii.pii_type {
        PiiType::Email => println!("Found email: {}", pii.value),
        PiiType::Phone => println!("Found phone: {}", pii.value),
        _ => {}
    }
}

// Replace with tokens
let sanitized = tokenizer.tokenize(text);
```

### Agent Behavior Analysis

Analyze and optimize agent patterns:

```rust
use vtcode_execution::exec::{AgentBehaviorAnalyzer, ToolStatistics};

let mut analyzer = AgentBehaviorAnalyzer::new();

// Track tool usage
analyzer.record_tool_call("read_file", true, Duration::from_millis(50));
analyzer.record_tool_call("write_file", false, Duration::from_millis(100));

// Analyze patterns
let stats = analyzer.get_tool_statistics();
let failures = analyzer.detect_failure_patterns();
let recommendations = analyzer.suggest_optimizations();
```

## Module Overview

### `exec` Module

The execution module contains:

- **`code_executor`**: Execute code with MCP tool integration
- **`async_command`**: Async process execution with streaming
- **`skill_manager`**: Persistent skill storage and retrieval
- **`pii_tokenizer`**: PII detection and tokenization
- **`agent_optimization`**: Agent behavior analysis
- **`tool_versioning`**: Tool compatibility checking
- **`sdk_ipc`**: IPC for tool communication
- **`cancellation`**: Cancellation token support

### `sandbox` Module

Sandbox environment management:

- **`environment`**: Builder and configuration
- **`profile`**: Runtime profile generation
- **`settings`**: Permission and network settings

### `policy` Module

Execution policy validation:

- Command allow-list enforcement
- Argument validation
- Workspace boundary checks
- Path traversal prevention

## Use Cases

### 1. AI Agent Frameworks

Use vtcode-execution to add secure code execution to your agent:

```rust
// Your agent can execute code safely
let executor = CodeExecutor::new(language, sandbox, mcp, workspace);
let result = executor.execute(agent_generated_code).await?;
```

### 2. Code Analysis Tools

Leverage sandbox management for safe code analysis:

```rust
let mut env = SandboxEnvironment::builder(workspace).build();
env.allow_domain("github.com")?;
let profile = env.create_profile(runtime_path);
```

### 3. Security Research

Use execution policy validation in security tools:

```rust
// Validate commands before execution
validate_command(&cmd, workspace, working_dir).await?;
```

### 4. Educational Platforms

Execute student code safely with sandboxing:

```rust
let executor = CodeExecutor::new(
    Language::Python3,
    student_sandbox,
    tool_client,
    workspace,
);
```

## Testing

Run tests with:

```bash
cargo test -p vtcode-execution
```

Test with specific features:

```bash
cargo test -p vtcode-execution --all-features
```

## Dependencies

This crate has minimal dependencies on other vtcode crates:

- `vtcode-commons`: Foundation utilities and path validation
- `vtcode-bash-runner`: Bash command execution primitives
- `vtcode-tool-traits`: MCP tool executor trait definitions
- `vtcode-exec-events`: Execution event type definitions

External dependencies include standard async runtime (tokio), serialization (serde), and logging (tracing).

## Contribution

This crate was extracted from `vtcode-core` as part of the Phase 2 architecture transformation. When contributing:

1. Keep dependencies minimal (foundation layer only)
2. Add tests for new functionality
3. Document public APIs thoroughly
4. Follow the existing code structure

## License

MIT License - see LICENSE file for details.

## Related Crates

- [`vtcode-core`](../vtcode-core): Main VTCode library (re-exports this crate)
- [`vtcode-commons`](../vtcode-commons): Shared utilities
- [`vtcode-tool-traits`](../vtcode-tool-traits): Tool system traits
- [`vtcode-bash-runner`](../vtcode-bash-runner): Command execution

## References

- [VTCode Repository](https://github.com/vinhnx/vtcode)
- [Architecture Transformation Plan](../ARCHITECTURE_TRANSFORMATION.md)
- [Phase 2 Completion Summary](./PHASE2_VTCODE_EXECUTION_COMPLETE.md)
