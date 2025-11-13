# LLM Provider Analysis - Quick Reference

## Line Count Summary

```
Provider                     Lines    Streaming   Tools   Cache   Notes
─────────────────────────────────────────────────────────────────────────
openai.rs                    2,624    YES         YES     YES     Largest, Responses API
openrouter.rs                2,252    YES         YES     YES     Proxy pattern
gemini.rs                    1,253    YES         YES     YES     Google API
anthropic.rs                 1,127    NO          YES     YES     Minimax wrapper source
ollama.rs                      893    YES         YES     NO      Local inference
zai.rs                         735    NO          YES     NO      Z.AI proxy
deepseek.rs                    582    NO          YES     YES     Reasoning support
moonshot.rs                    521    NO          YES     YES     Heavy mode support
reasoning.rs                   420    N/A         N/A     N/A     Shared reasoning
minimax.rs                     411    NO          YES     YES     Wraps Anthropic
lmstudio.rs                    215    YES         YES     NO      OpenAI-compatible
xai.rs                         143    NO          YES     YES      Wraps OpenAI
shared/mod.rs                  496    YES         N/A     N/A     Stream processing
common.rs                      205    N/A         N/A     YES      Builder pattern
codex_prompt.rs                103    N/A         N/A     N/A     Dev prompt
mod.rs                          31    N/A         N/A     N/A     Module exports
─────────────────────────────────────────────────────────────────────────
TOTAL                        11,589
```

## Duplication Matrix

```
Pattern                    Affected Providers (count)
─────────────────────────────────────────────────────────
Constructor (new/with_model/from_config)     11/11 ✗
Message Serialization                         6/11 ✗
Request Format Conversion                     9/11 ✗
Tool Serialization                            7/11 ✗
Chat Request Parsing                          5/11 ✗
HTTP Error Handling                           9/11 ✗
Finish Reason Mapping                         9/11 ✗
Streaming Impl (SSE/deltas)                   4/11 ✗

Wrapper Pattern (good)                        2/11 ✓
Shared Reasoning                             11/11 ✓
Shared Provider Builder                       8/11 ✓
```

## Duplicated Code Blocks

### Duplicated 11 times (all providers):
```
pub fn new(api_key: String) -> Self { ... }
pub fn with_model(api_key: String, model: String) -> Self { ... }
pub fn from_config(...) -> Self { ... }
```
**Savings if extracted:** 550 lines (50+ lines × 11 providers)
**Solution:** Use `impl_provider_constructors!` macro (already exists!)

### Duplicated 9 times (temperature):
```
if let Some(temperature) = request.temperature {
    payload.insert("temperature".to_string(), 
        Value::Number(...));
}
```
**Savings if extracted:** ~100 lines
**Solution:** Create `RequestPayloadBuilder` trait

### Duplicated 9 times (error handling):
```
if !response.status().is_success() {
    // Check 401, 403 for authentication
    // Check 429 for rate limit
    // Check 400 for invalid request
}
```
**Savings if extracted:** 150-200 lines
**Solution:** Create `ErrorMapper` trait

### Duplicated 7 times (tool serialization):
```
fn serialize_tools(tools: &[ToolDefinition]) -> Option<Value> {
    // Convert to provider-specific format
}
```
**Savings if extracted:** 250-350 lines
**Solution:** Create `ToolSerializer` trait

### Duplicated 6 times (message serialization):
```
fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError> {
    for msg in &request.messages {
        match msg.role {
            MessageRole::System => { ... }
            MessageRole::User => { ... }
            MessageRole::Assistant => { ... }
            MessageRole::Tool => { ... }
        }
    }
}
```
**Savings if extracted:** 400-500 lines
**Solution:** Create `MessageConverter` trait

## Unused Infrastructure

### Macro NOT Being Used:
File: `src/llm/providers/shared/mod.rs` (lines 465-496)
```rust
#[macro_export]
macro_rules! impl_provider_constructors {
    (default_model: $default_model:expr, resolve_fn: $resolve_fn:path) => {
        // Generates new(), with_model(), from_config() automatically
    };
}
```
**Status:** Defined but NO providers use it
**Quick Win:** Apply to all 11 providers → 550 lines saved immediately

## Phase 3 Implementation Order

### PRIORITY 1 (Week 1, Low Risk, 550 lines saved):
1. Enable `impl_provider_constructors!` macro in all 11 providers
2. Verify no behavioral changes
3. Run full test suite

### PRIORITY 2 (Week 2, Low-Medium Risk, 150-200 lines):
1. Extract `ErrorMapper` trait
   - HTTP status → LLMError mapping
   - Finish reason string → FinishReason enum
2. Implement for each provider (small change)

### PRIORITY 3 (Week 3, Medium Risk, 400-500 lines):
1. Extract `MessageConverter` trait
   - serialize_messages() becomes trait method
2. Implement for 6 providers with message serialization
3. Add generic tests

### PRIORITY 4 (Week 4, Medium Risk, 250-350 lines):
1. Extract `ToolSerializer` trait
2. Implement for 7 providers
3. Add parameterized tests

### PRIORITY 5 (Week 5, Higher Risk, 300-400 lines):
1. Enhance `providers/shared/streaming.rs`
   - Add SSEStreamProcessor
   - Add EventHandler trait
2. Refactor 4 streaming providers

### PRIORITY 6 (Week 6, Medium Risk, 200-300 lines):
1. Apply wrapper pattern
   - Moonshot wraps OpenAI
   - LMStudio wraps OpenAI
   - Ollama wraps generic ChatCompletion

### PRIORITY 7 (Week 7, Validation):
1. Full test suite
2. Performance benchmarks
3. Code review

## File Changes Required

### New Files to Create:
- `src/llm/providers/message_converter.rs` (150-200 lines)
- `src/llm/providers/tool_serializer.rs` (100-150 lines)
- `src/llm/providers/error_mapper.rs` (150-200 lines)
- `src/llm/providers/request_builder.rs` (100-150 lines)
- Enhance: `src/llm/providers/shared/streaming.rs` (add 150-200 lines)

### Files to Modify:
- All 11 provider files (small changes to use traits)
- Tests in each provider (convert to trait tests)

## Immediate Quick Win

**Can be done in 1-2 hours:**
1. Uncomment `impl_provider_constructors!` usage in each provider
2. Test each provider
3. Save 550 lines of boilerplate

**Zero risk:** Macro already exists and works, just not being used.

## Estimated Effort & Savings

| Phase | Effort | Lines Saved | Risk Level |
|-------|--------|------------|-----------|
| Phase 3a (constructors + errors) | 1 week | 700 | LOW |
| Phase 3b (converters + serializers) | 2 weeks | 1,000 | MEDIUM |
| Phase 3c (wrappers + refactoring) | 1 week | 500 | MEDIUM |
| Phase 3d (testing + validation) | 1 week | 0 | LOW |
| **TOTAL** | **~4 weeks** | **~2,200** | **LOW-MEDIUM** |

## Providers by Size (where duplication matters most)

```
1. OpenAI (2,624 lines) - Consolidate streaming
2. OpenRouter (2,252 lines) - Consolidate streaming  
3. Gemini (1,253 lines) - Extract message converter
4. Anthropic (1,127 lines) - Message converter + tools
5. Ollama (893 lines) - Wrapper candidate
6. ZAI (735 lines) - Extract message/tool serializers
7. DeepSeek (582 lines) - Message serializer
8. Moonshot (521 lines) - Wrapper candidate
9. Minimax (411 lines) - Already wraps Anthropic ✓
10. LMStudio (215 lines) - Wrapper candidate
11. XAI (143 lines) - Already wraps OpenAI ✓
```

**Best ROI:** Focus on OpenAI, OpenRouter, Gemini, Anthropic (accounts for 7,256 / 11,589 = 63% of code)

## Test Coverage Status

```
Module                    Current Tests    Gap
────────────────────────────────────────────────
shared/mod.rs                 6           GOOD
reasoning.rs                  6           GOOD
anthropic.rs                 15           MEDIUM
openai.rs                    12           MEDIUM
gemini.rs                     8           MEDIUM
other providers            ~100           POOR

Gaps to fill:
- Unified message serialization tests (needed)
- Unified tool serialization tests (needed)
- Unified error mapping tests (needed)
- Streaming delta tests (minimal)
```

