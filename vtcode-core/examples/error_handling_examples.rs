//! Error Handling Examples for vtcode-core
//!
//! This file demonstrates proper error handling patterns using the
//! vtcode-core error handling infrastructure.
//!
//! Run with: cargo run --example error_handling_examples

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

// Import error handling utilities
use vtcode_core::errors::{
    VTCodeError, UserError, SystemError, ConfigError, ToolError, ProviderError,
};
use vtcode_core::errors::context::{
    FileErrorExt, CommandErrorExt, ToolErrorExt, ProviderErrorExt,
};
use vtcode_core::errors::recovery::{
    try_with_fallback, try_alternatives,
};

// Example 1: File Operations with Proper Error Context
fn read_config_file(path: &Path) -> Result<String> {
    // ❌ BAD - no context
    // fs::read_to_string(path)?

    // ✅ GOOD - with file error context
    fs::read_to_string(path)
        .with_file_read_context(path)
}

fn write_config_file(path: &Path, content: &str) -> Result<()> {
    // ✅ GOOD - with file error context
    fs::write(path, content)
        .with_file_write_context(path)
}

fn delete_temp_file(path: &Path) -> Result<()> {
    // ✅ GOOD - with file error context
    fs::remove_file(path)
        .with_file_delete_context(path)
}

// Example 2: Configuration Loading with Custom Error Types
fn load_api_key(config_path: &Path) -> Result<String> {
    let content = fs::read_to_string(config_path)
        .with_file_read_context(config_path)?;

    let config: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML in {}", config_path.display()))?;

    // Use custom error type for missing configuration
    let api_key = config
        .get("api_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::Missing("api_key".to_string()))?;

    Ok(api_key.to_string())
}

// Example 3: Tool Execution with Error Context
async fn execute_grep_tool(pattern: &str, file: &Path) -> Result<String> {
    // Simulate tool execution
    if !file.exists() {
        return Err(UserError::FileNotFound(file.display().to_string()).into());
    }

    let content = fs::read_to_string(file)
        .with_file_read_context(file)?;

    // Simulate grep operation
    let matches: Vec<&str> = content
        .lines()
        .filter(|line| line.contains(pattern))
        .collect();

    if matches.is_empty() {
        Ok(String::new())
    } else {
        Ok(matches.join("\n"))
    }
}

// Example 4: Recovery Strategy - Fallback Values
#[derive(Debug, Clone)]
struct Theme {
    name: String,
    colors: Vec<String>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "default".to_string(),
            colors: vec!["#000000".to_string(), "#FFFFFF".to_string()],
        }
    }
}

fn load_theme_from_file() -> Result<Theme> {
    // Simulate theme loading that might fail
    Err(anyhow!("Theme file not found"))
}

fn load_theme() -> Theme {
    // ❌ BAD - error is silently swallowed
    // load_theme_from_file().unwrap_or_default()

    // ✅ GOOD - error is logged
    try_with_fallback(
        || load_theme_from_file(),
        Theme::default(),
        "Failed to load custom theme, using default",
    )
}

// Example 5: Try Alternatives Pattern
fn load_config_from_multiple_locations() -> Result<String> {
    try_alternatives(vec![
        (
            "project config",
            Box::new(|| {
                fs::read_to_string(".vtcode/config.toml")
                    .with_file_read_context(Path::new(".vtcode/config.toml"))
            }),
        ),
        (
            "user config",
            Box::new(|| {
                let home = dirs::home_dir().ok_or_else(|| anyhow!("No home directory"))?;
                let path = home.join(".vtcode/config.toml");
                fs::read_to_string(&path).with_file_read_context(&path)
            }),
        ),
        (
            "system config",
            Box::new(|| {
                let path = Path::new("/etc/vtcode/config.toml");
                fs::read_to_string(path).with_file_read_context(path)
            }),
        ),
        (
            "default config",
            Box::new(|| Ok("[vtcode]\nversion = \"0.1.0\"\n".to_string())),
        ),
    ])
}

// Example 6: Command Execution with Context
fn execute_git_command(args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .with_command_context(&format!("git {}", args.join(" ")))?;

    if !output.status.success() {
        return Err(anyhow!(
            "Git command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    String::from_utf8(output.stdout)
        .with_context(|| "Failed to parse git command output as UTF-8")
}

// Example 7: Tool Registry Pattern
struct ToolRegistry {
    tools: std::collections::HashMap<String, String>,
}

impl ToolRegistry {
    fn new() -> Self {
        let mut tools = std::collections::HashMap::new();
        tools.insert("grep".to_string(), "Search tool".to_string());
        tools.insert("ls".to_string(), "List files tool".to_string());
        ToolRegistry { tools }
    }

    fn execute_tool(&self, name: &str, args: &str) -> Result<String> {
        // Use custom error type
        if !self.tools.contains_key(name) {
            return Err(ToolError::NotFound(name.to_string()).into());
        }

        // Simulate tool execution
        // In real code, this would have .with_tool_context(name)
        Ok(format!("Executed {} with args: {}", name, args))
    }
}

// Example 8: LLM Provider Error Handling
async fn call_llm_provider(prompt: &str, api_key: &str) -> Result<String> {
    if api_key.is_empty() {
        return Err(ProviderError::InvalidApiKey.into());
    }

    // Simulate API call
    // In real code, this would use .with_provider_context("anthropic")
    Ok(format!("Response to: {}", prompt))
}

// Example 9: Proper expect() Usage
fn compile_time_regex() -> regex::Regex {
    // ✅ OK - hardcoded regex must compile
    regex::Regex::new(r"^\d+\.\d+\.\d+$")
        .expect("Hardcoded version regex must compile")
}

// Example 10: Match Pattern for Custom Error Handling
async fn load_cache_with_rebuild() -> Result<Vec<String>> {
    match try_load_cache() {
        Ok(cache) => {
            tracing::info!("Loaded cache with {} entries", cache.len());
            Ok(cache)
        }
        Err(e) => {
            tracing::warn!("Failed to load cache: {}. Rebuilding...", e);
            rebuild_cache().await
        }
    }
}

fn try_load_cache() -> Result<Vec<String>> {
    // Simulate cache loading
    Err(anyhow!("Cache file corrupted"))
}

async fn rebuild_cache() -> Result<Vec<String>> {
    // Simulate cache rebuilding
    Ok(vec!["item1".to_string(), "item2".to_string()])
}

// Example 11: Comprehensive Config Loading
fn load_project_config() -> Result<ProjectConfig> {
    // Load config file
    let config_path = Path::new("vtcode.toml");
    let content = fs::read_to_string(config_path)
        .with_file_read_context(config_path)?;

    // Parse TOML
    let toml: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    // Extract required fields with proper error types
    let project_name = toml
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| ConfigError::Missing("project.name".to_string()))?;

    let version = toml
        .get("project")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::Missing("project.version".to_string()))?;

    // Optional field with default
    let timeout_ms = toml
        .get("project")
        .and_then(|p| p.get("timeout_ms"))
        .and_then(|t| t.as_integer())
        .unwrap_or(5000) as u64;

    Ok(ProjectConfig {
        name: project_name.to_string(),
        version: version.to_string(),
        timeout_ms,
    })
}

#[derive(Debug)]
struct ProjectConfig {
    name: String,
    version: String,
    timeout_ms: u64,
}

// Main function demonstrating all examples
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Error Handling Examples for vtcode-core\n");
    println!("========================================\n");

    // Example 1: File Operations
    println!("Example 1: File Operations with Error Context");
    let temp_path = PathBuf::from("/tmp/test_config.toml");
    match write_config_file(&temp_path, "test = true\n") {
        Ok(_) => println!("✓ File written successfully"),
        Err(e) => println!("✗ Error: {:#}", e),
    }
    println!();

    // Example 2: Configuration Loading
    println!("Example 2: Configuration with Custom Error Types");
    match load_config_from_multiple_locations() {
        Ok(config) => println!("✓ Config loaded: {} bytes", config.len()),
        Err(e) => println!("✗ Error: {:#}", e),
    }
    println!();

    // Example 3: Tool Execution
    println!("Example 3: Tool Execution with Error Context");
    let registry = ToolRegistry::new();
    match registry.execute_tool("grep", "pattern=test") {
        Ok(result) => println!("✓ Tool executed: {}", result),
        Err(e) => println!("✗ Error: {:#}", e),
    }

    match registry.execute_tool("nonexistent", "") {
        Ok(_) => println!("✓ Unexpected success"),
        Err(e) => {
            println!("✓ Expected error: {:#}", e);
            // Verify error type
            if let Some(VTCodeError::Tool(ToolError::NotFound(_))) =
                e.downcast_ref::<VTCodeError>()
            {
                println!("  (Correct error type: ToolError::NotFound)");
            }
        }
    }
    println!();

    // Example 4: Recovery with Fallback
    println!("Example 4: Recovery Strategy - Fallback Values");
    let theme = load_theme();
    println!("✓ Theme loaded: {:?}", theme);
    println!();

    // Example 5: Git Command
    println!("Example 5: Command Execution with Context");
    match execute_git_command(&["--version"]) {
        Ok(version) => println!("✓ Git version: {}", version.trim()),
        Err(e) => println!("✗ Error: {:#}", e),
    }
    println!();

    // Example 6: LLM Provider
    println!("Example 6: LLM Provider Error Handling");
    match call_llm_provider("Hello", "").await {
        Ok(response) => println!("✓ Response: {}", response),
        Err(e) => {
            println!("✓ Expected error: {:#}", e);
            if let Some(VTCodeError::Provider(ProviderError::InvalidApiKey)) =
                e.downcast_ref::<VTCodeError>()
            {
                println!("  (Correct error type: ProviderError::InvalidApiKey)");
            }
        }
    }
    println!();

    // Example 7: Cache with Rebuild
    println!("Example 7: Custom Error Handling with Match");
    match load_cache_with_rebuild().await {
        Ok(cache) => println!("✓ Cache loaded/rebuilt: {} items", cache.len()),
        Err(e) => println!("✗ Error: {:#}", e),
    }
    println!();

    println!("========================================");
    println!("\nAll examples completed!");
    println!("\nKey Takeaways:");
    println!("1. Always use .with_context() or extension traits");
    println!("2. Use custom error types for domain-specific errors");
    println!("3. Log before using fallback values");
    println!("4. Never use .unwrap() in production code");
    println!("5. Test error paths as thoroughly as success paths");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_config_error_context() {
        let result = read_config_file(Path::new("nonexistent.toml"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Failed to read file"));
        assert!(error.to_string().contains("nonexistent.toml"));
    }

    #[test]
    fn test_tool_not_found_error() {
        let registry = ToolRegistry::new();
        let result = registry.execute_tool("nonexistent", "");

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<VTCodeError>() {
            Some(VTCodeError::Tool(ToolError::NotFound(name))) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected ToolError::NotFound"),
        }
    }

    #[test]
    fn test_theme_fallback() {
        let theme = load_theme();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.colors.len(), 2);
    }

    #[tokio::test]
    async fn test_provider_invalid_api_key() {
        let result = call_llm_provider("test", "").await;
        assert!(result.is_err());

        match result.unwrap_err().downcast_ref::<VTCodeError>() {
            Some(VTCodeError::Provider(ProviderError::InvalidApiKey)) => (),
            _ => panic!("Expected ProviderError::InvalidApiKey"),
        }
    }

    #[test]
    fn test_compile_time_regex() {
        let re = compile_time_regex();
        assert!(re.is_match("1.2.3"));
        assert!(!re.is_match("invalid"));
    }

    #[tokio::test]
    async fn test_cache_rebuild() {
        let result = load_cache_with_rebuild().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
