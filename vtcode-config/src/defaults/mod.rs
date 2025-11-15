use std::time::Duration;
use crate::constants::{memory, http_client};

pub mod provider;
pub mod syntax_highlighting;

pub use provider::{
    ConfigDefaultsProvider, WorkspacePathsDefaults, current_config_defaults,
    install_config_defaults_provider, reset_to_default_config_defaults, with_config_defaults,
};
pub use syntax_highlighting::SyntaxHighlightingDefaults;

/// Context store defaults
pub struct ContextStoreDefaults;

impl ContextStoreDefaults {
    pub fn max_size() -> usize {
        memory::DEFAULT_CONTEXT_STORE_MAX_SIZE
    }
    pub fn compression() -> bool {
        true
    }

    // Constants for backward compatibility
    pub const MAX_CONTEXTS: usize = memory::DEFAULT_MAX_CONTEXTS;
    pub const AUTO_CLEANUP_DAYS: u32 = memory::DEFAULT_AUTO_CLEANUP_DAYS;
    pub const ENABLE_PERSISTENCE: bool = true;
    pub const COMPRESSION_ENABLED: bool = true;
    pub const STORAGE_DIR: &'static str = ".vtcode/context";
}

/// Performance defaults
pub struct PerformanceDefaults;

impl PerformanceDefaults {
    pub fn max_concurrent_operations() -> usize {
        http_client::DEFAULT_POOL_MAX_IDLE_PER_HOST
    }
    pub fn timeout_seconds() -> u64 {
        http_client::DEFAULT_REQUEST_TIMEOUT_SECS
    }
}

/// Scenario defaults
pub struct ScenarioDefaults;

impl ScenarioDefaults {
    pub fn max_scenarios() -> usize {
        10
    }
    pub fn default_timeout() -> u64 {
        300
    }

    // High performance scenario constants
    pub const HIGH_PERF_MAX_AGENTS: usize = 5;
    pub const HIGH_PERF_CONTEXT_WINDOW: usize = 200000;
    pub const HIGH_PERF_MAX_CONTEXTS: usize = memory::HIGH_PERF_MAX_CONTEXTS;

    // High quality scenario constants
    pub const HIGH_QUALITY_MAX_AGENTS: usize = 3;
    pub const HIGH_QUALITY_CONTEXT_WINDOW: usize = 150000;
    pub const HIGH_QUALITY_MAX_CONTEXTS: usize = memory::HIGH_QUALITY_MAX_CONTEXTS;

    // Balanced scenario constants
    pub const BALANCED_MAX_AGENTS: usize = 4;
    pub const BALANCED_CONTEXT_WINDOW: usize = 125000;
    pub const BALANCED_MAX_CONTEXTS: usize = memory::BALANCED_MAX_CONTEXTS;

    pub fn high_perf_timeout() -> Duration {
        Duration::from_secs(180)
    }
    pub fn high_quality_timeout() -> Duration {
        Duration::from_secs(600)
    }
    pub fn balanced_timeout() -> Duration {
        Duration::from_secs(300)
    }
}
