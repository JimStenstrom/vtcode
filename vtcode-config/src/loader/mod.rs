//! Configuration loader for VTCode
//!
//! This module provides the main configuration loading and management system.
//! It has been organized into focused submodules for better maintainability:
//!
//! - [`file_io`]: File loading and parsing
//! - [`env_resolver`]: Environment variable resolution
//! - [`validator`]: Configuration validation
//! - [`merger`]: TOML document merging
//! - [`templates`]: Default configuration templates
//! - [`paths`]: Path resolution utilities

#[cfg(feature = "bootstrap")]
pub mod bootstrap;

pub mod env_resolver;
pub mod file_io;
pub mod merger;
pub mod paths;
pub mod templates;
pub mod validator;

// Re-export main types for public API
pub use validator::{SyntaxHighlightingConfig, VTCodeConfig};

use crate::defaults::{self, ConfigDefaultsProvider};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration manager for loading and validating configurations
#[derive(Clone)]
pub struct ConfigManager {
    config: VTCodeConfig,
    config_path: Option<PathBuf>,
    workspace_root: Option<PathBuf>,
    config_file_name: String,
}

impl ConfigManager {
    /// Load configuration from the default locations
    pub fn load() -> Result<Self> {
        Self::load_from_workspace(std::env::current_dir()?)
    }

    /// Load configuration from a specific workspace
    pub fn load_from_workspace(workspace: impl AsRef<Path>) -> Result<Self> {
        let workspace = workspace.as_ref();
        let defaults_provider = defaults::current_config_defaults();
        let workspace_paths = defaults_provider.workspace_paths_for(workspace);
        let workspace_root = workspace_paths.workspace_root().to_path_buf();
        let config_file_name = defaults_provider.config_file_name().to_string();

        // Try to find config file in standard locations
        if let Some(config_path) = file_io::find_config_file_in_workspace(workspace, defaults_provider.as_ref()) {
            let mut manager = Self::load_from_file(&config_path)?;
            manager.workspace_root = Some(workspace_root.clone());
            manager.config_file_name = config_file_name.clone();
            return Ok(manager);
        }

        // Use default configuration if no file found
        let config = VTCodeConfig::default();
        config
            .validate()
            .context("Default configuration failed validation")?;

        Ok(Self {
            config,
            config_path: None,
            workspace_root: Some(workspace_root),
            config_file_name,
        })
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let config = file_io::load_config_from_file(path)?;

        config
            .validate()
            .with_context(|| format!("Failed to validate config file: {}", path.display()))?;

        let config_file_name = file_io::extract_config_file_name(path);

        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
            workspace_root: path.parent().map(Path::to_path_buf),
            config_file_name,
        })
    }

    /// Get the loaded configuration
    pub fn config(&self) -> &VTCodeConfig {
        &self.config
    }

    /// Get the configuration file path (if loaded from file)
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Get session duration from agent config
    pub fn session_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60 * 60) // Default 1 hour
    }

    /// Persist configuration to a specific path, preserving comments
    pub fn save_config_to_path(path: impl AsRef<Path>, config: &VTCodeConfig) -> Result<()> {
        merger::save_config_preserving_comments(path.as_ref(), config)
    }

    /// Persist configuration to the manager's associated path or workspace
    pub fn save_config(&self, config: &VTCodeConfig) -> Result<()> {
        if let Some(path) = &self.config_path {
            return Self::save_config_to_path(path, config);
        }

        if let Some(workspace_root) = &self.workspace_root {
            let path = workspace_root.join(&self.config_file_name);
            return Self::save_config_to_path(path, config);
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let path = cwd.join(&self.config_file_name);
        Self::save_config_to_path(path, config)
    }
}

impl VTCodeConfig {
    #[cfg(feature = "bootstrap")]
    /// Bootstrap project with config + gitignore
    pub fn bootstrap_project<P: AsRef<Path>>(workspace: P, force: bool) -> Result<Vec<String>> {
        Self::bootstrap_project_with_options(workspace, force, false)
    }

    #[cfg(feature = "bootstrap")]
    /// Bootstrap project with config + gitignore, with option to create in home directory
    pub fn bootstrap_project_with_options<P: AsRef<Path>>(
        workspace: P,
        force: bool,
        use_home_dir: bool,
    ) -> Result<Vec<String>> {
        let workspace = workspace.as_ref().to_path_buf();
        defaults::with_config_defaults(|provider| {
            Self::bootstrap_project_with_provider(&workspace, force, use_home_dir, provider)
        })
    }

    #[cfg(feature = "bootstrap")]
    /// Bootstrap project files using the supplied [`ConfigDefaultsProvider`].
    pub fn bootstrap_project_with_provider<P: AsRef<Path>>(
        workspace: P,
        force: bool,
        use_home_dir: bool,
        defaults_provider: &dyn ConfigDefaultsProvider,
    ) -> Result<Vec<String>> {
        let workspace = workspace.as_ref();
        let config_file_name = defaults_provider.config_file_name().to_string();
        let (config_path, gitignore_path) = file_io::determine_bootstrap_targets(
            workspace,
            use_home_dir,
            &config_file_name,
            defaults_provider,
        )?;

        paths::ensure_parent_dir(&config_path)?;
        paths::ensure_parent_dir(&gitignore_path)?;

        let mut created_files = Vec::new();

        if !config_path.exists() || force {
            let config_content = templates::default_vtcode_toml_template();

            fs::write(&config_path, config_content).with_context(|| {
                format!("Failed to write config file: {}", config_path.display())
            })?;

            if let Some(file_name) = config_path.file_name().and_then(|name| name.to_str()) {
                created_files.push(file_name.to_string());
            }
        }

        if !gitignore_path.exists() || force {
            let gitignore_content = templates::default_vtcode_gitignore();
            fs::write(&gitignore_path, gitignore_content).with_context(|| {
                format!(
                    "Failed to write gitignore file: {}",
                    gitignore_path.display()
                )
            })?;

            if let Some(file_name) = gitignore_path.file_name().and_then(|name| name.to_str()) {
                created_files.push(file_name.to_string());
            }
        }

        Ok(created_files)
    }

    #[cfg(feature = "bootstrap")]
    /// Create sample configuration file
    pub fn create_sample_config<P: AsRef<Path>>(output: P) -> Result<()> {
        let output = output.as_ref();
        let config_content = templates::default_vtcode_toml_template();

        fs::write(output, config_content)
            .with_context(|| format!("Failed to write config file: {}", output.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::WorkspacePathsDefaults;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use vtcode_commons::StaticWorkspacePaths;

    #[test]
    fn load_from_file_rejects_invalid_syntax_highlighting() {
        let mut temp_file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(
            temp_file,
            "[syntax_highlighting]\nhighlight_timeout_ms = 0\n"
        )
        .expect("failed to write temp config");

        let result = ConfigManager::load_from_file(temp_file.path());
        assert!(result.is_err(), "expected validation error");
        let error = format!("{:?}", result.err().unwrap());
        assert!(
            error.contains("validate"),
            "expected validation context in error, got: {}",
            error
        );
    }

    #[test]
    fn config_defaults_provider_overrides_paths_and_theme() {
        let workspace = assert_fs::TempDir::new().expect("failed to create workspace");
        let workspace_root = workspace.path();
        let config_dir = workspace_root.join("config-root");
        fs::create_dir_all(&config_dir).expect("failed to create config directory");

        let config_file_name = "custom-config.toml";
        let config_path = config_dir.join(config_file_name);
        let serialized =
            toml::to_string(&VTCodeConfig::default()).expect("failed to serialize default config");
        fs::write(&config_path, serialized).expect("failed to write config file");

        let static_paths = StaticWorkspacePaths::new(workspace_root, &config_dir);
        let provider = WorkspacePathsDefaults::new(Arc::new(static_paths))
            .with_config_file_name(config_file_name)
            .with_home_paths(Vec::new())
            .with_syntax_theme("custom-theme")
            .with_syntax_languages(vec!["zig".to_string()]);

        defaults::provider::with_config_defaults_provider_for_test(Arc::new(provider), || {
            let manager = ConfigManager::load_from_workspace(workspace_root)
                .expect("failed to load workspace config");

            let resolved_path = manager
                .config_path()
                .expect("config path should be resolved");
            assert_eq!(resolved_path, config_path);

            assert_eq!(defaults::SyntaxHighlightingDefaults::theme(), "custom-theme");
            assert_eq!(
                defaults::SyntaxHighlightingDefaults::enabled_languages(),
                vec!["zig".to_string()]
            );
        });
    }
}
