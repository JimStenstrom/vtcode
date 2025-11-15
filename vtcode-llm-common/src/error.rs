//! LLM error formatting utilities
//!
//! Provides simple error formatting for LLM providers without UI dependencies.

/// Format an LLM error message with provider name
///
/// # Arguments
///
/// * `provider` - Name of the provider (e.g., "DeepSeek", "Ollama")
/// * `error` - Error message to format
///
/// # Example
///
/// ```
/// use vtcode_llm_common::format_llm_error;
///
/// let msg = format_llm_error("DeepSeek", "Connection timeout");
/// assert_eq!(msg, "[DeepSeek] Connection timeout");
/// ```
pub fn format_llm_error(provider: &str, error: &str) -> String {
    format!("[{}] {}", provider, error)
}

/// Format an LLM warning message with provider name
pub fn format_llm_warning(provider: &str, warning: &str) -> String {
    format!("[{}] WARNING: {}", provider, warning)
}

/// Format an LLM success message with provider name
pub fn format_llm_success(provider: &str, message: &str) -> String {
    format!("[{}] {}", provider, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_llm_error() {
        let result = format_llm_error("TestProvider", "Test error");
        assert_eq!(result, "[TestProvider] Test error");
    }

    #[test]
    fn test_format_llm_warning() {
        let result = format_llm_warning("TestProvider", "Test warning");
        assert_eq!(result, "[TestProvider] WARNING: Test warning");
    }

    #[test]
    fn test_format_llm_success() {
        let result = format_llm_success("TestProvider", "Test success");
        assert_eq!(result, "[TestProvider] Test success");
    }
}
