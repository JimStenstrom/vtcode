//! Anthropic model ID constants
//!
//! Updated for tool use best practices

// Standard model for straightforward tools - Sonnet 4 preferred for most use cases
pub const DEFAULT_MODEL: &str = "claude-sonnet-4-5";

pub const SUPPORTED_MODELS: &[&str] = &[
    "claude-opus-4-1-20250805", // Latest: Opus 4.1 (2025-08-05)
    "claude-sonnet-4-5",        // Latest: Sonnet 4.5 (2025-10-15)
    "claude-haiku-4-5",         // Latest: Haiku 4.5 (2025-10-15)
    "claude-sonnet-4-20250514", // Previous: Sonnet 4 (2025-05-14)
];

// Convenience constants for commonly used models
pub const CLAUDE_OPUS_4_1_20250805: &str = "claude-opus-4-1-20250805";
pub const CLAUDE_SONNET_4_5: &str = "claude-sonnet-4-5";
pub const CLAUDE_HAIKU_4_5: &str = "claude-haiku-4-5";
pub const CLAUDE_SONNET_4_20250514: &str = "claude-sonnet-4-20250514";

/// Models that accept the reasoning effort parameter
pub const REASONING_MODELS: &[&str] = &[
    CLAUDE_OPUS_4_1_20250805,
    CLAUDE_SONNET_4_5,
    CLAUDE_HAIKU_4_5,
    CLAUDE_SONNET_4_20250514,
];
