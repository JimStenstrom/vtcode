//! Environment variable resolution for configuration values
//!
//! This module provides utilities for resolving environment variables
//! in configuration strings and paths.

use std::env;
use std::path::PathBuf;

/// Resolve environment variables in a string
///
/// Supports both `$VAR` and `${VAR}` syntax.
///
/// # Examples
///
/// ```
/// use std::env;
/// use vtcode_config::loader::env_resolver::resolve_env_vars;
///
/// env::set_var("TEST_VAR", "hello");
/// let result = resolve_env_vars("Value is: $TEST_VAR");
/// assert_eq!(result, "Value is: hello");
/// ```
pub fn resolve_env_vars(value: &str) -> String {
    let mut result = value.to_string();

    // Replace ${VAR} and $VAR patterns
    for (key, val) in env::vars() {
        // Replace ${VAR} first (more specific)
        result = result.replace(&format!("${{{}}}", key), &val);
        // Then replace $VAR
        result = result.replace(&format!("${}", key), &val);
    }

    result
}

/// Resolve a path with environment variables and expand tilde
///
/// # Examples
///
/// ```
/// use std::env;
/// use vtcode_config::loader::env_resolver::resolve_path;
///
/// env::set_var("MY_DIR", "/tmp/test");
/// let path = resolve_path("$MY_DIR/config.toml");
/// assert_eq!(path.to_str().unwrap(), "/tmp/test/config.toml");
/// ```
pub fn resolve_path(path: &str) -> PathBuf {
    let resolved = resolve_env_vars(path);

    // Expand tilde to home directory
    if resolved.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            let mut home_path = PathBuf::from(home);
            home_path.push(&resolved[2..]);
            return home_path;
        }
    }

    PathBuf::from(resolved)
}

/// Get API key from environment variable
///
/// Returns `None` if the variable is not set or is empty/whitespace.
pub fn get_api_key_from_env(env_var: &str) -> Option<String> {
    env::var(env_var).ok().filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_env_vars_with_braces() {
        unsafe { env::set_var("TEST_VAR_1", "value1"); }
        let result = resolve_env_vars("Path: ${TEST_VAR_1}/subdir");
        assert_eq!(result, "Path: value1/subdir");
    }

    #[test]
    fn resolve_env_vars_without_braces() {
        unsafe { env::set_var("TEST_VAR_2", "value2"); }
        let result = resolve_env_vars("Path: $TEST_VAR_2/subdir");
        assert_eq!(result, "Path: value2/subdir");
    }

    #[test]
    fn resolve_env_vars_no_vars() {
        let result = resolve_env_vars("Just a plain string");
        assert_eq!(result, "Just a plain string");
    }

    #[test]
    fn resolve_path_expands_home() {
        let path = resolve_path("~/config/vtcode.toml");
        let expected_prefix = env::var("HOME").unwrap();
        assert!(path.to_str().unwrap().starts_with(&expected_prefix));
        assert!(path.to_str().unwrap().ends_with("config/vtcode.toml"));
    }

    #[test]
    fn resolve_path_with_env_var() {
        unsafe { env::set_var("CONFIG_DIR", "/etc/vtcode"); }
        let path = resolve_path("$CONFIG_DIR/vtcode.toml");
        assert_eq!(path.to_str().unwrap(), "/etc/vtcode/vtcode.toml");
    }

    #[test]
    fn get_api_key_from_env_returns_value() {
        unsafe { env::set_var("TEST_API_KEY", "sk-test123"); }
        let key = get_api_key_from_env("TEST_API_KEY");
        assert_eq!(key, Some("sk-test123".to_string()));
    }

    #[test]
    fn get_api_key_from_env_returns_none_for_empty() {
        unsafe { env::set_var("EMPTY_API_KEY", "   "); }
        let key = get_api_key_from_env("EMPTY_API_KEY");
        assert_eq!(key, None);
    }

    #[test]
    fn get_api_key_from_env_returns_none_for_missing() {
        let key = get_api_key_from_env("NON_EXISTENT_VAR_XYZ");
        assert_eq!(key, None);
    }
}
