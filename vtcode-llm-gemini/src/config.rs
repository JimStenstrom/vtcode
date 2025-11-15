//! Gemini configuration and model capabilities
//!
//! This module provides configuration utilities and model capability
//! information for the Gemini provider.

use vtcode_config::constants::models;

/// Check if model supports context caching
pub fn supports_caching(model: &str) -> bool {
    models::google::CACHING_MODELS.contains(&model)
}

/// Check if model supports code execution
pub fn supports_code_execution(model: &str) -> bool {
    models::google::CODE_EXECUTION_MODELS.contains(&model)
}

/// Get maximum input token limit for a model
pub fn max_input_tokens(model: &str) -> usize {
    if model.contains("2.5") || model.contains("2.0") {
        1_048_576 // 1M tokens for Gemini 2.x models
    } else {
        // Conservative default for unknown models
        32_768
    }
}

/// Get maximum output token limit for a model
pub fn max_output_tokens(model: &str) -> usize {
    if model.contains("2.5") {
        65_536 // 65K tokens for Gemini 2.5 models
    } else if model.contains("2.0") {
        8_192 // 8K tokens for Gemini 2.0 models
    } else {
        8_192 // Conservative default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_output_tokens_gemini_2_5() {
        assert_eq!(max_output_tokens("gemini-2.5-flash"), 65_536);
        assert_eq!(max_output_tokens("gemini-2.5-pro"), 65_536);
    }

    #[test]
    fn test_max_output_tokens_gemini_2_0() {
        assert_eq!(max_output_tokens("gemini-2.0-flash"), 8_192);
    }

    #[test]
    fn test_max_input_tokens() {
        assert_eq!(max_input_tokens("gemini-2.5-flash"), 1_048_576);
        assert_eq!(max_input_tokens("gemini-2.0-flash"), 1_048_576);
    }
}
