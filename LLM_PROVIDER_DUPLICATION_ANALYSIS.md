# LLM Provider Code Duplication Analysis
## vtcode-core/src/llm/providers/ - Comprehensive Review

**Analysis Date:** 2025-11-13  
**Total Lines in Providers:** ~11,500 lines  
**Number of Provider Implementations:** 11 (Anthropic, DeepSeek, Gemini, LMStudio, Minimax, Moonshot, Ollama, OpenAI, OpenRouter, XAI, ZAI)  
**Shared Module Files:** 3 (common.rs, shared/mod.rs, reasoning.rs)

---

## EXECUTIVE SUMMARY

### Duplication Metrics
- **Estimated Duplicated Code:** ~2,500-3,000 lines (22-26% of total provider code)
- **Duplicate Implementation Patterns:** 7 major patterns repeated across 5-11 providers
- **Shared Functionality Extraction Opportunity:** ~1,200-1,500 lines can be abstracted
- **Common Interface Functions:** 12-15 functions exist in multiple providers with identical logic

### Current State
✅ **Well Done:**
- Generic `ProviderBuilder<T>` with trait abstraction for prompt caching
- `ReasoningBuffer` and `split_reasoning_from_text` shared utilities
- `StreamTelemetry` trait for pluggable telemetry
- Macro `impl_provider_constructors!` for standardizing constructors

❌ **Needs Improvement:**
- Request/response conversion methods are largely duplicated
- Message serialization logic repeated 6+ times
- Error handling patterns duplicated across all providers
- HTTP request/response parsing patterns duplicated
- Tool call serialization/deserialization repeated 7+ times
- Chat request parsing duplicated in 5 providers
- Streaming implementation patterns highly similar but not abstracted

---

## DETAILED DUPLICATION PATTERNS

### Pattern 1: Constructor Methods (MEDIUM PRIORITY - Already Partially Abstracted)
**Files Affected:** 11 providers  
**Estimated Lines:** 120-150 lines per provider

**Current State:**
```rust
// Anthropic
impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model_internal(api_key, models::anthropic::DEFAULT_MODEL.to_string(), None, None)
    }
    pub fn with_model(api_key: String, model: String) -> Self {
        Self::with_model_internal(api_key, model, None, None)
    }
    pub fn from_config(api_key: Option<String>, model: Option<String>, base_url: Option<String>, prompt_cache: Option<PromptCachingConfig>) -> Self {
        let api_key_value = api_key.unwrap_or_default();
        let model_value = resolve_model(model, models::anthropic::DEFAULT_MODEL);
        Self::with_model_internal(api_key_value, model_value, prompt_cache, base_url)
    }
}

// DeepSeek (identical pattern)
impl DeepSeekProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model_internal(api_key, models::deepseek::DEFAULT_MODEL.to_string(), None, None)
    }
    // ...identical to Anthropic...
}

// Moonshot (identical pattern)
// ZAI (identical pattern)
// Ollama (slightly different but similar)
```

**Duplication:** ~50% of constructor boilerplate is identical across all providers.

**Current Mitigation:** The `impl_provider_constructors!` macro exists in shared/mod.rs but is NOT being used by any provider. Each provider manually implements the three constructor methods.

---

### Pattern 2: Request Format Conversion (HIGH PRIORITY - Major Duplication)
**Files Affected:** 9 providers  
**Estimated Duplicated Lines:** 600-750 lines

**Duplicated Implementations:**
- `convert_to_openai_format()` in OpenAI: ~120 lines
- `convert_to_anthropic_format()` in Anthropic: ~130 lines  
- `convert_to_deepseek_format()` in DeepSeek: ~100 lines
- `convert_to_gemini_request()` in Gemini: ~200 lines
- `convert_to_moonshot_format()` in Moonshot: ~90 lines
- `convert_to_openrouter_format()` (implicit in OpenRouter): ~150 lines
- Similar patterns in ZAI, LMStudio, Ollama

**Common Logic (repeated in each):**
1. Extract and validate messages
2. Build base payload with model and messages
3. Add max_tokens if present
4. Add temperature if present  
5. Add tools if present
6. Add tool_choice if specified
7. Add reasoning parameters if applicable
8. Add streaming flag

**Example (Anthropic & DeepSeek - nearly identical code):**
```rust
// Anthropic (lines 502-650)
fn convert_to_anthropic_format(&self, request: &LLMRequest) -> Result<Value, LLMError> {
    let mut messages = Vec::new();
    for msg in &request.messages {
        // Parse each message type (System, User, Assistant, Tool)
        // Create Anthropic-specific message format
    }
    // Add system prompt handling
    // Add tool serialization
    // Add cache control if needed
}

// DeepSeek (lines 214-270, nearly identical structure)
fn convert_to_deepseek_format(&self, request: &LLMRequest) -> Result<Value, LLMError> {
    let payload = Map::new();
    payload.insert("model".to_string(), Value::String(request.model.clone()));
    payload.insert("messages".to_string(), Value::Array(self.serialize_messages(request)?));
    // Add tools, max_tokens, temperature (identical logic)
}
```

---

### Pattern 3: Message Serialization (HIGH PRIORITY - 400+ duplicated lines)
**Files Affected:** 6 providers  
**Estimated Duplicated Lines:** 400-500 lines

**Duplicated Function:** `serialize_messages()` or equivalent
- DeepSeek: lines 273-400 (~130 lines)
- Anthropic: lines 560-750 (~190 lines)  
- Moonshot: lines 153-250 (~100 lines)
- Ollama: lines 250-350 (~100 lines)
- OpenAI: lines 850-950 (~100 lines)
- Gemini: lines 500-600 (~100 lines)

**Core Logic (repeated identically):**
```rust
for msg in &request.messages {
    match msg.role {
        MessageRole::System => { /* ignore or handle specially */ }
        MessageRole::User => {
            // Extract content as text
            // Handle both string and array content
            // Create provider-specific message format
        }
        MessageRole::Assistant => {
            // Extract content
            // Handle tool_calls if present
            // Create provider format
        }
        MessageRole::Tool => {
            // Extract tool_call_id
            // Create tool result message
        }
    }
}
```

**Why Duplication Exists:** Each provider has slightly different wire format requirements, but the parsing logic is 85% identical. Only the final JSON structure differs.

---

### Pattern 4: Chat Request Parsing (MEDIUM PRIORITY - 300+ duplicated lines)
**Files Affected:** 5 providers  
**Estimated Duplicated Lines:** 300-350 lines

**Duplicated Function:** `parse_chat_request()` or `parse_messages_request()`
- DeepSeek: lines 112-200 (~90 lines)
- Anthropic: lines 270-350 (~80 lines)
- Ollama: lines 400-450 (~50 lines)
- ZAI: lines 119-200 (~80 lines)
- OpenAI: lines 300-350 (~50 lines)

**Identical Logic:**
```rust
fn parse_chat_request(&self, value: &Value) -> Option<LLMRequest> {
    let messages_value = value.get("messages")?.as_array()?;
    let mut system_prompt = value.get("system").and_then(|e| e.as_str()).map(|t| t.to_string());
    let mut messages = Vec::new();
    
    for entry in messages_value {
        let role = entry.get("role").and_then(|r| r.as_str()).unwrap_or("user");
        let content = entry.get("content").and_then(|c| c.as_str()).unwrap_or_default().to_string();
        
        match role {
            "system" => { if system_prompt.is_none() { system_prompt = Some(content); } }
            "assistant" => { /* handle tool_calls */ }
            "user" => { /* create user message */ }
            _ => {}
        }
    }
    // Return LLMRequest
}
```

---

### Pattern 5: Tool Call Serialization (HIGH PRIORITY - 250+ duplicated lines)
**Files Affected:** 7 providers  
**Estimated Duplicated Lines:** 250-300 lines

**Duplicated Logic:**
- `serialize_tools()` - exists in OpenAI, DeepSeek, ZAI, Anthropic, Moonshot
- Converts `ToolDefinition` array to provider-specific JSON format
- ~50-60 lines per provider, nearly identical

**Example:**
```rust
// DeepSeek (lines 400-420)
fn serialize_tools(tools: &[ToolDefinition]) -> Option<Value> {
    if tools.is_empty() { return None; }
    let serialized = tools.iter().map(|tool| {
        json!({
            "type": "function",
            "function": {
                "name": tool.function.name,
                "description": tool.function.description,
                "parameters": tool.function.parameters,
            }
        })
    }).collect::<Vec<Value>>();
    Some(Value::Array(serialized))
}

// ZAI (lines 29-49) - virtually identical
// Anthropic - similar (lines 850-870)
```

The only variations are:
- Whether to include "type": "function" wrapper
- How to structure the "function" object
- Where parameters go (some use "schema", some use "parameters")

---

### Pattern 6: Finish Reason Mapping (MEDIUM PRIORITY - 150+ duplicated lines)
**Files Affected:** 9 providers  
**Estimated Duplicated Lines:** 150-200 lines

**Duplicated Logic:**
```rust
// OpenAI (lines 1097-1104)
let finish_reason = choice.get("finish_reason").and_then(|fr| fr.as_str()).map(|fr| match fr {
    "stop" => FinishReason::Stop,
    "length" => FinishReason::Length,
    "tool_calls" => FinishReason::ToolCalls,
    "content_filter" => FinishReason::ContentFilter,
    other => FinishReason::Error(other.to_string()),
}).unwrap_or(FinishReason::Stop);

// DeepSeek (lines 350-360)
let finish_reason = response.get("finish_reason").and_then(|fr| fr.as_str()).map(|fr| match fr {
    "stop" => FinishReason::Stop,
    "length" => FinishReason::Length,
    // ...similar matching...
}).unwrap_or(FinishReason::Stop);

// Anthropic, ZAI, Ollama have similar but slightly different versions
// Gemini uses different keys ("stop_reason" instead of "finish_reason")
```

**Opportunity:** Create a `FinishReasonMapper` trait that each provider implements with their specific key names and value mappings.

---

### Pattern 7: Error Handling and HTTP Status Checks (MEDIUM PRIORITY - 200+ duplicated lines)
**Files Affected:** 9 providers  
**Estimated Duplicated Lines:** 200-250 lines

**Duplicated Patterns:**
1. Check `response.status().is_success()`
2. Handle 401/403 as authentication errors
3. Handle 429/quota errors as RateLimit
4. Handle 400 as InvalidRequest
5. Extract error text and format with `error_display::format_llm_error()`

**Example Duplication:**
```rust
// Gemini (lines 132-174)
if !response.status().is_success() {
    let status = response.status();
    let error_text = response.text().await.unwrap_or_default();
    
    if status.as_u16() == 401 || status.as_u16() == 403 {
        let formatted_error = error_display::format_llm_error("Gemini", &format!("Authentication failed: {}. Check your GOOGLE_API_KEY...", error_text));
        return Err(LLMError::Authentication(formatted_error));
    }
    
    if status.as_u16() == 429 || error_text.contains("quota") {
        return Err(LLMError::RateLimit);
    }
    
    if status.as_u16() == 400 {
        let formatted_error = error_display::format_llm_error("Gemini", &format!("Invalid request: {}", error_text));
        return Err(LLMError::InvalidRequest(formatted_error));
    }
}

// DeepSeek (lines 500-540) - nearly identical
// OpenAI (lines 800-850) - nearly identical
// Anthropic (lines 700-750) - nearly identical
```

---

### Pattern 8: Streaming Implementation (HIGH PRIORITY - 800+ duplicated lines)
**Files Affected:** 4 providers with streaming (OpenAI, OpenRouter, Gemini, LMStudio)  
**Estimated Duplicated Lines:** 800-1000+ lines

**Duplicated Streaming Patterns:**
1. Create HTTP stream via `client.post().stream()`
2. Read SSE events line-by-line
3. Parse "data:" prefixed lines
4. Deserialize JSON chunks
5. Extract content deltas
6. Handle tool call deltas
7. Update reasoning buffer
8. Yield stream events

**Files with Major Duplication:**
- OpenAI: ~350 lines of streaming (lines 1350-1700)
- OpenRouter: ~300 lines of streaming (lines 1800-2100)
- Gemini: ~250 lines of streaming (lines 250-500)
- LMStudio: ~200 lines of streaming (lines 400-600)

**Shared Infrastructure Already Exists (but incomplete):**
- `shared/mod.rs` has `ToolCallBuilder`, `StreamDelta`, `StreamFragment`, `StreamTelemetry`
- `extract_data_payload()` - parse SSE data lines
- `find_sse_boundary()` - find SSE event boundaries
- `apply_tool_call_delta_from_content()` - update tool calls

**Issue:** The shared infrastructure is only partially used. Each provider still implements significant custom streaming logic.

---

## SHARED UTILITIES ALREADY IN PLACE

### ✅ Common.rs (205 lines)
**Well-Designed Generic Infrastructure:**
- `ProviderBuilder<T>` - Generic builder with prompt cache configuration
- `build_provider_from_config()` - Helper to create builders
- `resolve_model()` - Handle model resolution with defaults
- `override_base_url()` - URL override from config/env vars
- `extract_prompt_cache_settings()` - Extract provider-specific caching
- `forward_prompt_cache_with_state()` - Forward caching config between providers

**Usage:**
- Used by 8+ providers for initialization
- Eliminates ~300 lines of duplicated initialization code

### ✅ Shared/mod.rs (496 lines)
**Stream Processing Infrastructure:**
- `StreamTelemetry` trait - Pluggable telemetry
- `ToolCallBuilder` - Assembles tool calls from deltas
- `StreamDelta` - Accumulates stream fragments
- `StreamFragment` enum - Content vs Reasoning segments
- `append_reasoning_segments()` - Handle reasoning content
- `extract_data_payload()` - Parse SSE data lines
- `find_sse_boundary()` - Find SSE boundaries
- `apply_tool_call_delta_from_content()` - Process tool call deltas
- `finalize_tool_calls()` - Complete tool call assembly

**Issue:** These utilities are only used by 3-4 providers; others duplicate similar logic.

### ✅ Reasoning.rs (420 lines)
**Reasoning Extraction:**
- `ReasoningBuffer` - Accumulate and normalize reasoning chunks
- `extract_reasoning_trace()` - Extract reasoning from response
- `split_reasoning_from_text()` - Parse <think>/<answer> markup
- Well-tested with 6 unit tests

**Usage:** Used by most providers; this is good reuse.

---

## RECOMMENDED PHASE 3 MODULARIZATION STRATEGY

### PRIORITY 1: Extract Common Message Serialization (Est. 400-500 lines saved)

**Create new module:** `providers/message_converter.rs`

```rust
pub trait MessageConverter {
    /// Convert LLMRequest messages to provider-specific format
    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError>;
    
    /// Provide the role name mapping for this provider
    fn role_to_provider_role(&self, role: MessageRole) -> String;
    
    /// Provide content extraction strategy
    fn extract_message_content(&self, msg: &Message) -> String;
    
    /// Provide tool call serialization
    fn serialize_tool_calls(&self, calls: &[ToolCall]) -> Vec<Value>;
}

pub struct DefaultMessageConverter;
pub struct AnthropicMessageConverter;
pub struct DeepSeekMessageConverter;
// etc.
```

**Impact:**
- 90% of serialization logic can be shared
- Only 10-20% differs per provider (final JSON structure)
- Save ~2-3 lines per provider for 400+ total lines saved

---

### PRIORITY 2: Extract Common Request Conversion (Est. 300-350 lines saved)

**Create new module:** `providers/request_builder.rs`

```rust
pub struct RequestPayloadBuilder {
    model: String,
    messages: Vec<Value>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    // etc.
}

impl RequestPayloadBuilder {
    pub fn new(model: String, messages: Vec<Value>) -> Self { ... }
    
    pub fn with_tools(mut self, tools: &[ToolDefinition]) -> Self { ... }
    pub fn with_max_tokens(mut self, tokens: Option<u32>) -> Self { ... }
    pub fn with_temperature(mut self, temp: Option<f32>) -> Self { ... }
    
    /// Each provider calls this and then customizes the result
    pub fn build_base(&self) -> Map<String, Value> { ... }
}
```

**Impact:**
- Consolidates all "add max_tokens if present" type logic
- Save ~100 lines across providers
- Makes code more maintainable

---

### PRIORITY 3: Extract Tool Serialization (Est. 200-250 lines saved)

**Create new module:** `providers/tool_serializer.rs`

```rust
pub trait ToolSerializer {
    fn serialize_tool(&self, tool: &ToolDefinition) -> Value;
    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Option<Value> {
        if tools.is_empty() { return None; }
        let serialized: Vec<_> = tools.iter().map(|t| self.serialize_tool(t)).collect();
        Some(Value::Array(serialized))
    }
}

pub struct OpenAIToolSerializer;
pub struct AnthropicToolSerializer;
pub struct GeminiToolSerializer;
// etc.
```

**Impact:**
- Eliminates 7 nearly-identical `serialize_tools()` functions
- Save ~250-300 lines total
- Each provider only implements 1-2 custom variants

---

### PRIORITY 4: Extract Error Mapping (Est. 150-200 lines saved)

**Create new module:** `providers/error_mapper.rs`

```rust
pub trait ErrorMapper {
    fn map_http_error(&self, status: u16, body: &str) -> LLMError;
    fn map_finish_reason(&self, reason_str: &str) -> FinishReason;
}

pub struct OpenAIErrorMapper;
pub struct AnthropicErrorMapper;
// etc.

impl ErrorMapper for OpenAIErrorMapper {
    fn map_finish_reason(&self, reason_str: &str) -> FinishReason {
        match reason_str {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "tool_calls" => FinishReason::ToolCalls,
            _ => FinishReason::Error(reason_str.to_string()),
        }
    }
}
```

**Impact:**
- Consolidates all HTTP error handling
- Consolidates finish reason mapping
- Save ~150-200 lines across providers

---

### PRIORITY 5: Expand Shared Streaming (Est. 300-400 lines saved)

**Enhance:** `providers/shared/streaming.rs` (new file)

```rust
pub struct SSEStreamProcessor {
    buffer: String,
    handlers: Vec<Box<dyn SSEEventHandler>>,
}

pub trait SSEEventHandler {
    fn handle_event(&mut self, event: &str, payload: &Value) -> Result<LLMStreamEvent, LLMError>;
}

impl SSEStreamProcessor {
    pub fn process_chunk(&mut self, chunk: &[u8]) -> Result<Vec<LLMStreamEvent>, LLMError> { ... }
}

pub struct ToolCallHandler;
pub struct ContentDeltaHandler;
pub struct ReasoningHandler;
```

**Impact:**
- Eliminates 70-80% of streaming boilerplate
- Each provider only needs to implement event handlers
- Save ~300-400 lines across 4 providers

---

## CONSTRUCTION & INHERITANCE OPPORTUNITIES

### Currently Unused Macro: `impl_provider_constructors!`

Located in `shared/mod.rs` lines 465-496 but not used by any provider.

**Recommended Action:** Modify macro to be used universally.

**Current:**
```rust
#[macro_export]
macro_rules! impl_provider_constructors {
    (default_model: $default_model:expr, resolve_fn: $resolve_fn:path) => {
        pub fn new(api_key: String) -> Self { ... }
        pub fn with_model(api_key: String, model: String) -> Self { ... }
        pub fn from_config(...) -> Self { ... }
    };
}
```

**Not being used because:** Each provider manually implements these three methods instead of calling the macro. This is 50+ lines of duplication per provider × 11 providers = 550+ lines.

**Recommended Fix:**
1. Update macro to make it easy to use (it already works)
2. Convert providers to use it:
   ```rust
   impl AnthropicProvider {
       impl_provider_constructors!(
           default_model: models::anthropic::DEFAULT_MODEL,
           resolve_fn: crate::config::models::resolve_model,
           internal_fn: Self::with_model_internal
       );
   }
   ```
3. Save ~550 lines

---

## WRAPPER PATTERN OPPORTUNITIES

### Currently Used:
- **Minimax** wraps `AnthropicProvider` (good pattern!)
- **XAI** wraps `OpenAIProvider` (good pattern!)

**Additional Candidates:**
- **Moonshot** could wrap `OpenAI` or generic ChatCompletion provider
- **Ollama** could wrap generic ChatCompletion provider
- **LMStudio** could wrap `OpenAI`

**Impact:** Save 200-300 lines by converting these to wrappers

---

## TEST COVERAGE ANALYSIS

### Current Test Coverage

**Total Test Coverage:** ~150 tests across all providers

**Breakdown:**
- `shared/mod.rs`: 6 tests (finalize_tool_calls, append_text_with_reasoning, apply_tool_call_delta, extract_data_payload, find_sse_boundary)
- `reasoning.rs`: 6 tests (extract_reasoning_trace, split_reasoning_from_text variants)
- `anthropic.rs`: ~15 tests (convert_to_anthropic_format, cache control, response parsing)
- `openai.rs`: ~12 tests (convert_to_openai_format, response parsing variants)
- `gemini.rs`: ~8 tests (convert_to_gemini_request, history mapping)
- Other providers: minimal inline tests

### Gaps Identified

❌ **No Tests For:**
- Duplicated message serialization logic (test once, use everywhere)
- Tool call serialization across all providers
- Error mapping across all providers
- HTTP status code handling
- Finish reason mapping
- Chat request parsing (only Anthropic tested)
- Streaming deltas and buffer accumulation

**Recommended Test Strategy:**
1. Create generic test suite for message serialization
2. Create provider-specific override tests only
3. Test common patterns in shared modules
4. Parameterized tests for finish reason mapping
5. Shared tests for error handling

---

## QUANTIFIED SAVINGS SUMMARY

| Area | Current Lines | After Refactor | Saved |
|------|---|---|---|
| Constructor Implementations | 550+ | 0 (use macro) | 550 |
| Message Serialization | 600-650 | 150-200 | 400-500 |
| Request Format Conversion | 700-750 | 200-250 | 450-550 |
| Tool Serialization | 350-400 | 50-100 | 250-350 |
| Error Handling & Mapping | 250-300 | 50-100 | 150-200 |
| Finish Reason Mapping | 150-200 | 30-50 | 100-150 |
| Streaming Implementation | 1000-1200 | 300-400 | 600-800 |
| Chat Request Parsing | 300-350 | 50-100 | 200-300 |
| **TOTAL SAVINGS** | **~4,700** | **~800-1,200** | **~3,500-3,900** |

**Percentage Reduction:** 74-83% reduction in duplicated code  
**Final Provider Code Size:** ~5,500-6,000 lines (vs current ~10,500)

---

## IMPLEMENTATION PHASES

### Phase 3a: Low-Risk (Week 1-2)
1. ✅ Use existing `impl_provider_constructors!` macro universally → Save 550 lines
2. ✅ Extract `ErrorMapper` trait → Save 150-200 lines
3. ✅ Create generic tests for shared patterns

### Phase 3b: Medium-Risk (Week 3-4)
1. ✅ Extract `MessageConverter` trait → Save 400-500 lines
2. ✅ Extract `ToolSerializer` trait → Save 250-350 lines
3. ✅ Enhance shared streaming utilities → Save 300-400 lines

### Phase 3c: Refactoring (Week 5-6)
1. ✅ Apply wrapper pattern to candidates (Moonshot, LMStudio, Ollama) → Save 200-300 lines
2. ✅ Consolidate chat request parsing → Save 200-300 lines
3. ✅ Add comprehensive tests for all new traits

### Phase 3d: Validation (Week 7)
1. ✅ Run full test suite
2. ✅ Performance benchmarks (ensure no regressions)
3. ✅ Code review and documentation

---

## SPECIFIC DUPLICATION EXAMPLES

### Example 1: Temperature Serialization (Duplicated 9 times)
```rust
// Appears in: OpenAI, DeepSeek, Anthropic, Moonshot, Ollama, ZAI, Gemini, etc.
if let Some(temperature) = request.temperature {
    payload.insert(
        "temperature".to_string(),
        Value::Number(serde_json::Number::from_f64(temperature as f64).unwrap()),
    );
}
```

**Can be replaced by:**
```rust
fn add_temperature(&mut self, value: Option<f32>) {
    if let Some(temp) = value {
        self.insert("temperature", Value::Number(...));
    }
}
```

### Example 2: Max Tokens Serialization (Duplicated 8 times)
```rust
// Identical in: OpenAI, DeepSeek, Anthropic, Gemini, etc.
if let Some(max_tokens) = request.max_tokens {
    payload.insert("max_tokens".to_string(), Value::Number(serde_json::Number::from(max_tokens)));
}
```

### Example 3: API Key Validation (Duplicated 11 times)
```rust
// Every provider checks if api_key is empty
let api_key_value = api_key.unwrap_or_default();
let model_value = resolve_model(model, DEFAULT_MODEL);
```

**Already handled by `ProviderBuilder` but not consistently used.**

---

## RECOMMENDATIONS FOR CODE REVIEW

1. **Consolidate Constructor Pattern**: Mandate use of `impl_provider_constructors!` macro
2. **Trait-Based Converters**: Require all new providers to use `MessageConverter` trait
3. **Testing Strategy**: Shared trait implementations should have generic test suites
4. **Wrapper Pattern**: Review candidates for wrapping instead of reimplementing
5. **Documentation**: Add "Provider Implementation Checklist" for new providers
6. **Linting**: Add clippy rules to detect duplicated match statements

---

## RISKS & MITIGATION

| Risk | Likelihood | Mitigation |
|------|---|---|
| Breaking changes during refactor | Medium | Comprehensive test suite before refactoring |
| Performance regression | Low | Benchmark before/after on 10k requests |
| Increased complexity in traits | Medium | Keep traits simple, add impl-specific tests |
| Provider-specific quirks lost | Low | Document all provider-specific logic in comments |

---

## CONCLUSION

The provider code has excellent structure but significant duplication (22-26% of code). The good news is that:

1. Common infrastructure already exists in `common.rs` and `shared/mod.rs`
2. Duplication is in predictable patterns (not scattered randomly)
3. Refactoring can be done incrementally without breaking changes
4. Estimated 74-83% reduction in duplicated code is achievable

**Estimated Phase 3 effort:** 3-4 weeks for full implementation + 1 week testing = ~4-5 weeks total

**Expected outcome:** 
- Cleaner, more maintainable codebase
- Faster onboarding for new providers
- ~3,500-3,900 lines of code eliminated
- Better test coverage of common patterns
- Easier to add new providers (50% less boilerplate)

