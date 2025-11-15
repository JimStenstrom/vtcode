//! Context window management defaults and token estimation constants

/// Approximate character count per token when estimating context size
pub const CHAR_PER_TOKEN_APPROX: usize = 3;

/// Default maximum context window (in approximate tokens)
pub const DEFAULT_MAX_TOKENS: usize = 90_000;

/// Trim target as a percentage of the maximum token budget
pub const DEFAULT_TRIM_TO_PERCENT: u8 = 80;

/// Minimum allowed trim percentage (prevents overly aggressive retention)
pub const MIN_TRIM_RATIO_PERCENT: u8 = 60;

/// Maximum allowed trim percentage (prevents minimal trimming)
pub const MAX_TRIM_RATIO_PERCENT: u8 = 90;

/// Default number of recent turns to preserve verbatim
pub const DEFAULT_PRESERVE_RECENT_TURNS: usize = 12;

/// Minimum number of recent turns that must remain after trimming
pub const MIN_PRESERVE_RECENT_TURNS: usize = 6;

/// Maximum number of recent turns to keep when aggressively reducing context
pub const AGGRESSIVE_PRESERVE_RECENT_TURNS: usize = 8;

/// Enable semantic-aware compression heuristics by default
pub const DEFAULT_SEMANTIC_COMPRESSION_ENABLED: bool = false;

/// Enable tool-aware retention heuristics by default
pub const DEFAULT_TOOL_AWARE_RETENTION_ENABLED: bool = false;

/// Default maximum structural depth to preserve during semantic pruning
pub const DEFAULT_MAX_STRUCTURAL_DEPTH: usize = 3;

/// Default number of recent tool results to preserve when tool-aware retention is enabled
pub const DEFAULT_PRESERVE_RECENT_TOOLS: usize = 5;

/// Minimum structural depth allowed for semantic pruning
pub const MIN_STRUCTURAL_DEPTH: usize = 1;

/// Maximum structural depth allowed for semantic pruning to prevent runaway retention
pub const MAX_STRUCTURAL_DEPTH: usize = 12;

/// Minimum number of tool outputs to preserve when tool-aware retention is enabled
pub const MIN_PRESERVE_RECENT_TOOLS: usize = 1;

/// Maximum number of tool outputs to preserve when tool-aware retention is enabled
pub const MAX_PRESERVE_RECENT_TOOLS: usize = 24;

/// Maximum number of retry attempts when the provider signals context overflow
pub const CONTEXT_ERROR_RETRY_LIMIT: usize = 2;

/// Default semantic score for cached values (0-255 scale, typically)
pub const DEFAULT_SEMANTIC_CACHE_SCORE: u8 = 128;

/// Default semantic score for non-system messages
pub const DEFAULT_SEMANTIC_SCORE: u32 = 500;

/// Default token count estimate for message parts with multiple components
pub const DEFAULT_TOKENS_FOR_PARTS: usize = 256;

/// Approximate number of characters per token used for token estimation
pub const CHAR_PER_TOKEN_APPROXIMATION: usize = 4;

/// Default semantic score for system messages
pub const SYSTEM_MESSAGE_SEMANTIC_SCORE: u32 = 950;

/// Default semantic score for user messages
pub const USER_MESSAGE_SEMANTIC_SCORE: u32 = 850;

/// Scaling factor for semantic scores (typically scales from 0-255 to 0-1000 range)
pub const SEMANTIC_SCORE_SCALING_FACTOR: u32 = 4;

/// Conversion factor for percentage calculations (100.0)
pub const PERCENTAGE_CONVERSION_FACTOR: f64 = 100.0;

/// Decimal precision for context utilization percentage display
pub const CONTEXT_UTILIZATION_PRECISION: usize = 1;

/// Decimal precision for semantic value per token display
pub const SEMANTIC_VALUE_PRECISION: usize = 2;

/// Minimum token count to prevent division by zero
pub const MIN_TOKEN_COUNT: usize = 1;
