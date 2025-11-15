//! Google/Gemini model ID constants

/// Default model - using stable version for production reliability
pub const DEFAULT_MODEL: &str = "gemini-2.5-flash";

pub const SUPPORTED_MODELS: &[&str] = &[
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-preview-05-20",
];

/// Models that support thinking/reasoning capability
/// Based on: https://ai.google.dev/gemini-api/docs/models
/// All Gemini 2.5 models support the Thinking capability
pub const REASONING_MODELS: &[&str] = &[
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-preview-05-20",
];

/// Models that support context caching
/// Context caching reduces costs for repeated API calls with similar contexts
pub const CACHING_MODELS: &[&str] = &[
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-preview-05-20",
];

/// Models that support code execution
/// Code execution allows models to write and execute Python code
pub const CODE_EXECUTION_MODELS: &[&str] = &[
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
    "gemini-2.5-flash-preview-05-20",
];

// Convenience constants for commonly used models
pub const GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
pub const GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
pub const GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";
pub const GEMINI_2_5_FLASH_PREVIEW: &str = "gemini-2.5-flash-preview-05-20";
