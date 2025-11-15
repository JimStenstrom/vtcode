# Error Handling Standards

**Status**: Active
**Last Updated**: 2025-11-15
**Applies To**: vtcode-core and all dependent crates

## Overview

vtcode-core uses `anyhow::Result` for error handling with structured error types and consistent patterns. This document establishes the standards for error handling across the codebase.

## Quick Reference

✅ **DO**:
- Always use `?` operator with context
- Add rich error context using extension traits
- Log recoverable failures before using fallbacks
- Use custom error types for domain-specific errors
- Test error paths thoroughly

❌ **DON'T**:
- Never use `.unwrap()` in production code
- Never use `.expect()` without clear justification
- Never swallow errors silently
- Never use error messages without context

## Error Type Hierarchy

```
VTCodeError (top-level error type)
├── UserError          - Invalid input, wrong arguments
├── SystemError        - I/O, network, OS errors
├── ConfigError        - Configuration issues
├── ToolError          - Tool execution failures
├── ProviderError      - LLM provider errors
└── Internal           - Should never happen (bugs)
```

### When to Use Each Error Type

**UserError**: Problems with user input or arguments
```rust
use vtcode_core::errors::{UserError, VTCodeError};

if !path.exists() {
    return Err(UserError::FileNotFound(path.display().to_string()).into());
}
```

**SystemError**: OS, network, or I/O failures
```rust
use vtcode_core::errors::{SystemError, VTCodeError};

match tokio::time::timeout(duration, operation).await {
    Err(_) => Err(SystemError::Timeout(format!("Operation timed out after {:?}", duration)).into()),
    Ok(result) => result,
}
```

**ConfigError**: Configuration missing, invalid, or unparseable
```rust
use vtcode_core::errors::{ConfigError, VTCodeError};

let api_key = config.api_key
    .ok_or_else(|| ConfigError::Missing("api_key".to_string()))?;
```

**ToolError**: Tool not found, execution failed, invalid arguments
```rust
use vtcode_core::errors::{ToolError, VTCodeError};

if !self.tools.contains_key(name) {
    return Err(ToolError::NotFound(name.to_string()).into());
}
```

**ProviderError**: LLM provider issues (rate limits, invalid keys, etc.)
```rust
use vtcode_core::errors::{ProviderError, VTCodeError};

if response.status() == 429 {
    return Err(ProviderError::RateLimit.into());
}
```

## Error Context Patterns

### File Operations

```rust
use anyhow::Result;
use vtcode_core::errors::context::FileErrorExt;

// ❌ BAD - no context
let content = fs::read_to_string(&path)?;

// ✅ GOOD - with extension trait
let content = fs::read_to_string(&path)
    .with_file_read_context(&path)?;

// ✅ ALSO GOOD - with custom context
let content = fs::read_to_string(&config_path)
    .with_context(|| format!("Failed to load config for project: {}", project_name))?;
```

### Command Execution

```rust
use vtcode_core::errors::context::CommandErrorExt;

// ❌ BAD
let output = Command::new("git").arg("status").output()?;

// ✅ GOOD
let output = Command::new("git")
    .arg("status")
    .output()
    .with_command_context("git status")?;
```

### Tool Execution

```rust
use vtcode_core::errors::context::ToolErrorExt;

// ❌ BAD
let result = registry.execute_tool(name, args).await?;

// ✅ GOOD
let result = registry.execute_tool(name, args)
    .await
    .with_tool_context(name)?;
```

### PTY Operations

```rust
use vtcode_core::errors::context::CommandErrorExt;

// ❌ BAD
pty_session.write_input(data).await?;

// ✅ GOOD
pty_session.write_input(data)
    .await
    .with_pty_context(&session_id)?;
```

### LLM Provider Operations

```rust
use vtcode_core::errors::context::ProviderErrorExt;

// ❌ BAD
let response = client.chat_completion(request).await?;

// ✅ GOOD
let response = client.chat_completion(request)
    .await
    .with_provider_context("anthropic")?;
```

## Recovery Strategies

### Fallback Values

Use when a reasonable default exists and the operation is non-critical:

```rust
use vtcode_core::errors::recovery::try_with_fallback;

// ❌ BAD - error is silently swallowed
let theme = load_theme().unwrap_or_default();

// ✅ GOOD - error is logged
let theme = try_with_fallback(
    || load_theme(),
    Theme::default(),
    "Failed to load user theme, using default",
);
```

### Retry with Backoff

Use for transient failures (network, rate limits):

```rust
use vtcode_core::errors::recovery::retry_with_backoff;

let response = retry_with_backoff(
    || async { api_client.fetch_data().await },
    3,      // max attempts
    1000,   // initial delay (ms)
).await?;
```

### Try Alternatives

Use when multiple approaches are available:

```rust
use vtcode_core::errors::recovery::try_alternatives;

let config = try_alternatives(vec![
    ("project config", Box::new(|| load_from(".vtcode/config.toml"))),
    ("user config", Box::new(|| load_from("~/.vtcode/config.toml"))),
    ("system config", Box::new(|| load_from("/etc/vtcode/config.toml"))),
    ("default", Box::new(|| Ok(Config::default()))),
])?;
```

### Timeout Protection

Use for operations that might hang:

```rust
use vtcode_core::errors::recovery::try_with_timeout;

let result = try_with_timeout(
    || async { slow_network_operation().await },
    5000,  // 5 second timeout
).await?;
```

## When to Use Each Pattern

### `.unwrap()` - NEVER in Production

```rust
// ❌ NEVER DO THIS
let value = map.get(&key).unwrap();

// ✅ DO THIS INSTEAD
let value = map.get(&key)
    .ok_or_else(|| anyhow!("Missing key: {}", key))?;

// ⚠️  ONLY IN TESTS
#[test]
fn test_something() {
    let value = map.get(&key).unwrap();  // OK in tests
    assert_eq!(value, &42);
}
```

### `.expect()` - Only for Provably Infallible Operations

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// ✅ OK - hardcoded regex must compile
static VERSION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+\.\d+\.\d+$")
        .expect("Hardcoded regex must compile")
});

// ✅ OK - mutex poisoning is unrecoverable
let data = mutex.lock()
    .expect("Mutex poisoned - unrecoverable state");
```

### `?` Operator - ALWAYS PREFER

```rust
use anyhow::{Context, Result};

fn process_config(path: &Path) -> Result<Config> {
    // ✅ ALWAYS use ? with context
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;

    Ok(config)
}
```

### `match` / `if let` - For Custom Recovery Logic

```rust
use tracing::warn;

// ✅ Custom error handling
match load_cache() {
    Ok(cache) => {
        info!("Loaded cache with {} entries", cache.len());
        cache
    }
    Err(e) => {
        warn!("Failed to load cache: {}. Rebuilding...", e);
        rebuild_cache()?
    }
}

// ✅ Type-specific handling
match execute_command(cmd).await {
    Ok(output) => output,
    Err(e) if is_permission_error(&e) => {
        warn!("Permission denied, trying with elevated privileges");
        execute_with_sudo(cmd).await?
    }
    Err(e) => return Err(e),
}
```

### `.unwrap_or()` / `.unwrap_or_default()` - For Acceptable Defaults

```rust
// ✅ OK - timeout has a reasonable default
let timeout = config.timeout_ms.unwrap_or(5000);

// ✅ OK - empty list is acceptable
let excluded = config.excluded_files.unwrap_or_default();

// ⚠️  CONSIDER - should we log this?
// Better to use try_with_fallback for logging
let max_retries = config.max_retries.unwrap_or(3);
```

## Common Scenarios

### Reading Configuration Files

```rust
use anyhow::{Context, Result};
use vtcode_core::errors::context::FileErrorExt;

fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_file_read_context(path)?;

    toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML in {}", path.display()))
}
```

### Executing Tools

```rust
use vtcode_core::errors::{ToolError, VTCodeError};
use vtcode_core::errors::context::ToolErrorExt;

async fn execute_tool(name: &str, args: Value) -> Result<Value> {
    let tool = registry.get_tool(name)
        .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

    tool.execute(args)
        .await
        .with_tool_context(name)
}
```

### Network Requests with Retry

```rust
use vtcode_core::errors::recovery::retry_with_backoff;
use vtcode_core::errors::context::ProviderErrorExt;

async fn call_llm_api(request: Request) -> Result<Response> {
    retry_with_backoff(
        || async {
            client.send(request.clone())
                .await
                .with_provider_context("anthropic")
        },
        3,      // max attempts
        1000,   // initial delay
    ).await
}
```

### Loading Optional Resources

```rust
use vtcode_core::errors::recovery::try_with_fallback;

fn load_theme() -> Theme {
    try_with_fallback(
        || load_theme_from_file(),
        Theme::default(),
        "Failed to load custom theme, using default",
    )
}
```

## Testing Error Paths

Always test error conditions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_file_error() {
        let result = load_config(Path::new("nonexistent.toml"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Failed to read"));
        assert!(error.to_string().contains("nonexistent.toml"));
    }

    #[test]
    fn test_tool_not_found_error() {
        let result = execute_tool("nonexistent_tool", json!({}));
        assert!(result.is_err());

        // Verify error type
        match result.unwrap_err().downcast_ref::<VTCodeError>() {
            Some(VTCodeError::Tool(ToolError::NotFound(_))) => (),
            _ => panic!("Expected ToolError::NotFound"),
        }
    }

    #[tokio::test]
    async fn test_retry_eventually_succeeds() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    let attempt = c.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(anyhow!("transient failure"))
                    } else {
                        Ok("success")
                    }
                }
            },
            3,
            10,
        ).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
```

## Migration Guide

### Replacing `.unwrap()`

```rust
// Before
let value = map.get(&key).unwrap();

// After
let value = map.get(&key)
    .ok_or_else(|| anyhow!("Missing required key: {}", key))?;
```

### Replacing `.expect()`

```rust
// Before
let config = load_config().expect("Config must exist");

// After
let config = load_config()
    .context("Failed to load required configuration")?;
```

### Adding Context to Existing Errors

```rust
// Before
let content = fs::read_to_string(&path)?;

// After
use vtcode_core::errors::context::FileErrorExt;
let content = fs::read_to_string(&path)
    .with_file_read_context(&path)?;
```

### Converting Silent Failures to Logged Fallbacks

```rust
// Before
let cache = load_cache().ok();

// After
use vtcode_core::errors::recovery::try_with_fallback;
let cache = try_with_fallback(
    || load_cache(),
    None,
    "Failed to load cache",
);
```

## Error Message Guidelines

### Good Error Messages

✅ **Specific and actionable**:
```rust
format!("Failed to read config file '{}': file not found. Create it with 'vtcode init'",
    path.display())
```

✅ **Include relevant context**:
```rust
format!("Tool '{}' execution failed with args {:?}: {}",
    tool_name, args, error)
```

✅ **Suggest solutions when possible**:
```rust
format!("API key not found in config. Set ANTHROPIC_API_KEY or add to vtcode.toml")
```

### Poor Error Messages

❌ **Too generic**:
```rust
"Error occurred"  // What error? Where?
```

❌ **No context**:
```rust
"File not found"  // Which file?
```

❌ **Technical jargon without explanation**:
```rust
"NoneError"  // What was None? Why?
```

## Best Practices Summary

1. **Always propagate errors with `?`** unless you have a specific reason to handle them
2. **Always add context** to errors using `.with_context()` or extension traits
3. **Log before swallowing** - use `try_with_fallback` for recoverable errors
4. **Use structured error types** from `vtcode_core::errors` for domain-specific errors
5. **Test error paths** as thoroughly as success paths
6. **Write helpful error messages** that include context and suggest solutions
7. **Never use `.unwrap()`** in production code
8. **Document why** when using `.expect()` (and only use it for provably infallible operations)
9. **Prefer recovery strategies** over panicking
10. **Be consistent** - follow these patterns throughout the codebase

## Reference Links

- Error module: `vtcode-core/src/errors/`
- Error type hierarchy: `vtcode-core/src/errors/mod.rs`
- Context helpers: `vtcode-core/src/errors/context.rs`
- Recovery strategies: `vtcode-core/src/errors/recovery.rs`
- Guidelines: `vtcode-core/src/errors/categories.rs`

## Examples in the Codebase

See these files for examples of proper error handling:

- Tool execution: `vtcode-core/src/tools/registry/executors.rs`
- Config loading: `vtcode-core/src/config/loader.rs`
- File operations: `vtcode-core/src/tools/file_ops.rs`
- Agent runner: `vtcode-core/src/core/agent/runner.rs`
- LLM factory: `vtcode-core/src/llm/factory.rs`

---

**Questions or suggestions?** Open an issue or PR to improve these standards.
