//! Configuration validation

use crate::defaults::SyntaxHighlightingDefaults;
use anyhow::{ensure, Context, Result};

/// Syntax highlighting configuration
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SyntaxHighlightingConfig {
    /// Enable syntax highlighting for tool output
    #[serde(default = "crate::defaults::syntax_highlighting::enabled")]
    pub enabled: bool,

    /// Theme to use for syntax highlighting
    #[serde(default = "crate::defaults::syntax_highlighting::theme")]
    pub theme: String,

    /// Enable theme caching for better performance
    #[serde(default = "crate::defaults::syntax_highlighting::cache_themes")]
    pub cache_themes: bool,

    /// Maximum file size for syntax highlighting (in MB)
    #[serde(default = "crate::defaults::syntax_highlighting::max_file_size_mb")]
    pub max_file_size_mb: usize,

    /// Languages to enable syntax highlighting for
    #[serde(default = "crate::defaults::syntax_highlighting::enabled_languages")]
    pub enabled_languages: Vec<String>,

    /// Performance settings - highlight timeout in milliseconds
    #[serde(default = "crate::defaults::syntax_highlighting::highlight_timeout_ms")]
    pub highlight_timeout_ms: u64,
}

impl Default for SyntaxHighlightingConfig {
    fn default() -> Self {
        Self {
            enabled: crate::defaults::syntax_highlighting::enabled(),
            theme: crate::defaults::syntax_highlighting::theme(),
            cache_themes: crate::defaults::syntax_highlighting::cache_themes(),
            max_file_size_mb: crate::defaults::syntax_highlighting::max_file_size_mb(),
            enabled_languages: crate::defaults::syntax_highlighting::enabled_languages(),
            highlight_timeout_ms: crate::defaults::syntax_highlighting::highlight_timeout_ms(),
        }
    }
}

impl SyntaxHighlightingConfig {
    /// Validate syntax highlighting configuration
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        ensure!(
            self.max_file_size_mb >= SyntaxHighlightingDefaults::min_file_size_mb(),
            "Syntax highlighting max_file_size_mb must be at least {} MB",
            SyntaxHighlightingDefaults::min_file_size_mb()
        );

        ensure!(
            self.highlight_timeout_ms >= SyntaxHighlightingDefaults::min_highlight_timeout_ms(),
            "Syntax highlighting highlight_timeout_ms must be at least {} ms",
            SyntaxHighlightingDefaults::min_highlight_timeout_ms()
        );

        ensure!(
            !self.theme.trim().is_empty(),
            "Syntax highlighting theme must not be empty"
        );

        ensure!(
            self.enabled_languages
                .iter()
                .all(|lang| !lang.trim().is_empty()),
            "Syntax highlighting languages must not contain empty entries"
        );

        Ok(())
    }
}

/// Main configuration structure for VTCode
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct VTCodeConfig {
    /// Agent-wide settings
    #[serde(default)]
    pub agent: crate::core::AgentConfig,

    /// Tool execution policies
    #[serde(default)]
    pub tools: crate::core::ToolsConfig,

    /// Unix command permissions
    #[serde(default)]
    pub commands: crate::core::CommandsConfig,

    /// Permission system settings (resolution, audit logging, caching)
    #[serde(default)]
    pub permissions: crate::core::PermissionsConfig,

    /// Security settings
    #[serde(default)]
    pub security: crate::core::SecurityConfig,

    /// UI settings
    #[serde(default)]
    pub ui: crate::root::UiConfig,

    /// PTY settings
    #[serde(default)]
    pub pty: crate::root::PtyConfig,

    /// Debug and tracing settings
    #[serde(default)]
    pub debug: crate::debug::DebugConfig,

    /// Context features (e.g., Decision Ledger)
    #[serde(default)]
    pub context: crate::context::ContextFeaturesConfig,

    /// Router configuration (dynamic model + engine selection)
    #[serde(default)]
    pub router: crate::router::RouterConfig,

    /// Telemetry configuration (logging, trajectory)
    #[serde(default)]
    pub telemetry: crate::telemetry::TelemetryConfig,

    /// Syntax highlighting configuration
    #[serde(default)]
    pub syntax_highlighting: SyntaxHighlightingConfig,

    /// Timeout ceilings and UI warning thresholds
    #[serde(default)]
    pub timeouts: crate::timeouts::TimeoutsConfig,

    /// Automation configuration
    #[serde(default)]
    pub automation: crate::core::AutomationConfig,

    /// Prompt cache configuration (local + provider integration)
    #[serde(default)]
    pub prompt_cache: crate::core::PromptCachingConfig,

    /// Model Context Protocol configuration
    #[serde(default)]
    pub mcp: crate::mcp::McpClientConfig,

    /// Agent Client Protocol configuration
    #[serde(default)]
    pub acp: crate::acp::AgentClientProtocolConfig,

    /// Lifecycle hooks configuration
    #[serde(default)]
    pub hooks: crate::hooks::HooksConfig,

    /// Model-specific behavior configuration
    #[serde(default)]
    pub model: crate::core::ModelConfig,

    /// Memory system configuration
    #[serde(default)]
    pub memory: crate::core::MemoryConfig,

    /// VectorDB configuration
    #[serde(default)]
    pub vectordb: crate::core::VectorDbConfig,
}

impl VTCodeConfig {
    /// Validate entire configuration
    pub fn validate(&self) -> Result<()> {
        self.syntax_highlighting
            .validate()
            .context("Invalid syntax_highlighting configuration")?;

        self.context
            .validate()
            .context("Invalid context configuration")?;

        self.router
            .validate()
            .context("Invalid router configuration")?;

        self.hooks
            .validate()
            .context("Invalid hooks configuration")?;

        self.timeouts
            .validate()
            .context("Invalid timeouts configuration")?;

        self.memory
            .validate()
            .context("Invalid memory configuration")?;

        self.vectordb
            .validate()
            .context("Invalid vectordb configuration")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_highlighting_defaults_are_valid() {
        let config = SyntaxHighlightingConfig::default();
        config
            .validate()
            .expect("default syntax highlighting config should be valid");
    }

    #[test]
    fn vtcode_config_validation_fails_for_invalid_highlight_timeout() {
        let mut config = VTCodeConfig::default();
        config.syntax_highlighting.highlight_timeout_ms = 0;
        let error = config
            .validate()
            .expect_err("validation should fail for zero highlight timeout");
        let error_string = format!("{:?}", error);
        assert!(
            error_string.contains("highlight timeout") || error_string.contains("syntax_highlighting"),
            "expected error to mention highlight timeout or syntax_highlighting, got: {}",
            error
        );
    }

    #[test]
    fn syntax_highlighting_validates_theme_not_empty() {
        let mut config = SyntaxHighlightingConfig::default();
        config.theme = "".to_string();
        let error = config
            .validate()
            .expect_err("validation should fail for empty theme");
        assert!(error.to_string().contains("theme"));
    }

    #[test]
    fn syntax_highlighting_validates_languages_not_empty() {
        let mut config = SyntaxHighlightingConfig::default();
        config.enabled_languages = vec!["rust".to_string(), "".to_string()];
        let error = config
            .validate()
            .expect_err("validation should fail for empty language entry");
        assert!(error.to_string().contains("languages"));
    }

    #[test]
    fn syntax_highlighting_disabled_skips_validation() {
        let mut config = SyntaxHighlightingConfig::default();
        config.enabled = false;
        config.theme = "".to_string(); // Invalid if enabled
        config.highlight_timeout_ms = 0; // Invalid if enabled

        // Should pass validation because enabled = false
        config
            .validate()
            .expect("disabled config should skip validation");
    }
}
