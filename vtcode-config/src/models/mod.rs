//! Model configuration and provider management
//!
//! This module provides centralized enums for model identifiers and provider configurations,
//! replacing hardcoded model strings throughout the codebase for better maintainability.

pub mod capabilities;
pub mod model_id;
pub mod parser;
pub mod providers;
pub mod registry;

#[cfg(test)]
mod tests;

pub use model_id::ModelId;
pub use parser::ModelParseError;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// OpenRouter model metadata
#[derive(Clone, Copy)]
pub struct OpenRouterMetadata {
    pub(crate) id: &'static str,
    pub(crate) vendor: &'static str,
    pub(crate) display: &'static str,
    pub(crate) description: &'static str,
    pub(crate) efficient: bool,
    pub(crate) top_tier: bool,
    pub(crate) generation: &'static str,
    pub(crate) reasoning: bool,
    pub(crate) tool_call: bool,
}

/// Supported AI model providers
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Provider {
    /// Google Gemini models
    #[default]
    Gemini,
    /// OpenAI GPT models
    OpenAI,
    /// Anthropic Claude models
    Anthropic,
    /// DeepSeek native models
    DeepSeek,
    /// OpenRouter marketplace models
    OpenRouter,
    /// Local Ollama models
    Ollama,
    /// LM Studio local server (OpenAI-compatible)
    LmStudio,
    /// Moonshot.ai models
    Moonshot,
    /// xAI Grok models
    XAI,
    /// Z.AI GLM models
    ZAI,
    /// Microsoft Direct Line v3 (Bot Framework / M365 Copilot)
    Microsoft,
}

impl Provider {
    /// Get the default API key environment variable for this provider
    pub fn default_api_key_env(&self) -> &'static str {
        match self {
            Provider::Gemini => "GEMINI_API_KEY",
            Provider::OpenAI => "OPENAI_API_KEY",
            Provider::Anthropic => "ANTHROPIC_API_KEY",
            Provider::DeepSeek => "DEEPSEEK_API_KEY",
            Provider::OpenRouter => "OPENROUTER_API_KEY",
            Provider::Ollama => "OLLAMA_API_KEY",
            Provider::LmStudio => "LMSTUDIO_API_KEY",
            Provider::Moonshot => "MOONSHOT_API_KEY",
            Provider::XAI => "XAI_API_KEY",
            Provider::ZAI => "ZAI_API_KEY",
            Provider::Microsoft => "MICROSOFT_DIRECTLINE_SECRET",
        }
    }

    /// Get all supported providers
    pub fn all_providers() -> Vec<Provider> {
        vec![
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Gemini,
            Provider::DeepSeek,
            Provider::OpenRouter,
            Provider::Ollama,
            Provider::LmStudio,
            Provider::Moonshot,
            Provider::XAI,
            Provider::ZAI,
            Provider::Microsoft,
        ]
    }

    /// Human-friendly label for display purposes
    pub fn label(&self) -> &'static str {
        match self {
            Provider::Gemini => "Gemini",
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::DeepSeek => "DeepSeek",
            Provider::OpenRouter => "OpenRouter",
            Provider::Ollama => "Ollama",
            Provider::LmStudio => "LM Studio",
            Provider::Moonshot => "Moonshot",
            Provider::XAI => "xAI",
            Provider::ZAI => "Z.AI",
            Provider::Microsoft => "Microsoft Direct Line",
        }
    }

    /// Determine if the provider supports configurable reasoning effort for the model
    pub fn supports_reasoning_effort(&self, model: &str) -> bool {
        capabilities::supports_reasoning_effort(self, model)
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Gemini => write!(f, "gemini"),
            Provider::OpenAI => write!(f, "openai"),
            Provider::Anthropic => write!(f, "anthropic"),
            Provider::DeepSeek => write!(f, "deepseek"),
            Provider::OpenRouter => write!(f, "openrouter"),
            Provider::Ollama => write!(f, "ollama"),
            Provider::LmStudio => write!(f, "lmstudio"),
            Provider::Moonshot => write!(f, "moonshot"),
            Provider::XAI => write!(f, "xai"),
            Provider::ZAI => write!(f, "zai"),
            Provider::Microsoft => write!(f, "microsoft"),
        }
    }
}

impl FromStr for Provider {
    type Err = ModelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gemini" => Ok(Provider::Gemini),
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "deepseek" => Ok(Provider::DeepSeek),
            "openrouter" => Ok(Provider::OpenRouter),
            "ollama" => Ok(Provider::Ollama),
            "lmstudio" => Ok(Provider::LmStudio),
            "moonshot" => Ok(Provider::Moonshot),
            "xai" => Ok(Provider::XAI),
            "zai" => Ok(Provider::ZAI),
            "microsoft" => Ok(Provider::Microsoft),
            _ => Err(ModelParseError::InvalidProvider(s.to_string())),
        }
    }
}
