# Provider Architecture

This document describes the comprehensive architecture of the LLM provider system in VTCode, which enables seamless integration with multiple AI providers through a unified interface.

## Table of Contents

1. [Overview](#overview)
2. [Supported Providers](#supported-providers)
3. [Core Architecture](#core-architecture)
4. [Factory Pattern and Registration](#factory-pattern-and-registration)
5. [Provider Initialization](#provider-initialization)
6. [Message Format and Role Handling](#message-format-and-role-handling)
7. [Tool Calling System](#tool-calling-system)
8. [Streaming and Async Processing](#streaming-and-async-processing)
9. [Request Flow](#request-flow)
10. [Capability Detection](#capability-detection)
11. [Configuration System](#configuration-system)
12. [Error Handling](#error-handling)
13. [Provider-Specific Features](#provider-specific-features)
14. [Key Files and Locations](#key-files-and-locations)

## Overview

VTCode uses a **trait-based abstraction layer** to support 11 different LLM providers while handling provider-specific API differences transparently. The architecture follows several key principles:

- **Single `LLMProvider` trait** - All providers implement the same interface
- **Provider-agnostic message format** - Universal format with provider-specific conversion
- **Factory pattern** - Centralized provider instantiation and registration
- **Lazy initialization** - Global factory with thread-safe access
- **Modular design** - Shared utilities and common patterns across providers

## Supported Providers

The system currently supports 11 LLM providers:

| Provider | Module | Lines of Code | Key Features |
|----------|--------|---------------|--------------|
| **OpenAI** | `openai.rs` | 2,601 | Responses API, streaming, tool use, reasoning (o1 models) |
| **Anthropic** | `anthropic.rs` | 1,223 | Claude models, tool use, parallel tools, prompt caching |
| **Gemini** | `gemini.rs` | 1,374 | Google models, function calling, streaming, thinking |
| **OpenRouter** | `openrouter.rs` | 2,314 | Proxy for multiple providers, cross-provider support |
| **DeepSeek** | `deepseek.rs` | 866 | Reasoning models, tool calling |
| **ZAI** | `zai.rs` | 812 | Alibaba GLM models, tool use |
| **Ollama** | `ollama.rs` | 956 | Local models, community models |
| **Moonshot** | `moonshot.rs` | 582 | Kimi models, thinking capability |
| **Minimax** | `minimax.rs` | 484 | Chinese AI provider |
| **LMStudio** | `lmstudio.rs` | 270 | Local development models |
| **XAI** | `xai.rs` | 187 | Grok models, streaming |

**Location:** `/vtcode-core/src/llm/providers/`

## Core Architecture

### The LLMProvider Trait

The `LLMProvider` trait defines the contract that all providers must implement:

```rust
pub trait LLMProvider: Send + Sync {
    // Provider identification
    fn name(&self) -> &str;

    // Capability detection
    fn supports_streaming(&self) -> bool;
    fn supports_reasoning(&self, model: &str) -> bool;
    fn supports_reasoning_effort(&self, model: &str) -> bool;
    fn supports_tools(&self, model: &str) -> bool;
    fn supports_parallel_tool_config(&self, model: &str) -> bool;
    fn supports_structured_output(&self, model: &str) -> bool;
    fn supports_context_caching(&self, model: &str) -> bool;
    fn effective_context_size(&self, model: &str) -> usize;

    // Core operations
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError>;
    async fn stream(&self, request: LLMRequest) -> Result<LLMStream, LLMError>;

    // Model support
    fn supported_models(&self) -> Vec<String>;
    fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError>;
}
```

**Location:** `/vtcode-core/src/llm/provider.rs` (1,077 lines)

### Universal Message Types

The architecture uses universal message types that are converted to provider-specific formats:

#### Message Roles

```rust
pub enum MessageRole {
    System,    // Handled differently by each provider
    User,      // Standard across all providers
    Assistant, // Called "model" in Gemini
    Tool,      // Not supported in Anthropic (converted to User)
}
```

**Provider-specific role mappings:**
- **OpenAI**: `"system"`, `"user"`, `"assistant"`, `"tool"`
- **Anthropic**: `"user"`, `"assistant"` (tool responses as user messages)
- **Gemini**: `"user"`, `"model"` (system as systemInstruction parameter)

#### Message Content

```rust
pub enum MessageContent {
    Text(String),              // Simple text content
    Parts(Vec<ContentPart>),   // Multi-part content with images
}

pub enum ContentPart {
    Text { text: String },
    Image { data: String, mime_type: String, ... },
}
```

### Request and Response Types

#### LLMRequest

```rust
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
    pub tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub parallel_tool_config: Option<ParallelToolConfig>,
    pub reasoning_effort: Option<ReasoningEffortLevel>,
}
```

#### LLMResponse

```rust
pub struct LLMResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<Usage>,
    pub finish_reason: FinishReason,
    pub reasoning: Option<String>,
    pub reasoning_details: Option<Vec<serde_json::Value>>,
}

pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error(String),
}
```

## Factory Pattern and Registration

### LLMFactory

The factory provides centralized provider instantiation and management:

```rust
pub struct LLMFactory {
    providers: HashMap<String, Box<dyn Fn(ProviderConfig) -> Box<dyn LLMProvider> + Send + Sync>>,
}

pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub prompt_cache: Option<PromptCachingConfig>,
}
```

**Location:** `/vtcode-core/src/llm/factory.rs` (416 lines)

### Registration System

Providers are registered using a macro-based pattern:

```rust
register_providers!(
    factory,
    "gemini" => GeminiProvider,
    "openai" => OpenAIProvider,
    "anthropic" => AnthropicProvider,
    "deepseek" => DeepSeekProvider,
    "openrouter" => OpenRouterProvider,
    "zai" => ZAIProvider,
    "moonshot" => MoonshotProvider,
    "ollama" => OllamaProvider,
    "xai" => XAIProvider,
    "lmstudio" => LMStudioProvider,
    "minimax" => MinimaxProvider,
);
```

Each provider implements the `BuiltinProvider` trait:

```rust
trait BuiltinProvider: LLMProvider {
    fn build_from_config(config: ProviderConfig) -> Box<dyn LLMProvider>;
}
```

### Global Factory Access

The factory uses lazy initialization with thread-safe access:

```rust
static FACTORY: LazyLock<Mutex<LLMFactory>> = LazyLock::new(|| Mutex::new(LLMFactory::new()));

pub fn get_factory() -> &'static Mutex<LLMFactory> { &FACTORY }
pub fn create_provider_for_model(...) -> Result<Box<dyn LLMProvider>, LLMError>
pub fn create_provider_with_config(...) -> Result<Box<dyn LLMProvider>, LLMError>
```

### Provider Auto-Detection

The factory can automatically detect the provider from model names:

```rust
pub fn provider_from_model(&self, model: &str) -> Option<String> {
    // Pattern matching:
    // "gpt-*" or "o1" -> openai
    // "claude-*" -> anthropic
    // "gemini" or "palm" -> gemini
    // "grok-*" or "xai-*" -> xai
    // "deepseek-*" -> deepseek
    // etc.
}
```

## Provider Initialization

### Macro-Based Constructor Generation

All providers use the `impl_provider_constructors!` macro to generate standard constructors:

```rust
macro_rules! impl_provider_constructors {
    (default_model: $default_model:expr, resolve_fn: $resolve_fn:path) => {
        // Generates three constructors:
        pub fn new(api_key: String) -> Self { ... }
        pub fn with_model(api_key: String, model: String) -> Self { ... }
        pub fn from_config(
            api_key: Option<String>,
            model: Option<String>,
            base_url: Option<String>,
            prompt_cache: Option<PromptCachingConfig>,
        ) -> Self { ... }
    };
}
```

**Location:** `/vtcode-core/src/llm/providers/shared/mod.rs` (496 lines)

### Builder Pattern

The `ProviderBuilder` provides a fluent interface for configuration:

```rust
pub struct ProviderBuilder<T: Clone + Default> {
    pub api_key: String,
    pub http_client: HttpClient,
    pub base_url: String,
    pub model: String,
    pub prompt_cache_enabled: bool,
    pub prompt_cache_settings: T,
}

// Usage:
let builder = ProviderBuilder::new(api_key, model, base_url)
    .with_base_url(override_url, env_var_name)
    .with_prompt_cache(cache_config, select_settings, enabled)
    .with_http_client(client)
    .build()
```

### Example: Anthropic Provider Initialization

```rust
pub struct AnthropicProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_enabled: bool,
    prompt_cache_settings: AnthropicPromptCacheSettings,
}

impl AnthropicProvider {
    impl_provider_constructors!(
        default_model: models::anthropic::DEFAULT_MODEL,
        resolve_fn: resolve_model
    );

    fn with_model_internal(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        let builder = ProviderBuilder::new(api_key, model, urls::ANTHROPIC_API_BASE)
            .with_base_url(base_url, Some(env_vars::ANTHROPIC_BASE_URL))
            .with_prompt_cache(
                prompt_cache,
                |providers| &providers.anthropic,
                |cfg, provider_settings| cfg.enabled && provider_settings.enabled,
            );

        Self {
            api_key: builder.api_key,
            http_client: builder.http_client,
            base_url: builder.base_url,
            model: builder.model,
            prompt_cache_enabled: builder.prompt_cache_enabled,
            prompt_cache_settings: builder.prompt_cache_settings,
        }
    }
}
```

## Message Format and Role Handling

### Role Conversion Strategy

Each provider handles message roles differently. The architecture automatically converts between formats:

#### OpenAI
- Supports all four roles: `system`, `user`, `assistant`, `tool`
- Tool messages must include `tool_call_id`
- System messages placed at the beginning

#### Anthropic
- Supports only `user` and `assistant` roles
- System messages hoisted to `system` parameter
- Tool responses converted to user messages with special formatting
- Alternating user/assistant message pattern required

#### Gemini
- Uses `user` and `model` (not assistant)
- System messages converted to `systemInstruction` parameter
- Tool responses as user messages with `functionResponse` format
- Function calls embedded in model messages as `functionCall`

### Message Validation

Each provider validates messages before sending:

```rust
fn validate_request(&self, request: &LLMRequest) -> Result<(), LLMError> {
    // 1. Check model is supported
    if !self.supported_models().contains(&request.model) {
        return Err(LLMError::InvalidRequest(...));
    }

    // 2. Validate message roles for provider
    for message in &request.messages {
        message.validate_for_provider(self.name())?;
    }

    // 3. Validate tool definitions
    if let Some(tools) = &request.tools {
        for tool in tools {
            tool.validate()?;
        }
    }

    Ok(())
}
```

## Tool Calling System

### Tool Definition

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

### Tool Choice Configuration

```rust
pub enum ToolChoice {
    Auto,                              // Model decides
    None,                              // No tools allowed
    Any,                               // Force at least one tool call
    Specific(SpecificToolChoice),      // Force specific tool
}

pub struct SpecificToolChoice {
    pub name: String,
}
```

### Parallel Tool Calling

```rust
pub struct ParallelToolConfig {
    pub enabled: bool,
    pub max_parallel: Option<usize>,
}
```

**Provider support:**
- **OpenAI**: Full support with `parallel_tool_calls` parameter
- **Anthropic**: Supported with special configuration
- **Gemini**: Limited support through function calling
- **Others**: Varies by provider

### Tool Call Building

During streaming, tool calls are built incrementally:

```rust
pub struct ToolCallBuilder {
    id: Option<String>,
    name: String,
    arguments: String,
}

impl ToolCallBuilder {
    pub fn add_delta(&mut self, delta: &ToolCallDelta) {
        if let Some(id) = &delta.id {
            self.id = Some(id.clone());
        }
        if let Some(name) = &delta.function.name {
            self.name.push_str(name);
        }
        if let Some(args) = &delta.function.arguments {
            self.arguments.push_str(args);
        }
    }

    pub fn build(self) -> Result<ToolCall, LLMError> {
        // Validate and parse JSON arguments
    }
}
```

**Location:** `/vtcode-core/src/llm/providers/shared/mod.rs`

## Streaming and Async Processing

### Stream Types

```rust
pub type LLMStream = Pin<Box<dyn futures::Stream<Item = Result<LLMStreamEvent, LLMError>> + Send>>;

pub enum LLMStreamEvent {
    Token { delta: String },
    Reasoning { delta: String },
    Completed { response: LLMResponse },
}
```

### SSE Stream Parsing

The architecture provides utilities for parsing Server-Sent Events:

```rust
// Extract data payload from SSE stream
pub fn extract_data_payload(line: &str) -> Option<&str> {
    line.strip_prefix("data: ")
}

// Find SSE message boundaries
pub fn find_sse_boundary(buffer: &str) -> Option<usize> {
    // Detects \n\n or \r\n\r\n
}
```

### Reasoning Buffer

For models that support reasoning (e.g., OpenAI o1), a specialized buffer manages reasoning output:

```rust
pub struct ReasoningBuffer {
    text: String,
    last_chunk: Option<String>,
}

impl ReasoningBuffer {
    pub fn add_chunk(&mut self, chunk: String) {
        self.text.push_str(&chunk);
        self.last_chunk = Some(chunk);
    }

    pub fn get_last_chunk(&self) -> Option<&str> {
        self.last_chunk.as_deref()
    }

    pub fn finalize(self) -> String {
        self.text
    }
}
```

### Stream Telemetry

Providers can emit telemetry events during streaming:

```rust
pub trait StreamTelemetry: Send + Sync {
    fn on_content_delta(&self, _delta: &str) {}
    fn on_reasoning_delta(&self, _delta: &str) {}
    fn on_tool_call_delta(&self) {}
}
```

## Request Flow

### Standard Request Flow

```
User Request
    ↓
LLMRequest (universal format)
    ↓
validate_request() - Ensure model/roles supported
    ↓
convert_to_provider_request() - Format-specific conversion
    ↓
HTTP POST to provider API
    ↓
Parse provider response
    ↓
convert_from_provider_response() - Back to universal format
    ↓
LLMResponse
    ↓
User code
```

### Streaming Request Flow

```
HTTP Stream
    ↓
Parse SSE events
    ↓
extract_data_payload()
    ↓
find_sse_boundary()
    ↓
Provider-specific parsing
    ↓
LLMStreamEvent stream
    ↓
ToolCallBuilder accumulation
    ↓
ReasoningBuffer processing
    ↓
Emit Token/Reasoning/Completed events
```

### Implementation Example

```rust
#[async_trait]
impl LLMProvider for MyProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        // 1. Validate request
        self.validate_request(&request)?;

        // 2. Convert to provider format
        let provider_request = self.convert_to_provider_format(&request)?;

        // 3. Make HTTP request
        let response = self.http_client
            .post(&self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&provider_request)
            .send()
            .await?;

        // 4. Handle errors
        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await?);
        }

        // 5. Parse and convert response
        let provider_response: ProviderResponse = response.json().await?;
        self.convert_from_provider_format(provider_response)
    }
}
```

## Capability Detection

### ProviderCapabilities

The system provides automatic capability detection:

```rust
pub struct ProviderCapabilities {
    pub provider_name: String,
    pub streaming: bool,
    pub advanced_reasoning: bool,
    pub reasoning_effort: bool,
    pub tools: bool,
    pub parallel_tools: bool,
    pub structured_output: bool,
    pub context_caching: bool,
    pub model: String,
    pub context_size: usize,
}

impl ProviderCapabilities {
    pub fn detect(provider: &dyn LLMProvider, model: &str) -> Self {
        Self {
            provider_name: provider.name().to_string(),
            streaming: provider.supports_streaming(),
            advanced_reasoning: provider.supports_reasoning(model),
            reasoning_effort: provider.supports_reasoning_effort(model),
            tools: provider.supports_tools(model),
            parallel_tools: provider.supports_parallel_tool_config(model),
            structured_output: provider.supports_structured_output(model),
            context_caching: provider.supports_context_caching(model),
            model: model.to_string(),
            context_size: provider.effective_context_size(model),
        }
    }
}
```

**Location:** `/vtcode-core/src/llm/capabilities.rs` (151 lines)

## Configuration System

### Configuration Hierarchy

Configuration is resolved in the following order (highest priority first):

1. **Explicit parameters** - Direct function arguments (api_key, base_url, model)
2. **Environment variables** - `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc.
3. **Configuration file** - `vtcode.toml`
4. **Provider defaults** - Hard-coded default values

### Prompt Caching Configuration

```rust
pub struct PromptCachingConfig {
    pub enabled: bool,
    pub cache_dir: String,
    pub max_entries: usize,
    pub max_age_days: u64,
    pub enable_auto_cleanup: bool,
    pub min_quality_threshold: f64,
    pub providers: ProviderPromptCachingConfig,
}

pub struct ProviderPromptCachingConfig {
    pub openai: OpenAIPromptCacheSettings,
    pub anthropic: AnthropicPromptCacheSettings,
    pub gemini: GeminiPromptCacheSettings,
    pub openrouter: OpenRouterPromptCacheSettings,
    pub moonshot: MoonshotPromptCacheSettings,
    pub xai: XAIPromptCacheSettings,
    pub deepseek: DeepSeekPromptCacheSettings,
    pub zai: ZaiPromptCacheSettings,
}
```

**Location:** `/vtcode-config/src/core/prompt_cache.rs`

### Base URL Resolution

```rust
fn override_base_url(
    default: &str,
    override_config: Option<String>,
    env_var_name: Option<&str>,
) -> String {
    // Priority:
    // 1. Config override
    if let Some(override_url) = override_config {
        return override_url;
    }

    // 2. Environment variable
    if let Some(env_var) = env_var_name {
        if let Ok(env_url) = std::env::var(env_var) {
            return env_url;
        }
    }

    // 3. Default
    default.to_string()
}
```

## Error Handling

### LLMError Types

```rust
pub enum LLMError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),
}
```

### Error Detection Patterns

Each provider implements custom error detection logic:

```rust
async fn handle_error_response(&self, response: Response) -> Result<(), LLMError> {
    let status = response.status();

    match status.as_u16() {
        401 | 403 => Err(LLMError::Authentication(
            "Invalid API key or insufficient permissions".to_string()
        )),
        429 => Err(LLMError::RateLimit),
        400 => {
            let error_body = response.text().await?;
            Err(LLMError::InvalidRequest(error_body))
        },
        _ => {
            let error_body = response.text().await?;
            Err(LLMError::Provider(error_body))
        }
    }
}
```

### Rate Limit Detection

Providers detect rate limits through:
- **Status codes**: 429
- **Error message patterns**: "rate limit", "quota exceeded"
- **Response headers**: `X-RateLimit-*` headers

## Provider-Specific Features

### OpenAI Provider

**File:** `/vtcode-core/src/llm/providers/openai.rs` (2,601 lines)

**Unique features:**
- **Responses API** - Advanced structured output formatting
- **Reasoning Models** - o1-preview, o1-mini with reasoning output
- **Streaming with Deltas** - Incremental token streaming
- **Prompt Caching** - Automatic cost optimization
- **Model Auto-Detection** - Fallback to Responses API when needed
- **Reasoning Effort Levels** - low, medium, high for o1 models

### Anthropic Provider

**File:** `/vtcode-core/src/llm/providers/anthropic.rs` (1,223 lines)

**Unique features:**
- **Prompt Caching** - 5-minute and 1-hour TTL options
- **Extended Cache TTL** - 1-hour caching with `anthropic-beta` header
- **Parallel Tool Use** - Multiple simultaneous tool calls
- **Cache Breakpoints** - Up to 4 cache points per request
- **Minimax Fallback** - Special handling for Minimax M2 model
- **Tool Choice Modes** - auto, any, tool (specific), none

### Gemini Provider

**File:** `/vtcode-core/src/llm/providers/gemini.rs` (1,374 lines)

**Unique features:**
- **Thinking Capability** - Reasoning-capable models
- **System Instructions** - Separate from conversation history
- **Function Calling** - Integrated tool support
- **Cache Modes** - Custom or semantic caching
- **Streaming** - Structured SSE responses

### OpenRouter Provider

**File:** `/vtcode-core/src/llm/providers/openrouter.rs` (2,314 lines)

**Unique features:**
- **Multi-Provider Proxy** - Access to multiple providers through one API
- **Cross-Provider Support** - Unified interface for various backends
- **Model Routing** - Automatic routing to appropriate provider
- **Fallback Support** - Alternative models when primary unavailable

### DeepSeek Provider

**File:** `/vtcode-core/src/llm/providers/deepseek.rs` (866 lines)

**Unique features:**
- **Reasoning Models** - Advanced reasoning capabilities
- **Tool Calling** - Native tool support
- **Cost Optimization** - Efficient token usage

### Local Providers (Ollama, LMStudio)

**Files:**
- `/vtcode-core/src/llm/providers/ollama.rs` (956 lines)
- `/vtcode-core/src/llm/providers/lmstudio.rs` (270 lines)

**Unique features:**
- **Local Execution** - No external API calls
- **Community Models** - Support for custom and community models
- **No API Keys** - Works without authentication
- **Flexible Base URLs** - Configurable local endpoints

## Key Files and Locations

### Core Components

| Component | Location | Lines | Description |
|-----------|----------|-------|-------------|
| Core Trait | `/vtcode-core/src/llm/provider.rs` | 1,077 | LLMProvider trait, message types, request/response |
| Factory | `/vtcode-core/src/llm/factory.rs` | 416 | Provider registration and creation |
| Shared Utilities | `/vtcode-core/src/llm/providers/shared/mod.rs` | 496 | Macros, streaming, tool call building |
| Capabilities | `/vtcode-core/src/llm/capabilities.rs` | 151 | Feature detection system |
| Client | `/vtcode-core/src/llm/client.rs` | - | Unified client interface |

### Configuration

| Component | Location | Description |
|-----------|----------|-------------|
| Prompt Cache Config | `/vtcode-config/src/core/prompt_cache.rs` | Cache configuration types |
| Model Constants | `/vtcode-config/src/constants.rs` | Model lists and default values |
| Type Definitions | `/vtcode-llm-types/src/provider.rs` | Shared type definitions |

### Provider Implementations

| Provider | Location | Lines |
|----------|----------|-------|
| OpenAI | `/vtcode-core/src/llm/providers/openai.rs` | 2,601 |
| OpenRouter | `/vtcode-core/src/llm/providers/openrouter.rs` | 2,314 |
| Gemini | `/vtcode-core/src/llm/providers/gemini.rs` | 1,374 |
| Anthropic | `/vtcode-core/src/llm/providers/anthropic.rs` | 1,223 |
| Ollama | `/vtcode-core/src/llm/providers/ollama.rs` | 956 |
| DeepSeek | `/vtcode-core/src/llm/providers/deepseek.rs` | 866 |
| ZAI | `/vtcode-core/src/llm/providers/zai.rs` | 812 |
| Moonshot | `/vtcode-core/src/llm/providers/moonshot.rs` | 582 |
| Minimax | `/vtcode-core/src/llm/providers/minimax.rs` | 484 |
| LMStudio | `/vtcode-core/src/llm/providers/lmstudio.rs` | 270 |
| XAI | `/vtcode-core/src/llm/providers/xai.rs` | 187 |

### Testing

| Component | Location | Description |
|-----------|----------|-------------|
| Provider Tests | `/tests/llm_providers_test.rs` | Comprehensive provider tests |
| Integration Tests | `/tests/llm_provider_integration.rs` | Integration testing |
| Performance Benchmarks | `/vtcode-core/benches/provider_performance.rs` | Performance testing |

## Architecture Patterns

The provider architecture employs several key design patterns:

1. **Factory Pattern** - Centralized provider instantiation via `LLMFactory`
2. **Trait-Based Abstraction** - `LLMProvider` trait for polymorphism
3. **Builder Pattern** - `ProviderBuilder` for flexible configuration
4. **Macro-Based Code Generation** - `impl_provider_constructors!` for DRY principle
5. **Lazy Initialization** - `LazyLock` for global factory instance
6. **Strategy Pattern** - Different conversion strategies per provider
7. **Decorator Pattern** - `ProviderCapabilities` for feature detection
8. **Stream Processing** - Async streaming with event aggregation
9. **Configuration Hierarchy** - Layered configuration resolution
10. **Adapter Pattern** - Converting between universal and provider-specific formats

## Best Practices

### Adding a New Provider

To add a new provider to the system:

1. **Create provider module** in `/vtcode-core/src/llm/providers/`
2. **Implement `LLMProvider` trait** with all required methods
3. **Use standard constructor macros** via `impl_provider_constructors!`
4. **Implement message conversion** from universal to provider format
5. **Handle provider-specific errors** with appropriate error types
6. **Add to factory registration** in `factory.rs`
7. **Add configuration types** in `/vtcode-config/src/core/`
8. **Write comprehensive tests** in `/tests/`
9. **Update documentation** including this file

### Provider Implementation Checklist

- [ ] Implement all `LLMProvider` trait methods
- [ ] Handle message role conversion correctly
- [ ] Support streaming if provider offers it
- [ ] Implement tool calling if supported
- [ ] Add prompt caching support if available
- [ ] Handle reasoning output for reasoning models
- [ ] Implement proper error handling and detection
- [ ] Add capability detection methods
- [ ] Support configuration via env vars and config file
- [ ] Write unit and integration tests
- [ ] Add performance benchmarks
- [ ] Document provider-specific features

## Conclusion

The VTCode provider architecture provides a robust, extensible foundation for integrating multiple LLM providers. Through its trait-based abstraction, factory pattern, and comprehensive utilities, it enables:

- **Seamless provider switching** without code changes
- **Transparent handling** of provider-specific differences
- **Advanced features** like streaming, tool calling, and reasoning
- **Easy addition** of new providers
- **Consistent interface** for all provider interactions

This architecture ensures VTCode can adapt to the evolving LLM landscape while maintaining stability and ease of use.
