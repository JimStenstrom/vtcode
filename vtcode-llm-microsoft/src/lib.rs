//! Microsoft DirectLine v3 provider for VTCode LLM integration
//!
//! This crate provides a standalone Microsoft DirectLine provider that can be used
//! independently of VTCode's core infrastructure. It enables communication with
//! Azure Bot Service and Microsoft Bot Framework bots through the DirectLine API v3.
//!
//! ## Features
//!
//! - **DirectLine v3 Protocol**: Full support for Microsoft Bot Framework DirectLine API
//! - **Azure Integration**: Works with Azure Bot Service and Azure OpenAI Service backends
//! - **Stateful Conversations**: Manages conversation state and activity flow
//! - **OpenAI Compatible**: Delegates to OpenAI-compatible endpoints for many scenarios
//!
//! ## Configuration
//!
//! The provider can be configured using environment variables:
//!
//! - `DIRECTLINE_API_KEY` - DirectLine secret (required)
//! - `DIRECTLINE_BASE_URL` - Custom DirectLine endpoint (optional)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use vtcode_llm_microsoft::DirectLineProvider;
//! use vtcode_llm::LLMProvider;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a DirectLine provider with API key
//! let provider = DirectLineProvider::new("your-directline-secret".to_string());
//!
//! // Use with custom model
//! let provider = DirectLineProvider::with_model(
//!     "your-directline-secret".to_string(),
//!     "directline-gpt-4".to_string()
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ## DirectLine Protocol
//!
//! DirectLine v3 uses a REST-based protocol for bot communication:
//!
//! 1. **Start Conversation** - Establishes a new conversation session
//! 2. **Send Activity** - Posts user messages to the bot
//! 3. **Get Activities** - Retrieves bot responses
//! 4. **WebSocket Streaming** - Optional real-time message delivery
//!
//! ## Azure Bot Service Integration
//!
//! This provider works with:
//! - Azure Bot Service bots
//! - Bot Framework Composer bots
//! - Azure OpenAI Service bots
//! - Custom bot implementations
//!
//! For local development, use the Bot Framework Emulator.
//!
//! ## References
//!
//! - [DirectLine API v3 Documentation](https://docs.microsoft.com/azure/bot-service/rest-api/bot-framework-rest-direct-line-3-0)
//! - [Azure Bot Service](https://azure.microsoft.com/services/bot-services/)
//! - [Bot Framework SDK](https://github.com/microsoft/botframework-sdk)

// Re-export from vtcode-llm
pub use vtcode_llm::{
    config::ProviderConfig, AnyClient, BackendKind, DirectLineProvider, ErrorFormatter,
    ErrorReporter, LLMError, LLMProvider, LLMRequest, LLMResponse, LLMStream, LLMStreamEvent,
    Message, MessageRole, PathResolver, PathScope, TelemetrySink, Usage, WorkspacePaths,
    create_provider_with_config, error_display, get_factory, make_client,
};

#[cfg(feature = "mock")]
pub use vtcode_llm::StaticResponseClient;

#[cfg(feature = "telemetry")]
pub use vtcode_llm::{NoopStreamTelemetry, StreamTelemetry};

/// Configuration module for DirectLine provider
pub mod config {
    pub use vtcode_llm::config::*;

    /// DirectLine-specific configuration helper
    ///
    /// Creates a DirectLine provider configuration from environment variables
    /// or explicit values.
    pub fn directline_config_from_env() -> Option<super::ProviderConfig> {
        std::env::var("DIRECTLINE_API_KEY").ok().map(|api_key| {
            super::ProviderConfig {
                api_key: Some(api_key),
                base_url: std::env::var("DIRECTLINE_BASE_URL").ok(),
                model: Some("directline-gpt-4".to_string()),
                prompt_cache: None,
            }
        })
    }
}
