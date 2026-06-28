//! Token counting via tiktoken BPE tokenizer.
//!
//! All token estimation goes through [`tiktoken`]'s `cl100k_base` encoding
//! (GPT-4, GPT-3.5-turbo). BPE tokenizers are similar enough across providers
//! that this gives reasonable accuracy for Anthropic, Gemini, and others.
//!
//! Provider-reported exact token counts (from API responses) should always be
//! preferred when available. This module is for pre-call budget estimation and
//! offline token sizing where no provider response exists yet.

use std::sync::OnceLock;
use tiktoken::CoreBpe;

/// Return the process-global `cl100k_base` BPE instance.
///
/// Loaded once on first call; all subsequent calls return the same reference.
fn bpe() -> &'static CoreBpe {
    static BPE: OnceLock<&CoreBpe> = OnceLock::new();
    BPE.get_or_init(|| tiktoken::get_encoding("cl100k_base").expect("failed to load cl100k_base"))
}

/// Count the number of tokens in `text` using tiktoken BPE.
///
/// Returns 0 for empty strings.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    bpe().count(text)
}

/// Truncate `text` to at most `max_tokens` tokens.
///
/// Decodes the truncated token sequence back to text so the result is always
/// valid UTF-8 with no mid-token corruption. Falls back to byte-level
/// truncation if BPE decode fails (should not happen in practice).
pub fn truncate_to_tokens(text: &str, max_tokens: usize) -> String {
    if max_tokens == 0 || text.is_empty() {
        return String::new();
    }
    let tokens = bpe().encode_with_special_tokens(text);
    if tokens.len() <= max_tokens {
        return text.to_string();
    }
    bpe()
        .decode_to_string(&tokens[..max_tokens])
        .unwrap_or_else(|_| {
            // Byte-level fallback (should never fire for valid UTF-8).
            let end = (max_tokens * 4).min(text.len());
            let mut end = end;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            let mut result = text[..end].to_string();
            result.push_str("...");
            result
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_returns_zero() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(truncate_to_tokens("", 10), "");
    }

    #[test]
    fn count_is_reasonable() {
        let count = estimate_tokens("Hello, how are you today?");
        assert!(count >= 4 && count <= 12, "count={count}");
    }

    #[test]
    fn truncate_respects_limit() {
        let text = "the quick brown fox jumps over the lazy dog";
        let truncated = truncate_to_tokens(text, 5);
        let count = estimate_tokens(&truncated);
        assert!(count <= 5 + 1, "count={count} should be <= 6");
    }

    #[test]
    fn truncate_zero_returns_empty() {
        assert_eq!(truncate_to_tokens("hello", 0), "");
    }

    #[test]
    fn code_and_prose_tokenize() {
        let code = "fn main() { println!(\"hello\"); }";
        let prose = "the main function prints hello to console";
        assert!(estimate_tokens(code) > 0);
        assert!(estimate_tokens(prose) > 0);
    }

    #[test]
    fn json_tokenizes() {
        let json = r#"{"name":"test","value":123,"nested":{"key":"value"}}"#;
        let count = estimate_tokens(json);
        assert!(count >= 10 && count <= 40, "json count={count}");
    }
}
