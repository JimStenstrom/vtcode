//! Model ID constants for all supported LLM providers

pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod openai;
pub mod openrouter;
pub mod others;

// Re-export provider modules for convenience
pub use others::{lmstudio, microsoft, minimax, moonshot, ollama, xai, zai};

// Re-export commonly used model constants for backwards compatibility
pub use anthropic::{
    CLAUDE_HAIKU_4_5, CLAUDE_OPUS_4_1_20250805, CLAUDE_SONNET_4_20250514, CLAUDE_SONNET_4_5,
};
pub use deepseek::{DEEPSEEK_CHAT, DEEPSEEK_REASONER};
pub use google::{
    GEMINI_2_5_FLASH, GEMINI_2_5_FLASH_LITE, GEMINI_2_5_FLASH_PREVIEW, GEMINI_2_5_PRO,
};
pub use minimax::MINIMAX_M2;
pub use moonshot::{
    KIMI_K2_0711_PREVIEW, KIMI_K2_0905_PREVIEW, KIMI_K2_THINKING, KIMI_K2_THINKING_TURBO,
    KIMI_K2_TURBO_PREVIEW, KIMI_LATEST, KIMI_LATEST_128K, KIMI_LATEST_32K, KIMI_LATEST_8K,
};
pub use openai::{CODEX_MINI_LATEST, GPT_5, GPT_5_CODEX, GPT_5_MINI, GPT_5_NANO};
pub use xai::{GROK_4, GROK_4_CODE, GROK_4_CODE_LATEST, GROK_4_MINI, GROK_4_VISION};

// Backwards compatibility aliases for prefixed constants
pub const CODEX_MINI: &str = openai::CODEX_MINI_LATEST;
pub const MOONSHOT_KIMI_K2_TURBO_PREVIEW: &str = moonshot::KIMI_K2_TURBO_PREVIEW;
pub const MOONSHOT_KIMI_K2_THINKING: &str = moonshot::KIMI_K2_THINKING;
pub const MOONSHOT_KIMI_K2_THINKING_TURBO: &str = moonshot::KIMI_K2_THINKING_TURBO;
pub const MOONSHOT_KIMI_K2_0905_PREVIEW: &str = moonshot::KIMI_K2_0905_PREVIEW;
pub const MOONSHOT_KIMI_K2_0711_PREVIEW: &str = moonshot::KIMI_K2_0711_PREVIEW;
pub const MOONSHOT_KIMI_LATEST: &str = moonshot::KIMI_LATEST;
pub const MOONSHOT_KIMI_LATEST_8K: &str = moonshot::KIMI_LATEST_8K;
pub const MOONSHOT_KIMI_LATEST_32K: &str = moonshot::KIMI_LATEST_32K;
pub const MOONSHOT_KIMI_LATEST_128K: &str = moonshot::KIMI_LATEST_128K;
pub const XAI_GROK_4: &str = xai::GROK_4;
pub const XAI_GROK_4_MINI: &str = xai::GROK_4_MINI;
pub const XAI_GROK_4_CODE: &str = xai::GROK_4_CODE;
pub const XAI_GROK_4_CODE_LATEST: &str = xai::GROK_4_CODE_LATEST;
pub const XAI_GROK_4_VISION: &str = xai::GROK_4_VISION;

// OpenRouter backwards compatibility
#[cfg(not(docsrs))]
pub const OPENROUTER_X_AI_GROK_CODE_FAST_1: &str = openrouter::X_AI_GROK_CODE_FAST_1;
#[cfg(docsrs)]
pub const OPENROUTER_X_AI_GROK_CODE_FAST_1: &str = "x-ai/grok-code-fast-1";

#[cfg(not(docsrs))]
pub const OPENROUTER_QWEN3_CODER: &str = openrouter::QWEN3_CODER;
#[cfg(docsrs)]
pub const OPENROUTER_QWEN3_CODER: &str = "qwen/qwen3-coder";

#[cfg(not(docsrs))]
pub const OPENROUTER_ANTHROPIC_CLAUDE_SONNET_4_5: &str = openrouter::ANTHROPIC_CLAUDE_SONNET_4_5;
#[cfg(docsrs)]
pub const OPENROUTER_ANTHROPIC_CLAUDE_SONNET_4_5: &str = "anthropic/claude-4-5-sonnet";
