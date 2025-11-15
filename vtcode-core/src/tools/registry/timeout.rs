//! Timeout policy management for tools

use crate::config::TimeoutsConfig;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTimeoutCategory {
    Default,
    Pty,
    Mcp,
}

impl ToolTimeoutCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ToolTimeoutCategory::Default => "standard",
            ToolTimeoutCategory::Pty => "PTY",
            ToolTimeoutCategory::Mcp => "MCP",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolTimeoutPolicy {
    default_ceiling: Option<Duration>,
    pty_ceiling: Option<Duration>,
    mcp_ceiling: Option<Duration>,
    warning_fraction: f32,
}

impl Default for ToolTimeoutPolicy {
    fn default() -> Self {
        Self {
            default_ceiling: Some(Duration::from_secs(180)),
            pty_ceiling: Some(Duration::from_secs(300)),
            mcp_ceiling: Some(Duration::from_secs(120)),
            warning_fraction: 0.8,
        }
    }
}

impl ToolTimeoutPolicy {
    pub fn from_config(config: &TimeoutsConfig) -> Self {
        Self {
            default_ceiling: config.ceiling_duration(config.default_ceiling_seconds),
            pty_ceiling: config.ceiling_duration(config.pty_ceiling_seconds),
            mcp_ceiling: config.ceiling_duration(config.mcp_ceiling_seconds),
            warning_fraction: config.warning_threshold_fraction().clamp(0.0, 0.99),
        }
    }

    /// Validate the timeout policy configuration
    ///
    /// Ensures that:
    /// - Ceiling values are within reasonable bounds (1s - 3600s)
    /// - Warning fraction is between 0.0 and 1.0
    /// - No ceiling is configured as 0 seconds
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate default ceiling
        if let Some(ceiling) = self.default_ceiling {
            if ceiling < Duration::from_secs(1) {
                anyhow::bail!(
                    "default_ceiling_seconds must be at least 1 second (got {}s)",
                    ceiling.as_secs()
                );
            }
            if ceiling > Duration::from_secs(3600) {
                anyhow::bail!(
                    "default_ceiling_seconds must not exceed 3600 seconds/1 hour (got {}s)",
                    ceiling.as_secs()
                );
            }
        }

        // Validate PTY ceiling
        if let Some(ceiling) = self.pty_ceiling {
            if ceiling < Duration::from_secs(1) {
                anyhow::bail!(
                    "pty_ceiling_seconds must be at least 1 second (got {}s)",
                    ceiling.as_secs()
                );
            }
            if ceiling > Duration::from_secs(3600) {
                anyhow::bail!(
                    "pty_ceiling_seconds must not exceed 3600 seconds/1 hour (got {}s)",
                    ceiling.as_secs()
                );
            }
        }

        // Validate MCP ceiling
        if let Some(ceiling) = self.mcp_ceiling {
            if ceiling < Duration::from_secs(1) {
                anyhow::bail!(
                    "mcp_ceiling_seconds must be at least 1 second (got {}s)",
                    ceiling.as_secs()
                );
            }
            if ceiling > Duration::from_secs(3600) {
                anyhow::bail!(
                    "mcp_ceiling_seconds must not exceed 3600 seconds/1 hour (got {}s)",
                    ceiling.as_secs()
                );
            }
        }

        // Validate warning fraction
        if self.warning_fraction <= 0.0 {
            anyhow::bail!(
                "warning_threshold_percent must be greater than 0 (got {})",
                self.warning_fraction * 100.0
            );
        }
        if self.warning_fraction >= 1.0 {
            anyhow::bail!(
                "warning_threshold_percent must be less than 100 (got {})",
                self.warning_fraction * 100.0
            );
        }

        Ok(())
    }

    pub fn ceiling_for(&self, category: ToolTimeoutCategory) -> Option<Duration> {
        match category {
            ToolTimeoutCategory::Default => self.default_ceiling,
            ToolTimeoutCategory::Pty => self.pty_ceiling.or(self.default_ceiling),
            ToolTimeoutCategory::Mcp => self.mcp_ceiling.or(self.default_ceiling),
        }
    }

    pub fn warning_fraction(&self) -> f32 {
        self.warning_fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_policy_derives_from_config() {
        let mut config = TimeoutsConfig::default();
        config.default_ceiling_seconds = 0;
        config.pty_ceiling_seconds = 600;
        config.mcp_ceiling_seconds = 90;
        config.warning_threshold_percent = 75;

        let policy = ToolTimeoutPolicy::from_config(&config);
        assert_eq!(policy.ceiling_for(ToolTimeoutCategory::Default), None);
        assert_eq!(
            policy.ceiling_for(ToolTimeoutCategory::Pty),
            Some(Duration::from_secs(600))
        );
        assert_eq!(
            policy.ceiling_for(ToolTimeoutCategory::Mcp),
            Some(Duration::from_secs(90))
        );
        assert!((policy.warning_fraction() - 0.75).abs() < f32::EPSILON);
    }
}
