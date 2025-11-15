//! Configuration file loading and parsing

use super::validator::VTCodeConfig;
use crate::defaults::{self, ConfigDefaultsProvider};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Load and parse configuration from a file
///
/// Reads TOML content and deserializes into VTCodeConfig
pub fn load_config_from_file(path: &Path) -> Result<VTCodeConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: VTCodeConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Check if a configuration file exists at the given path
pub fn config_file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Find configuration file in standard locations
///
/// Searches in order:
/// 1. Workspace root (e.g., `vtcode.toml`)
/// 2. Config directory (e.g., `.vtcode/vtcode.toml`)
/// 3. Home directory configurations
/// 4. Project-specific config paths
///
/// Returns the first valid configuration file found
pub fn find_config_file_in_workspace(
    workspace: &Path,
    defaults_provider: &dyn ConfigDefaultsProvider,
) -> Option<PathBuf> {
    let workspace_paths = defaults_provider.workspace_paths_for(workspace);
    let workspace_root = workspace_paths.workspace_root();
    let config_dir = workspace_paths.config_dir();
    let config_file_name = defaults_provider.config_file_name();

    // Try workspace root first
    let workspace_config = workspace_root.join(config_file_name);
    if config_file_exists(&workspace_config) {
        return Some(workspace_config);
    }

    // Try config directory
    let config_dir_config = config_dir.join(config_file_name);
    if config_file_exists(&config_dir_config) {
        return Some(config_dir_config);
    }

    // Try home directory configurations
    for home_config_path in defaults_provider.home_config_paths(config_file_name) {
        if config_file_exists(&home_config_path) {
            return Some(home_config_path);
        }
    }

    // Try project-specific configuration
    if let Some(project_config) =
        super::paths::project_config_path(&config_dir, workspace_root, config_file_name)
    {
        return Some(project_config);
    }

    None
}

/// Extract config file name from path or use default
pub fn extract_config_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| {
            defaults::current_config_defaults()
                .config_file_name()
                .to_string()
        })
}

#[cfg(feature = "bootstrap")]
/// Determine bootstrap targets (config and gitignore paths)
///
/// This is a re-export of the bootstrap module's function for convenience
pub use super::bootstrap::determine_bootstrap_targets;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_valid_config_file() {
        let mut temp_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            temp_file,
            r#"
[agent]
provider = "openai"
default_model = "gpt-5-nano"

[tools]
default_policy = "prompt"
"#
        )
        .expect("failed to write config");

        let config = load_config_from_file(temp_file.path()).expect("failed to load config");
        assert_eq!(config.agent.provider.to_string(), "openai");
        assert_eq!(config.agent.default_model, "gpt-5-nano");
    }

    #[test]
    fn load_invalid_toml_fails() {
        let mut temp_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(temp_file, "invalid toml {{{{").expect("failed to write");

        let result = load_config_from_file(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));
    }

    #[test]
    fn config_file_exists_check() {
        let temp_file = NamedTempFile::new().expect("failed to create temp file");
        assert!(config_file_exists(temp_file.path()));

        let non_existent = PathBuf::from("/tmp/non-existent-config-file-xyz.toml");
        assert!(!config_file_exists(&non_existent));
    }

    #[test]
    fn extract_config_file_name_from_path() {
        let path = PathBuf::from("/home/user/.vtcode/vtcode.toml");
        assert_eq!(extract_config_file_name(&path), "vtcode.toml");
    }
}
