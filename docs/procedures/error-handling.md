---
type: standard-operating-procedure
id: error-handling
---

# Consistent Error Handling in Rust

## When to Use

When writing error handling code in Rust, whether for vtcode or any Rust project.

## Core Principles

1. **Use the right error type for the context**
   - Application code: `anyhow::Result<T>`
   - Library code: `thiserror::Error`
   - Critical paths: Custom error types

2. **Provide context, not just propagation**
   - Bad: `func()?`
   - Good: `func().context("Descriptive context")?`

3. **Errors should be actionable**
   - Include what failed
   - Include where it failed
   - Include why it failed (if known)

## Patterns

### Application Code (anyhow)

Use `anyhow` for application-level error handling:

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;

    serde_json::from_str(&content)
        .context("Failed to parse config JSON")
}
```

**Key points:**

- Use `Result<T>` (defaults to `anyhow::Result<T>`)
- Use `.context()` for static messages
- Use `.with_context(|| ...)` for dynamic messages
- Format paths/values in error messages

### Library Code (thiserror)

Use `thiserror` for library-level errors:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Failed to load session from {path}")]
    LoadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid memory config: {0}")]
    InvalidConfig(String),

    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
}
```

**Key points:**

- Derive `Error` trait
- Include relevant context in error variants
- Use `#[from]` for automatic conversion
- Use `#[source]` for error chains

### Critical Paths (Custom Types)

For performance-critical code, use custom error types:

```rust
#[derive(Debug)]
pub enum VectorDbError {
    NotFound,
    InvalidDimension { expected: usize, got: usize },
    BackendError(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for VectorDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Vector not found"),
            Self::InvalidDimension { expected, got } => {
                write!(f, "Invalid dimension: expected {expected}, got {got}")
            }
            Self::BackendError(e) => write!(f, "Backend error: {e}"),
        }
    }
}

impl std::error::Error for VectorDbError {}
```

## Error Message Guidelines

### Good Error Messages

✅ **Specific and actionable:**

```rust
return Err(anyhow!(
    "Failed to connect to Qdrant at {}: connection refused. Is Qdrant running?",
    url
));
```

✅ **Include context:**

```rust
fs::write(&path, data)
    .with_context(|| format!(
        "Failed to write {} bytes to {}",
        data.len(),
        path.display()
    ))?;
```

✅ **Explain what went wrong:**

```rust
ensure!(
    dimensions == 384,
    "Embedding dimension mismatch: expected 384 for default model, got {dimensions}"
);
```

### Bad Error Messages

❌ **Too generic:**

```rust
return Err(anyhow!("Error")); // What error?
```

❌ **Missing context:**

```rust
fs::read_to_string(path)?; // Which file? What failed?
```

❌ **Not actionable:**

```rust
return Err(anyhow!("Something went wrong")); // What? Where? Why?
```

## Context Propagation

### Filesystem Operations

Always add context for I/O:

```rust
// Reading
let content = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read {}", path.display()))?;

// Writing
fs::write(&path, data)
    .with_context(|| format!("Failed to write to {}", path.display()))?;

// Creating directories
fs::create_dir_all(&dir)
    .with_context(|| format!("Failed to create directory {}", dir.display()))?;
```

### Network Operations

Include URLs and operation details:

```rust
let response = client
    .get(&url)
    .send()
    .await
    .with_context(|| format!("Failed to fetch {url}"))?;

let body = response
    .text()
    .await
    .with_context(|| format!("Failed to read response body from {url}"))?;
```

### Parsing/Deserialization

Include what you're parsing:

```rust
let config: Config = serde_json::from_str(&content)
    .context("Failed to parse config JSON")?;

let value: i32 = s.parse()
    .with_context(|| format!("Failed to parse '{s}' as integer"))?;
```

## Using `ensure!` for Validation

Prefer `ensure!` over manual `if` + `Err`:

```rust
use anyhow::ensure;

// Good
ensure!(
    items.len() > 0,
    "Item list cannot be empty"
);

ensure!(
    dimensions <= 4096,
    "Embedding dimensions must be <= 4096, got {dimensions}"
);

// Instead of
if items.is_empty() {
    return Err(anyhow!("Item list cannot be empty"));
}
```

## Error Recovery

### Graceful Degradation

```rust
let config = match load_config(&config_path) {
    Ok(cfg) => cfg,
    Err(e) => {
        tracing::warn!("Failed to load config: {e}, using defaults");
        Config::default()
    }
};
```

### Retry Logic

```rust
use anyhow::{Context, Result};

async fn fetch_with_retry(url: &str, max_retries: u32) -> Result<String> {
    let mut last_error = None;

    for attempt in 0..max_retries {
        match reqwest::get(url).await {
            Ok(resp) => return Ok(resp.text().await?),
            Err(e) => {
                tracing::warn!("Attempt {}/{} failed: {}", attempt + 1, max_retries, e);
                last_error = Some(e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(attempt))).await;
            }
        }
    }

    Err(anyhow::Error::from(last_error.unwrap()))
        .context(format!("Failed to fetch {} after {max_retries} retries", url))
}
```

## Testing Error Cases

Always test error paths:

```rust
#[test]
fn test_load_config_missing_file() {
    let result = load_config(Path::new("/nonexistent/config.toml"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to read config"));
}

#[test]
fn test_invalid_dimension() {
    let result = validate_dimensions(999);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("dimension"));
}
```

## Logging Errors

Use appropriate log levels:

```rust
use tracing::{error, warn, debug};

// Critical errors (should never happen)
error!("Database corruption detected: {}", e);

// Expected errors (user-facing)
warn!("Failed to load user config, using defaults: {}", e);

// Debug information
debug!("Cache miss for key: {}", key);
```

## Anti-Patterns

❌ **Swallowing errors:**

```rust
let _ = func(); // Don't ignore errors silently
```

❌ **Generic error messages:**

```rust
.context("Error occurred") // Too vague
```

❌ **No error context:**

```rust
func()?; // What if this fails?
```

❌ **Panicking instead of errors:**

```rust
.unwrap() // Use ? or proper error handling
.expect("failed") // Return Result instead
```

❌ **String-based errors in libraries:**

```rust
return Err("something failed".into()); // Use thiserror
```

## Quick Reference

| Context | Error Type | Example |
|---------|-----------|---------|
| Application | `anyhow::Result<T>` | `load_config()` |
| Library | `thiserror::Error` | `MemoryError::NotFound` |
| Critical Path | Custom enum | `VectorDbError` |
| Validation | `anyhow::ensure!` | `ensure!(len > 0)` |
| I/O | `.with_context()` | `fs::read().with_context(...)` |

## Summary

1. Choose the right error type for your context
2. Always provide actionable context
3. Use `.context()` for I/O and parsing
4. Test error paths
5. Log appropriately
6. Never silently swallow errors
