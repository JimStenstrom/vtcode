//! Other LLM provider model ID constants
//!
//! Includes: Z.AI, Microsoft, Moonshot, LM Studio, Ollama, MiniMax, xAI

// Z.AI models (direct API)
pub mod zai {
    pub const DEFAULT_MODEL: &str = "glm-4.6";
    pub const SUPPORTED_MODELS: &[&str] = &[
        "glm-4.6",
        "glm-4.5",
        "glm-4.5-air",
        "glm-4.5-x",
        "glm-4.5-airx",
        "glm-4.5-flash",
        "glm-4-32b-0414-128k",
    ];

    pub const GLM_4_6: &str = "glm-4.6";
    pub const GLM_4_5: &str = "glm-4.5";
    pub const GLM_4_5_AIR: &str = "glm-4.5-air";
    pub const GLM_4_5_X: &str = "glm-4.5-x";
    pub const GLM_4_5_AIRX: &str = "glm-4.5-airx";
    pub const GLM_4_5_FLASH: &str = "glm-4.5-flash";
    pub const GLM_4_32B_0414_128K: &str = "glm-4-32b-0414-128k";
}

// Microsoft Direct Line v3 models (Bot Framework / M365 Copilot)
pub mod microsoft {
    pub const DEFAULT_MODEL: &str = "directline-gpt-4";
    pub const SUPPORTED_MODELS: &[&str] = &[
        "directline-gpt-4",
        "directline-gpt-35-turbo",
        "directline-custom",
        "copilot-m365",
        "bot-framework",
    ];

    pub const DIRECTLINE_GPT_4: &str = "directline-gpt-4";
    pub const DIRECTLINE_GPT_35_TURBO: &str = "directline-gpt-35-turbo";
    pub const DIRECTLINE_CUSTOM: &str = "directline-custom";
    pub const COPILOT_M365: &str = "copilot-m365";
    pub const BOT_FRAMEWORK: &str = "bot-framework";
}

// Moonshot.ai models (direct API)
pub mod moonshot {
    pub const DEFAULT_MODEL: &str = "kimi-k2-turbo-preview";
    pub const SUPPORTED_MODELS: &[&str] = &[
        "kimi-k2-turbo-preview",
        "kimi-k2-thinking",
        "kimi-k2-thinking-turbo",
        "kimi-k2-0905-preview",
        "kimi-k2-0711-preview",
        "kimi-latest",
        "kimi-latest-8k",
        "kimi-latest-32k",
        "kimi-latest-128k",
    ];

    pub const KIMI_K2_TURBO_PREVIEW: &str = "kimi-k2-turbo-preview";
    pub const KIMI_K2_THINKING: &str = "kimi-k2-thinking";
    pub const KIMI_K2_THINKING_TURBO: &str = "kimi-k2-thinking-turbo";
    pub const KIMI_K2_0905_PREVIEW: &str = "kimi-k2-0905-preview";
    pub const KIMI_K2_0711_PREVIEW: &str = "kimi-k2-0711-preview";
    pub const KIMI_LATEST: &str = "kimi-latest";
    pub const KIMI_LATEST_8K: &str = "kimi-latest-8k";
    pub const KIMI_LATEST_32K: &str = "kimi-latest-32k";
    pub const KIMI_LATEST_128K: &str = "kimi-latest-128k";
}

// LM Studio models (OpenAI-compatible local server)
pub mod lmstudio {
    pub const DEFAULT_MODEL: &str = META_LLAMA_31_8B_INSTRUCT;
    pub const SUPPORTED_MODELS: &[&str] = &[
        META_LLAMA_3_8B_INSTRUCT,
        META_LLAMA_31_8B_INSTRUCT,
        QWEN25_7B_INSTRUCT,
        GEMMA_2_2B_IT,
        GEMMA_2_9B_IT,
        PHI_31_MINI_4K_INSTRUCT,
    ];

    pub const META_LLAMA_3_8B_INSTRUCT: &str = "lmstudio-community/meta-llama-3-8b-instruct";
    pub const META_LLAMA_31_8B_INSTRUCT: &str = "lmstudio-community/meta-llama-3.1-8b-instruct";
    pub const QWEN25_7B_INSTRUCT: &str = "lmstudio-community/qwen2.5-7b-instruct";
    pub const GEMMA_2_2B_IT: &str = "lmstudio-community/gemma-2-2b-it";
    pub const GEMMA_2_9B_IT: &str = "lmstudio-community/gemma-2-9b-it";
    pub const PHI_31_MINI_4K_INSTRUCT: &str = "lmstudio-community/phi-3.1-mini-4k-instruct";
}

// Ollama models
pub mod ollama {
    pub const DEFAULT_LOCAL_MODEL: &str = "gpt-oss:20b";
    pub const DEFAULT_CLOUD_MODEL: &str = "gpt-oss:120b-cloud";
    pub const DEFAULT_MODEL: &str = DEFAULT_LOCAL_MODEL;
    pub const SUPPORTED_MODELS: &[&str] = &[
        DEFAULT_LOCAL_MODEL,
        QWEN3_1_7B,
        DEFAULT_CLOUD_MODEL,
        GPT_OSS_20B_CLOUD,
        DEEPSEEK_V31_671B_CLOUD,
        KIMI_K2_1T_CLOUD,
        QWEN3_CODER_480B_CLOUD,
        GLM_46_CLOUD,
        MINIMAX_M2_CLOUD,
    ];

    /// Models that emit structured reasoning traces when `think` is enabled
    pub const REASONING_MODELS: &[&str] = &[
        GPT_OSS_20B,
        GPT_OSS_20B_CLOUD,
        GPT_OSS_120B_CLOUD,
        QWEN3_1_7B,
        DEEPSEEK_V31_671B_CLOUD,
        KIMI_K2_1T_CLOUD,
        QWEN3_CODER_480B_CLOUD,
        GLM_46_CLOUD,
        MINIMAX_M2_CLOUD,
    ];

    /// Models that require an explicit reasoning effort level instead of boolean toggle
    pub const REASONING_LEVEL_MODELS: &[&str] =
        &[GPT_OSS_20B, GPT_OSS_20B_CLOUD, GPT_OSS_120B_CLOUD];

    pub const GPT_OSS_20B: &str = DEFAULT_LOCAL_MODEL;
    pub const GPT_OSS_20B_CLOUD: &str = "gpt-oss:20b-cloud";
    pub const GPT_OSS_120B_CLOUD: &str = DEFAULT_CLOUD_MODEL;
    pub const QWEN3_1_7B: &str = "qwen3:1.7b";
    pub const DEEPSEEK_V31_671B_CLOUD: &str = "deepseek-v3.1:671b-cloud";
    pub const KIMI_K2_1T_CLOUD: &str = "kimi-k2:1t-cloud";
    pub const QWEN3_CODER_480B_CLOUD: &str = "qwen3-coder:480b-cloud";
    pub const GLM_46_CLOUD: &str = "glm-4.6:cloud";
    pub const MINIMAX_M2_CLOUD: &str = "minimax-m2:cloud";
}

// MiniMax models (Anthropic-compatible API, standalone provider)
pub mod minimax {
    pub const DEFAULT_MODEL: &str = MINIMAX_M2;
    pub const SUPPORTED_MODELS: &[&str] = &[MINIMAX_M2];
    pub const MINIMAX_M2: &str = "MiniMax-M2";
}

// xAI models
pub mod xai {
    pub const DEFAULT_MODEL: &str = "grok-4";
    pub const SUPPORTED_MODELS: &[&str] = &[
        "grok-4",
        "grok-4-mini",
        "grok-4-code",
        "grok-4-code-latest",
        "grok-4-vision",
    ];

    pub const GROK_4: &str = "grok-4";
    pub const GROK_4_MINI: &str = "grok-4-mini";
    pub const GROK_4_CODE: &str = "grok-4-code";
    pub const GROK_4_CODE_LATEST: &str = "grok-4-code-latest";
    pub const GROK_4_VISION: &str = "grok-4-vision";
}
