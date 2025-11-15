//! OpenRouter model ID constants
//!
//! Models are extensible via vtcode.toml

#[cfg(not(docsrs))]
include!(concat!(env!("OUT_DIR"), "/openrouter_constants.rs"));

#[cfg(docsrs)]
pub mod docsrs_fallback {
    pub const SUPPORTED_MODELS: &[&str] = &[];
    pub const REASONING_MODELS: &[&str] = &[];
    pub const TOOL_UNAVAILABLE_MODELS: &[&str] = &[];

    // Define the constants that are referenced elsewhere to avoid compile errors
    pub const X_AI_GROK_CODE_FAST_1: &str = "x-ai/grok-code-fast-1";
    pub const QWEN3_CODER: &str = "qwen/qwen3-coder";
    pub const ANTHROPIC_CLAUDE_SONNET_4_5: &str = "anthropic/claude-4-5-sonnet";

    pub mod vendor {
        pub mod openrouter {
            pub const MODELS: &[&str] = &[];
        }
    }
}

#[cfg(docsrs)]
pub use docsrs_fallback::*;
