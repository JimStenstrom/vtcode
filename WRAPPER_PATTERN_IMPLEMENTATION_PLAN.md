# Implementation Plan for Non-Wrapper Providers

**Date**: 2025-11-13
**Status**: PROPOSED
**Related**: WRAPPER_PATTERN_ANALYSIS.md, PHASE_3_CRITICAL_REVIEW.md
**Branch**: `claude/fix-wrapper-pattern-limitations-011CV6BPgQ43pAwXavyDbbaC`

---

## Purpose

This document provides actionable recommendations for handling providers that cannot use the wrapper pattern (Moonshot, Ollama, and others) during Phase 3 refactoring.

---

## Problem Statement

The Phase 3 plan assumed all "OpenAI-compatible" providers could be wrapped, but analysis revealed:

**Reality**:
- **Moonshot**: 524 lines with Heavy Mode, custom reasoning, custom cache fields
- **Ollama**: 894 lines with local/cloud detection, custom streaming, unique request format

**Impact**:
- Cannot use simple wrapper pattern
- Still have code duplication with other providers
- Need alternative refactoring approach

---

## Strategy: Trait-Based Composition (Not Inheritance)

Instead of forcing wrappers, extract **common behaviors into traits** that providers implement with their own logic.

### Philosophy

```
❌ WRONG: Force all providers into wrapper pattern
✅ RIGHT: Extract common patterns, let providers implement with variations
```

---

## Proposed Trait Architecture

### 1. Message Serialization Trait

**File**: `vtcode-core/src/llm/providers/shared/message_converter.rs`

```rust
/// Converts LLMRequest messages to provider-specific format
pub trait MessageConverter {
    /// Serialize messages to provider's expected format
    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError>;

    /// Validate messages for provider constraints
    fn validate_messages(&self, messages: &[Message]) -> Result<(), String> {
        // Default implementation for common validations
        for message in messages {
            message.validate_for_provider(self.provider_key())?;
        }
        Ok(())
    }

    /// Provider identifier for validation
    fn provider_key(&self) -> &str;
}

/// Helper struct for OpenAI-style message conversion
pub struct OpenAIStyleConverter;

impl MessageConverter for OpenAIStyleConverter {
    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError> {
        let mut messages = Vec::with_capacity(request.messages.len());
        for message in &request.messages {
            let mut map = Map::new();
            map.insert("role".to_string(), Value::String(message.role.as_generic_str().to_string()));
            map.insert("content".to_string(), Value::String(message.content.as_text()));

            if let Some(tool_calls) = &message.tool_calls {
                map.insert("tool_calls".to_string(), serialize_tool_calls_openai(tool_calls)?);
            }

            messages.push(Value::Object(map));
        }
        Ok(messages)
    }

    fn provider_key(&self) -> &str {
        "openai"
    }
}
```

**How Moonshot Uses It**:
```rust
impl MessageConverter for MoonshotProvider {
    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError> {
        // Start with OpenAI-style base
        let mut messages = OpenAIStyleConverter.serialize_messages(request)?;

        // Add Moonshot-specific modifications if needed
        // (Currently Moonshot is already OpenAI-compatible here)

        Ok(messages)
    }

    fn provider_key(&self) -> &str {
        "moonshot"
    }
}
```

**How Ollama Uses It**:
```rust
impl MessageConverter for OllamaProvider {
    fn serialize_messages(&self, request: &LLMRequest) -> Result<Vec<Value>, LLMError> {
        let mut messages = Vec::new();
        let mut tool_names: HashMap<String, String> = HashMap::new();

        // System prompt handling
        if let Some(system) = &request.system_prompt {
            messages.push(/* ... Ollama-specific format ... */);
        }

        // Custom tool name tracking for Ollama
        for message in &request.messages {
            // ... Ollama-specific logic with tool_name field ...
        }

        Ok(messages)
    }

    fn provider_key(&self) -> &str {
        "ollama"
    }
}
```

**Code Savings**: ~150-200 lines (shared validation + helper functions)

---

### 2. Tool Serialization Trait

**File**: `vtcode-core/src/llm/providers/shared/tool_serializer.rs`

```rust
/// Converts tool definitions and calls to provider format
pub trait ToolSerializer {
    /// Serialize tool definitions for provider
    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Result<Vec<Value>, LLMError> {
        // Default OpenAI format
        Ok(tools.iter().map(|tool| json!(tool)).collect())
    }

    /// Serialize tool choice parameter
    fn serialize_tool_choice(&self, choice: &ToolChoice) -> Value {
        choice.to_provider_format(self.provider_key())
    }

    /// Convert provider's tool call response to common format
    fn parse_tool_calls(&self, response: &Value) -> Result<Option<Vec<ToolCall>>, LLMError>;

    fn provider_key(&self) -> &str;
}

/// Helper for OpenAI-style tool calls
pub fn parse_openai_tool_calls(tool_calls_value: &Value) -> Result<Option<Vec<ToolCall>>, LLMError> {
    let Some(calls_array) = tool_calls_value.as_array() else {
        return Ok(None);
    };

    let mut tool_calls = Vec::new();
    for call in calls_array {
        let id = call.get("id").and_then(|v| v.as_str()).ok_or(/* ... */)?;
        let function = call.get("function").and_then(|v| v.as_object()).ok_or(/* ... */)?;
        let name = function.get("name").and_then(|v| v.as_str()).ok_or(/* ... */)?;
        let arguments = function.get("arguments").and_then(|v| v.as_str()).ok_or(/* ... */)?;

        tool_calls.push(ToolCall::function(id.to_string(), name.to_string(), arguments.to_string()));
    }

    Ok(Some(tool_calls))
}
```

**How Ollama Uses It**:
```rust
impl ToolSerializer for OllamaProvider {
    fn parse_tool_calls(&self, response: &Value) -> Result<Option<Vec<ToolCall>>, LLMError> {
        // Custom parsing with index-based IDs and argument handling
        let Some(tool_calls) = response.as_array() else {
            return Ok(None);
        };

        let mut converted = Vec::new();
        for (index, call) in tool_calls.iter().enumerate() {
            let function = call.get("function").ok_or(/* ... */)?;
            let name = function.get("name").ok_or(/* ... */)?;
            let arguments_value = function.get("arguments").unwrap_or(&Value::Object(Map::new()));

            // Ollama-specific: arguments can be Value or String
            let arguments = match arguments_value {
                Value::String(raw) => raw.clone(),
                other => serde_json::to_string(other)?,
            };

            // Ollama-specific: use index for ID
            let id = function.get("index")
                .map(|v| format!("tool_call_{}", v))
                .unwrap_or_else(|| format!("tool_call_{}", index));

            converted.push(ToolCall::function(id, name.to_string(), arguments));
        }

        Ok(Some(converted))
    }

    fn provider_key(&self) -> &str {
        "ollama"
    }
}
```

**Code Savings**: ~250-350 lines (shared tool call parsing logic)

---

### 3. Error Mapping Trait

**File**: `vtcode-core/src/llm/providers/shared/error_mapper.rs`

```rust
/// Maps provider-specific errors to common LLMError
pub trait ErrorMapper {
    /// Map HTTP status and body to LLMError
    fn map_error(&self, status: StatusCode, body: &str) -> LLMError {
        match status.as_u16() {
            401 => {
                let formatted = error_display::format_llm_error(
                    self.provider_name(),
                    "Authentication failed (check API key)",
                );
                LLMError::Authentication(formatted)
            }
            429 => LLMError::RateLimit,
            _ => {
                // Try to extract provider-specific error message
                let error_msg = self.extract_error_message(body)
                    .unwrap_or_else(|| format!("HTTP {}: {}", status, body));

                let formatted = error_display::format_llm_error(self.provider_name(), &error_msg);
                LLMError::Provider(formatted)
            }
        }
    }

    /// Extract error message from response body
    fn extract_error_message(&self, body: &str) -> Option<String> {
        // Default: try to parse as JSON with "error" field
        serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| v.get("error"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
    }

    /// Map reqwest error
    fn map_reqwest_error(&self, err: reqwest::Error) -> LLMError {
        if err.is_timeout() || err.is_connect() {
            LLMError::Network(err.to_string())
        } else {
            LLMError::Provider(err.to_string())
        }
    }

    fn provider_name(&self) -> &str;
}
```

**How Moonshot Uses It**:
```rust
impl ErrorMapper for MoonshotProvider {
    fn map_error(&self, status: StatusCode, body: &str) -> LLMError {
        // Check for quota errors (common pattern)
        if status.as_u16() == 429 || body.contains("quota") {
            return LLMError::RateLimit;
        }

        // Use default mapping for other errors
        ErrorMapper::map_error(self, status, body)
    }

    fn provider_name(&self) -> &str {
        "Moonshot"
    }
}
```

**Code Savings**: ~150-200 lines (shared error mapping logic)

---

### 4. Request Payload Builder Trait

**File**: `vtcode-core/src/llm/providers/shared/payload_builder.rs`

```rust
/// Builds request payloads for providers
pub trait PayloadBuilder {
    /// Build payload for generate/stream request
    fn build_payload(&self, request: &LLMRequest, stream: bool) -> Result<Value, LLMError>;

    /// Add basic parameters (model, temperature, max_tokens)
    fn add_basic_params(&self, payload: &mut Map<String, Value>, request: &LLMRequest) {
        payload.insert("model".to_string(), Value::String(request.model.clone()));

        if let Some(max_tokens) = request.max_tokens {
            payload.insert("max_tokens".to_string(), json!(max_tokens));
        }

        if let Some(temperature) = request.temperature {
            payload.insert("temperature".to_string(), json!(temperature));
        }
    }

    /// Add tools if present
    fn add_tools(&self, payload: &mut Map<String, Value>, request: &LLMRequest) -> Result<(), LLMError>;

    /// Add provider-specific parameters
    fn add_provider_params(&self, payload: &mut Map<String, Value>, request: &LLMRequest) -> Result<(), LLMError> {
        // Default: no additional params
        Ok(())
    }
}
```

**How Moonshot Uses It**:
```rust
impl PayloadBuilder for MoonshotProvider {
    fn build_payload(&self, request: &LLMRequest, stream: bool) -> Result<Value, LLMError> {
        let mut payload = Map::new();

        // Use shared basic params
        self.add_basic_params(&mut payload, request);

        // Add messages
        payload.insert("messages".to_string(), Value::Array(self.serialize_messages(request)?));

        // Add stream flag
        payload.insert("stream".to_string(), Value::Bool(stream));

        // Add tools
        self.add_tools(&mut payload, request)?;

        // Add Moonshot-specific params (Heavy Mode, reasoning)
        self.add_provider_params(&mut payload, request)?;

        Ok(Value::Object(payload))
    }

    fn add_tools(&self, payload: &mut Map<String, Value>, request: &LLMRequest) -> Result<(), LLMError> {
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                payload.insert("tools".to_string(), json!(tools));

                if let Some(choice) = &request.tool_choice {
                    payload.insert("tool_choice".to_string(), choice.to_provider_format("moonshot"));
                }
            }
        }
        Ok(())
    }

    fn add_provider_params(&self, payload: &mut Map<String, Value>, request: &LLMRequest) -> Result<(), LLMError> {
        // Handle reasoning effort
        if let Some(effort) = request.reasoning_effort {
            if self.supports_reasoning_effort(&request.model) {
                if let Some(reasoning_payload) = reasoning_parameters_for(ModelProvider::Moonshot, effort) {
                    if let Some(obj) = reasoning_payload.as_object() {
                        for (key, value) in obj {
                            payload.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        // Apply Heavy Mode for turbo model
        if request.model == models::moonshot::KIMI_K2_THINKING_TURBO {
            payload.insert("heavy_thinking".to_string(), Value::Bool(true));
            payload.insert("parallel_trajectories".to_string(), json!(8));
            payload.insert("trajectory_aggregation".to_string(), json!("reflective"));
        }

        Ok(())
    }
}
```

**Code Savings**: ~100-150 lines (shared payload building logic)

---

### 5. Response Parser Trait

**File**: `vtcode-core/src/llm/providers/shared/response_parser.rs`

```rust
/// Parses provider responses to common LLMResponse format
pub trait ResponseParser {
    /// Parse response JSON to LLMResponse
    fn parse_response(&self, response_json: Value) -> Result<LLMResponse, LLMError>;

    /// Extract usage information
    fn extract_usage(&self, response: &Value) -> Option<Usage> {
        let usage_value = response.get("usage")?;

        Some(Usage {
            prompt_tokens: usage_value.get("prompt_tokens")?.as_u64()? as u32,
            completion_tokens: usage_value.get("completion_tokens")?.as_u64()? as u32,
            total_tokens: usage_value.get("total_tokens")?.as_u64()? as u32,
            cached_prompt_tokens: self.extract_cached_tokens(usage_value),
            cache_creation_tokens: self.extract_cache_creation_tokens(usage_value),
            cache_read_tokens: self.extract_cache_read_tokens(usage_value),
        })
    }

    /// Extract cached tokens (provider-specific field names)
    fn extract_cached_tokens(&self, usage: &Value) -> Option<u32> {
        // Default OpenAI naming
        usage.get("cached_tokens").and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn extract_cache_creation_tokens(&self, usage: &Value) -> Option<u32> {
        usage.get("cache_creation_tokens").and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn extract_cache_read_tokens(&self, usage: &Value) -> Option<u32> {
        usage.get("cache_read_tokens").and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    /// Parse finish reason
    fn parse_finish_reason(&self, reason: Option<&str>) -> FinishReason {
        match reason {
            Some("stop") | None => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            Some(other) => FinishReason::Error(other.to_string()),
        }
    }
}
```

**How Moonshot Uses It**:
```rust
impl ResponseParser for MoonshotProvider {
    fn parse_response(&self, response_json: Value) -> Result<LLMResponse, LLMError> {
        let choices = response_json.get("choices").and_then(|v| v.as_array()).ok_or(/* ... */)?;
        let choice = &choices[0];
        let message = choice.get("message").ok_or(/* ... */)?;

        let content = message.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());

        let tool_calls = self.parse_tool_calls(message)?;

        // Moonshot-specific: reasoning_content field
        let reasoning = message.get("reasoning_content").and_then(|v| v.as_str()).map(|s| s.to_string());

        let finish_reason = self.parse_finish_reason(
            choice.get("finish_reason").and_then(|v| v.as_str())
        );

        let usage = self.extract_usage(&response_json);

        Ok(LLMResponse {
            content,
            tool_calls,
            usage,
            finish_reason,
            reasoning,
            reasoning_details: None,
        })
    }

    // Override to use Moonshot-specific cache field names
    fn extract_cached_tokens(&self, usage: &Value) -> Option<u32> {
        if self.prompt_cache_enabled {
            usage.get("prompt_cache_hit_tokens").and_then(|v| v.as_u64()).map(|v| v as u32)
        } else {
            None
        }
    }

    fn extract_cache_creation_tokens(&self, usage: &Value) -> Option<u32> {
        if self.prompt_cache_enabled {
            usage.get("prompt_cache_miss_tokens").and_then(|v| v.as_u64()).map(|v| v as u32)
        } else {
            None
        }
    }
}
```

**Code Savings**: ~200-250 lines (shared response parsing logic)

---

## Implementation Phasing

### Phase 1: Trait Definitions (Week 1)
- [ ] Create trait files in `vtcode-core/src/llm/providers/shared/`
- [ ] Implement default/helper functions
- [ ] Add comprehensive documentation

**Estimated Effort**: 8-12 hours

### Phase 2: OpenAI Provider Family (Week 2)
- [ ] Refactor OpenAI to use traits
- [ ] Refactor XAI (verify still works as wrapper)
- [ ] Refactor LMStudio (verify still works as wrapper)
- [ ] Add trait tests

**Estimated Effort**: 12-16 hours

### Phase 3: Moonshot Provider (Week 3)
- [ ] Implement traits for Moonshot
- [ ] Remove duplicate code
- [ ] Verify Heavy Mode still works
- [ ] Add integration tests

**Estimated Effort**: 10-14 hours

### Phase 4: Ollama Provider (Week 4)
- [ ] Implement traits for Ollama
- [ ] Keep custom logic where needed
- [ ] Verify local/cloud handling
- [ ] Add integration tests

**Estimated Effort**: 12-16 hours

### Phase 5: Remaining Providers (Week 5-6)
- [ ] Anthropic, Gemini, DeepSeek, etc.
- [ ] Verify all providers still work
- [ ] Run full test suite
- [ ] Performance benchmarking

**Estimated Effort**: 20-30 hours

### Phase 6: Documentation & Cleanup (Week 7)
- [ ] Update provider implementation guide
- [ ] Document trait usage patterns
- [ ] Remove deprecated code
- [ ] Final PR review

**Estimated Effort**: 8-12 hours

**Total Estimated Effort**: 70-100 hours (7-10 weeks)

---

## Expected Code Reduction

| Area | Current Lines | After Traits | Savings |
|------|---------------|--------------|---------|
| Message serialization | ~800 | ~400 | ~400 |
| Tool serialization | ~600 | ~300 | ~300 |
| Error handling | ~400 | ~250 | ~150 |
| Response parsing | ~900 | ~550 | ~350 |
| Payload building | ~700 | ~500 | ~200 |
| **Total** | **~3,400** | **~2,000** | **~1,400** |

**Net Reduction**: ~40% of duplicated code

---

## Testing Strategy

### Unit Tests for Traits

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_message_converter() {
        let converter = OpenAIStyleConverter;
        let request = LLMRequest {
            messages: vec![Message::user("Hello".to_string())],
            // ...
        };

        let serialized = converter.serialize_messages(&request).unwrap();
        assert_eq!(serialized.len(), 1);
        assert_eq!(serialized[0]["role"], "user");
        assert_eq!(serialized[0]["content"], "Hello");
    }

    #[test]
    fn test_error_mapper_auth() {
        struct TestProvider;
        impl ErrorMapper for TestProvider {
            fn provider_name(&self) -> &str { "Test" }
        }

        let provider = TestProvider;
        let error = provider.map_error(StatusCode::UNAUTHORIZED, "Invalid API key");
        assert!(matches!(error, LLMError::Authentication(_)));
    }
}
```

### Integration Tests for Providers

```rust
#[tokio::test]
async fn test_moonshot_heavy_mode() {
    let provider = MoonshotProvider::with_model(
        test_api_key(),
        models::moonshot::KIMI_K2_THINKING_TURBO.to_string(),
    );

    let request = LLMRequest {
        model: models::moonshot::KIMI_K2_THINKING_TURBO.to_string(),
        messages: vec![Message::user("Test heavy mode".to_string())],
        // ...
    };

    let payload = provider.build_payload(&request, false).unwrap();

    // Verify Heavy Mode params are present
    assert_eq!(payload["heavy_thinking"], true);
    assert_eq!(payload["parallel_trajectories"], 8);
    assert_eq!(payload["trajectory_aggregation"], "reflective");
}
```

---

## Rollback Strategy

If trait refactoring causes issues:

1. **Revert provider changes** - Keep trait definitions but revert provider implementations
2. **Fix trait design** - Adjust trait signatures based on issues found
3. **Re-implement incrementally** - Do one provider at a time
4. **Feature flag** - Add `#[cfg(feature = "provider-traits")]` to allow opt-in

---

## Success Metrics

### Code Quality
- [ ] 40% reduction in duplicated code
- [ ] All providers pass existing tests
- [ ] No performance regression (within 5%)

### Maintainability
- [ ] New provider can be added in <4 hours (vs current ~8 hours)
- [ ] Trait documentation is clear and complete
- [ ] Code review feedback is positive

### Stability
- [ ] Zero breaking changes to public API
- [ ] All 11 providers still functional
- [ ] All provider-specific features working

---

## Conclusion

**Instead of forcing wrapper pattern**, use **trait-based composition** to:
1. Extract common behaviors
2. Allow provider customization
3. Reduce code duplication
4. Maintain provider-specific features

This approach:
- ✅ Works for ALL providers (wrappers AND standalone)
- ✅ Reduces code by ~40% (1,400 lines)
- ✅ Maintains provider flexibility
- ✅ Doesn't force inappropriate patterns

**Next Steps**:
1. Review and approve this plan
2. Create Phase 1 trait definitions
3. Start with OpenAI provider family
4. Iterate based on lessons learned

---

## References

- WRAPPER_PATTERN_ANALYSIS.md
- PHASE_3_CRITICAL_REVIEW.md
- vtcode-core/src/llm/providers/moonshot.rs
- vtcode-core/src/llm/providers/ollama.rs
