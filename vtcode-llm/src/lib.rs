//! Prototype crate that re-exports VTCode's LLM integration layer while
//! introducing decoupled configuration traits for downstream consumers.
//!
//! The goal is to let external applications supply their own configuration
//! sources without depending on VTCode's dot-config structures. Consumers can
//! implement [`config::ProviderConfig`] for their own types and then convert
//! them into the factory configuration used internally by `vtcode-core`.
//!
//! This crate exposes feature flags so downstream projects can opt into
//! provider-specific exports, function calling helpers, or streaming telemetry
//! utilities without pulling additional API surface by default. Consult
//! `docs/vtcode_llm_environment.md` for a full overview of environment
//! variables, configuration patterns, and the optional mock client helpers.

pub mod config;

pub use vtcode_commons::{
    ErrorFormatter, ErrorReporter, PathResolver, PathScope, TelemetrySink, WorkspacePaths,
};

// Re-export from vtcode_llm_types
pub use vtcode_llm_types::{
    BackendKind, LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream, LLMStreamEvent,
    Message, MessageRole, Usage, FinishReason, ParallelToolConfig,
};

#[cfg(feature = "functions")]
pub use vtcode_llm_types::{
    FunctionCall, FunctionDefinition, SpecificFunctionChoice, SpecificToolChoice, ToolCall,
    ToolChoice, ToolDefinition,
};

// Re-export provider implementations
#[cfg(feature = "anthropic")]
pub use vtcode_llm_anthropic::AnthropicProvider;
#[cfg(feature = "google")]
pub use vtcode_llm_gemini::GeminiProvider;
#[cfg(feature = "microsoft")]
pub use vtcode_llm_microsoft::DirectLineProvider;
#[cfg(feature = "openai")]
pub use vtcode_llm_openai::OpenAIProvider;
#[cfg(feature = "openrouter")]
pub use vtcode_llm_openrouter::OpenRouterProvider;

// Note: DeepSeek, Moonshot, Ollama, XAI, ZAI providers still in vtcode-core
// TODO: Extract these providers to standalone crates to complete Phase 3

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "mock")]
pub use mock::StaticResponseClient;

pub mod providers {
    //! Provider-specific exports gated behind feature flags so consumers can
    //! depend on a minimal surface when only a subset of providers is needed.
    #[cfg(feature = "anthropic")]
    pub use vtcode_llm_anthropic::*;
    #[cfg(feature = "deepseek")]
    pub use vtcode_core::llm::providers::deepseek::*;
    #[cfg(feature = "google")]
    pub use vtcode_llm_gemini::*;
    #[cfg(feature = "microsoft")]
    pub use vtcode_llm_microsoft::*;
    #[cfg(feature = "moonshot")]
    pub use vtcode_core::llm::providers::moonshot::*;
    #[cfg(feature = "ollama")]
    pub use vtcode_core::llm::providers::ollama::*;
    #[cfg(feature = "openai")]
    pub use vtcode_core::llm::providers::openai::*;
    #[cfg(feature = "openrouter")]
    pub use vtcode_llm_openrouter::*;
    #[cfg(feature = "xai")]
    pub use vtcode_core::llm::providers::xai::*;
    #[cfg(feature = "zai")]
    pub use vtcode_core::llm::providers::zai::*;
}

// Re-export modular providers when their features are enabled
#[cfg(feature = "openai")]
pub mod modular {
    //! Modular provider implementations from standalone packages
    pub use vtcode_llm_openai::*;
}

#[cfg(feature = "telemetry")]
pub mod telemetry {
    //! Streaming telemetry helpers shared across provider implementations.
    pub use vtcode_core::llm::providers::shared::{
        NoopStreamTelemetry, StreamAssemblyError, StreamDelta, StreamFragment, StreamTelemetry,
        ToolCallBuilder, append_reasoning_segments, append_text_with_reasoning,
        finalize_tool_calls, update_tool_calls,
    };
}

#[cfg(feature = "telemetry")]
pub use telemetry::{NoopStreamTelemetry, StreamTelemetry};
