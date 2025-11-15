//! LLM generation parameters

/// Default temperature for main LLM responses (0.0-1.0)
/// Controls randomness/creativity: 0=deterministic, 1=random
/// 0.7 provides balanced creativity and consistency
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Default maximum tokens for main LLM generation responses
pub const DEFAULT_MAX_TOKENS: u32 = 2_000;

/// Default temperature for prompt refinement (0.0-1.0)
/// Lower than main temperature for more deterministic refinement
pub const DEFAULT_REFINE_TEMPERATURE: f32 = 0.3;

/// Default maximum tokens for prompt refinement
/// Prompts are shorter, so 800 tokens is typically sufficient
pub const DEFAULT_REFINE_MAX_TOKENS: u32 = 800;

/// Maximum tokens recommended for models with 256k context window
/// Leaves room for input context and token overhead
pub const MAX_TOKENS_256K_CONTEXT: u32 = 32_768;

/// Maximum tokens recommended for models with 128k context window
pub const MAX_TOKENS_128K_CONTEXT: u32 = 16_384;
