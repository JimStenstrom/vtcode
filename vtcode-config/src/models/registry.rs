//! Central model registry with capabilities
//!
//! This module provides a structured database of AI models with their capabilities,
//! replacing fragile string matching throughout the codebase.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Model capabilities and characteristics
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Canonical model identifier
    pub id: &'static str,
    /// Provider (openai, anthropic, gemini, etc.)
    pub provider: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Context window size in tokens
    pub context_window: usize,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Cost tier (primary, small, reasoning)
    pub tier: ModelTier,
    /// Whether model is deprecated
    pub deprecated: bool,
    /// Replacement model if deprecated
    pub replacement: Option<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelCapabilities {
    /// Supports streaming responses
    pub streaming: bool,
    /// Supports tool/function calling
    pub tools: bool,
    /// Supports vision/image inputs
    pub vision: bool,
    /// Supports JSON mode
    pub json_mode: bool,
    /// Supports thinking/reasoning output
    pub thinking: bool,
    /// Supports prompt caching
    pub prompt_caching: bool,
    /// Maximum output tokens (if different from context window)
    pub max_output_tokens: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Most capable, expensive models
    Primary,
    /// Fast, cheap models
    Small,
    /// Special reasoning models (o1, o3, etc.)
    Reasoning,
}

/// Global model registry
pub static MODEL_REGISTRY: Lazy<HashMap<&'static str, ModelInfo>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // ==================== OpenAI Models ====================

    registry.insert(
        "gpt-4-turbo",
        ModelInfo {
            id: "gpt-4-turbo",
            provider: "openai",
            display_name: "GPT-4 Turbo",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gpt-4o",
        ModelInfo {
            id: "gpt-4o",
            provider: "openai",
            display_name: "GPT-4o",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(16_384),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gpt-4o-mini",
        ModelInfo {
            id: "gpt-4o-mini",
            provider: "openai",
            display_name: "GPT-4o Mini",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(16_384),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gpt-4",
        ModelInfo {
            id: "gpt-4",
            provider: "openai",
            display_name: "GPT-4",
            context_window: 8_192,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: false,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: Some("gpt-4-turbo"),
        },
    );

    registry.insert(
        "gpt-3.5-turbo",
        ModelInfo {
            id: "gpt-3.5-turbo",
            provider: "openai",
            display_name: "GPT-3.5 Turbo",
            context_window: 16_385,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: false,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gpt-3.5-turbo-16k",
        ModelInfo {
            id: "gpt-3.5-turbo-16k",
            provider: "openai",
            display_name: "GPT-3.5 Turbo 16K",
            context_window: 16_385,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: false,
                json_mode: true,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Small,
            deprecated: true,
            replacement: Some("gpt-3.5-turbo"),
        },
    );

    // OpenAI Reasoning Models
    registry.insert(
        "o1",
        ModelInfo {
            id: "o1",
            provider: "openai",
            display_name: "OpenAI o1",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false, // o1 doesn't support tools
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(100_000),
            },
            tier: ModelTier::Reasoning,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "o1-mini",
        ModelInfo {
            id: "o1-mini",
            provider: "openai",
            display_name: "OpenAI o1-mini",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false,
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(65_536),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "o1-preview",
        ModelInfo {
            id: "o1-preview",
            provider: "openai",
            display_name: "OpenAI o1-preview",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false,
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(32_768),
            },
            tier: ModelTier::Reasoning,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "o3",
        ModelInfo {
            id: "o3",
            provider: "openai",
            display_name: "OpenAI o3",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false,
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(100_000),
            },
            tier: ModelTier::Reasoning,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "o3-mini",
        ModelInfo {
            id: "o3-mini",
            provider: "openai",
            display_name: "OpenAI o3-mini",
            context_window: 128_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false,
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(65_536),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    // ==================== Anthropic Models ====================

    registry.insert(
        "claude-opus-4-20250514",
        ModelInfo {
            id: "claude-opus-4-20250514",
            provider: "anthropic",
            display_name: "Claude Opus 4",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false, // Uses prompt engineering
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(16_384),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "claude-sonnet-4-5-20250929",
        ModelInfo {
            id: "claude-sonnet-4-5-20250929",
            provider: "anthropic",
            display_name: "Claude Sonnet 4.5",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(16_384),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "claude-haiku-4-20250318",
        ModelInfo {
            id: "claude-haiku-4-20250318",
            provider: "anthropic",
            display_name: "Claude Haiku 4",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    // Legacy Claude models
    registry.insert(
        "claude-3-opus-20240229",
        ModelInfo {
            id: "claude-3-opus-20240229",
            provider: "anthropic",
            display_name: "Claude 3 Opus",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Primary,
            deprecated: true,
            replacement: Some("claude-opus-4-20250514"),
        },
    );

    registry.insert(
        "claude-3-5-sonnet-20241022",
        ModelInfo {
            id: "claude-3-5-sonnet-20241022",
            provider: "anthropic",
            display_name: "Claude 3.5 Sonnet",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Primary,
            deprecated: true,
            replacement: Some("claude-sonnet-4-5-20250929"),
        },
    );

    registry.insert(
        "claude-3-haiku-20240307",
        ModelInfo {
            id: "claude-3-haiku-20240307",
            provider: "anthropic",
            display_name: "Claude 3 Haiku",
            context_window: 200_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: true,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Small,
            deprecated: true,
            replacement: Some("claude-haiku-4-20250318"),
        },
    );

    // ==================== Google Gemini Models ====================

    registry.insert(
        "gemini-2.5-pro",
        ModelInfo {
            id: "gemini-2.5-pro",
            provider: "gemini",
            display_name: "Gemini 2.5 Pro",
            context_window: 2_000_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gemini-2.5-flash-lite",
        ModelInfo {
            id: "gemini-2.5-flash-lite",
            provider: "gemini",
            display_name: "Gemini 2.5 Flash Lite",
            context_window: 1_000_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Small,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gemini-2.0-flash-thinking-exp-01-21",
        ModelInfo {
            id: "gemini-2.0-flash-thinking-exp-01-21",
            provider: "gemini",
            display_name: "Gemini 2.0 Flash Thinking",
            context_window: 32_768,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false, // Thinking models may not support tools
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Reasoning,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gemini-2.0-pro",
        ModelInfo {
            id: "gemini-2.0-pro",
            provider: "gemini",
            display_name: "Gemini 2.0 Pro",
            context_window: 2_000_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "gemini-1.5-pro",
        ModelInfo {
            id: "gemini-1.5-pro",
            provider: "gemini",
            display_name: "Gemini 1.5 Pro",
            context_window: 2_000_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Primary,
            deprecated: true,
            replacement: Some("gemini-2.5-pro"),
        },
    );

    registry.insert(
        "gemini-1.5-flash",
        ModelInfo {
            id: "gemini-1.5-flash",
            provider: "gemini",
            display_name: "Gemini 1.5 Flash",
            context_window: 1_000_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: true,
                json_mode: false,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Small,
            deprecated: true,
            replacement: Some("gemini-2.5-flash-lite"),
        },
    );

    // ==================== DeepSeek Models ====================

    registry.insert(
        "deepseek-chat",
        ModelInfo {
            id: "deepseek-chat",
            provider: "deepseek",
            display_name: "DeepSeek Chat",
            context_window: 64_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: true,
                vision: false,
                json_mode: true,
                thinking: false,
                prompt_caching: false,
                max_output_tokens: Some(4096),
            },
            tier: ModelTier::Primary,
            deprecated: false,
            replacement: None,
        },
    );

    registry.insert(
        "deepseek-reasoner",
        ModelInfo {
            id: "deepseek-reasoner",
            provider: "deepseek",
            display_name: "DeepSeek Reasoner",
            context_window: 64_000,
            capabilities: ModelCapabilities {
                streaming: true,
                tools: false,
                vision: false,
                json_mode: false,
                thinking: true,
                prompt_caching: false,
                max_output_tokens: Some(8192),
            },
            tier: ModelTier::Reasoning,
            deprecated: false,
            replacement: None,
        },
    );

    registry
});

// ==================== Public API ====================

/// Get model info by ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_REGISTRY.get(model_id)
}

/// Check if model supports tool/function calling
pub fn model_supports_tools(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.capabilities.tools)
        .unwrap_or(true) // Default to true for unknown models
}

/// Check if model supports vision/image inputs
pub fn model_supports_vision(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.capabilities.vision)
        .unwrap_or(false)
}

/// Check if model supports streaming responses
pub fn model_supports_streaming(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.capabilities.streaming)
        .unwrap_or(true) // Default to true for unknown models
}

/// Check if model is a reasoning model (o1, o3, DeepSeek Reasoner, etc.)
pub fn is_reasoning_model(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.capabilities.thinking)
        .unwrap_or(false)
}

/// Check if model supports prompt caching
pub fn model_supports_prompt_caching(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.capabilities.prompt_caching)
        .unwrap_or(false)
}

/// Get context window size for a model
pub fn get_context_window(model_id: &str) -> Option<usize> {
    get_model_info(model_id).map(|info| info.context_window)
}

/// Get maximum output tokens for a model
pub fn get_max_output_tokens(model_id: &str) -> Option<usize> {
    get_model_info(model_id).and_then(|info| info.capabilities.max_output_tokens)
}

/// Get model tier (Primary, Small, Reasoning)
pub fn get_model_tier(model_id: &str) -> Option<ModelTier> {
    get_model_info(model_id).map(|info| info.tier)
}

/// Get provider for a model
pub fn get_model_provider(model_id: &str) -> Option<&'static str> {
    get_model_info(model_id).map(|info| info.provider)
}

/// List all models for a provider
pub fn models_for_provider(provider: &str) -> Vec<&'static ModelInfo> {
    MODEL_REGISTRY
        .values()
        .filter(|info| info.provider == provider && !info.deprecated)
        .collect()
}

/// Get all available model IDs
pub fn all_model_ids() -> Vec<&'static str> {
    MODEL_REGISTRY.keys().copied().collect()
}

/// Check if model is deprecated
pub fn is_model_deprecated(model_id: &str) -> bool {
    get_model_info(model_id)
        .map(|info| info.deprecated)
        .unwrap_or(false)
}

/// Get replacement model for deprecated model
pub fn get_replacement_model(model_id: &str) -> Option<&'static str> {
    get_model_info(model_id).and_then(|info| info.replacement)
}

/// Find model by fuzzy name match (for user input)
pub fn find_model_fuzzy(input: &str) -> Option<&'static str> {
    let input_lower = input.to_lowercase();

    // Exact match first (check if input matches a key in registry)
    if let Some(info) = MODEL_REGISTRY.get(input) {
        return Some(info.id);
    }

    // Display name match
    for (id, info) in MODEL_REGISTRY.iter() {
        if info.display_name.to_lowercase() == input_lower {
            return Some(id);
        }
    }

    // Partial match (be careful!)
    for (id, info) in MODEL_REGISTRY.iter() {
        if id.contains(&input_lower) || info.display_name.to_lowercase().contains(&input_lower) {
            return Some(id);
        }
    }

    None
}

/// Recommend best model for task requirements
pub fn recommend_model_for_task(
    provider: &str,
    requires_vision: bool,
    requires_tools: bool,
    prefer_cheap: bool,
) -> Option<&'static str> {
    let candidates: Vec<_> = MODEL_REGISTRY
        .iter()
        .filter(|(_, info)| {
            info.provider == provider
                && !info.deprecated
                && (!requires_vision || info.capabilities.vision)
                && (!requires_tools || info.capabilities.tools)
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by tier preference
    let best = if prefer_cheap {
        candidates
            .iter()
            .filter(|(_, info)| info.tier == ModelTier::Small)
            .next()
            .or_else(|| candidates.first())
    } else {
        candidates
            .iter()
            .filter(|(_, info)| info.tier == ModelTier::Primary)
            .next()
            .or_else(|| candidates.first())
    };

    best.map(|(id, _)| **id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_capabilities() {
        assert!(model_supports_tools("gpt-4-turbo"));
        assert!(!model_supports_tools("o1"));
        assert!(model_supports_vision("gpt-4o"));
        assert!(!model_supports_vision("gpt-3.5-turbo"));
    }

    #[test]
    fn test_reasoning_models() {
        assert!(is_reasoning_model("o1"));
        assert!(is_reasoning_model("o1-mini"));
        assert!(is_reasoning_model("o3"));
        assert!(is_reasoning_model("gemini-2.0-flash-thinking-exp-01-21"));
        assert!(is_reasoning_model("deepseek-reasoner"));
        assert!(!is_reasoning_model("gpt-4-turbo"));
        assert!(!is_reasoning_model("claude-opus-4-20250514"));
    }

    #[test]
    fn test_context_windows() {
        assert_eq!(get_context_window("gpt-4-turbo"), Some(128_000));
        assert_eq!(get_context_window("claude-opus-4-20250514"), Some(200_000));
        assert_eq!(get_context_window("gemini-2.5-pro"), Some(2_000_000));
        assert_eq!(get_context_window("o1"), Some(200_000));
    }

    #[test]
    fn test_fuzzy_matching() {
        assert_eq!(find_model_fuzzy("gpt-4-turbo"), Some("gpt-4-turbo"));
        assert_eq!(find_model_fuzzy("GPT-4 Turbo"), Some("gpt-4-turbo"));
        assert_eq!(find_model_fuzzy("unknown-model"), None);
    }

    #[test]
    fn test_model_recommendation() {
        let model = recommend_model_for_task(
            "openai",
            false, // no vision
            true,  // needs tools
            true,  // prefer cheap
        );
        // Should return a Small tier model - could be gpt-3.5-turbo or gpt-4o-mini
        assert!(model.is_some());
        let model_id = model.unwrap();
        let info = get_model_info(model_id).unwrap();
        assert_eq!(info.provider, "openai");
        assert_eq!(info.tier, ModelTier::Small);
        assert!(info.capabilities.tools);

        let model = recommend_model_for_task(
            "openai",
            false, // no vision
            true,  // needs tools
            false, // prefer expensive
        );
        // Should return a Primary tier model
        assert!(model.is_some());
        let model_id = model.unwrap();
        let info = get_model_info(model_id).unwrap();
        assert_eq!(info.provider, "openai");
        assert_eq!(info.tier, ModelTier::Primary);
        assert!(info.capabilities.tools);
    }

    #[test]
    fn test_provider_detection() {
        assert_eq!(get_model_provider("gpt-4o"), Some("openai"));
        assert_eq!(get_model_provider("claude-opus-4-20250514"), Some("anthropic"));
        assert_eq!(get_model_provider("gemini-2.5-pro"), Some("gemini"));
        assert_eq!(get_model_provider("deepseek-chat"), Some("deepseek"));
    }

    #[test]
    fn test_deprecation() {
        assert!(!is_model_deprecated("gpt-4-turbo"));
        assert!(is_model_deprecated("claude-3-opus-20240229"));
        assert_eq!(
            get_replacement_model("claude-3-opus-20240229"),
            Some("claude-opus-4-20250514")
        );
    }

    #[test]
    fn test_prompt_caching() {
        assert!(model_supports_prompt_caching("gpt-4-turbo"));
        assert!(model_supports_prompt_caching("claude-opus-4-20250514"));
        assert!(!model_supports_prompt_caching("o1"));
        assert!(!model_supports_prompt_caching("gemini-2.5-pro"));
    }

    #[test]
    fn test_models_for_provider() {
        let openai_models = models_for_provider("openai");
        assert!(!openai_models.is_empty());
        assert!(openai_models.iter().all(|m| m.provider == "openai"));
        assert!(openai_models.iter().all(|m| !m.deprecated));
    }

    #[test]
    fn test_all_model_ids() {
        let ids = all_model_ids();
        assert!(!ids.is_empty());
        assert!(ids.contains(&"gpt-4-turbo"));
        assert!(ids.contains(&"claude-opus-4-20250514"));
        assert!(ids.contains(&"gemini-2.5-pro"));
    }
}
