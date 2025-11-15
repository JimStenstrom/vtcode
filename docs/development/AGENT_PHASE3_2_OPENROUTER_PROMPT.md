# Agent 2: Fix vtcode-llm-openrouter Integration - Task Brief

**Your Role:** Fix vtcode-llm-openrouter provider integration with vtcode-core
**Estimated Time:** 1-1.5 hours
**Branch Name:** `phase3-fix-openrouter`
**Working in Parallel With:** Agent 1 (Gemini), Agent 3 (Microsoft)

---

## Your Mission

You are **Agent 2** in a 4-agent parallel execution to fix Phase 3 compilation errors. Your job is to fix the **vtcode-llm-openrouter** provider crate so it properly integrates with vtcode-core's new architecture.

**Current Status:**
- ❌ vtcode-core cannot find OpenRouterProvider constructors (`with_model`, `from_config`)
- ❌ OpenRouterProvider doesn't properly implement LLMProvider trait
- ❌ Build fails with 3 errors related to OpenRouter

**Your Goal:**
- ✅ Add missing constructor methods to OpenRouterProvider
- ✅ Ensure LLMProvider trait is properly implemented
- ✅ All OpenRouter-related compilation errors resolved
- ✅ Tests pass

---

## Context: Phase 3 Architecture

VTCode completed Phase 3 refactoring to break circular dependencies by extracting LLM providers into separate crates. The openrouter provider was extracted to `vtcode-llm-openrouter/` but is missing some integration points.

**What Changed:**
- Before: OpenRouter code lived in `vtcode-core/src/llm/providers/openrouter.rs`
- After: OpenRouter code moved to `vtcode-llm-openrouter/` crate
- Problem: Constructor methods and trait implementations incomplete

---

## Required Reading

**IMPORTANT:** Quickly review these for context:

1. **vtcode-llm-types/src/request.rs** - Common types (LLMRequest, LLMResponse, etc.)
2. **vtcode-llm-openai/src/provider.rs** - Reference implementation (OpenAI provider shows the pattern)
3. **vtcode-core/src/llm/factory.rs** - How providers are instantiated

---

## Your Tasks (1-1.5 hours)

### ✅ Pre-Flight Checklist (5 minutes)

```bash
# Clone repo (if not already)
cd /path/to/vtcode

# Checkout base branch
git checkout claude/main
git pull

# Create your branch
git checkout -b phase3-fix-openrouter

# Verify environment
direnv allow .
cargo build 2>&1 | grep -i openrouter

# You should see errors like:
# error[E0599]: no function or associated item named `with_model` found for struct `OpenRouterProvider`
# error[E0599]: no function or associated item named `from_config` found for struct `OpenRouterProvider`
# error[E0277]: the trait bound `OpenRouterProvider: llm::provider::LLMProvider` is not satisfied
```

---

### Task 2.1: Add `with_model` Constructor (20 minutes)

**Location:** `vtcode-llm-openrouter/src/provider.rs`

**Current Error:**
```
error[E0599]: no function or associated item named `with_model` found for struct `OpenRouterProvider`
  --> vtcode-core/src/llm/client.rs:43:62
   |
43 |         Provider::OpenRouter => Box::new(OpenRouterProvider::with_model(
   |                                                              ^^^^^^^^^^ function or associated item not found in `OpenRouterProvider`
```

**What vtcode-core Expects:**
```rust
// vtcode-core/src/llm/client.rs:43
OpenRouterProvider::with_model(
    api_key.to_string(),
    model.to_string(),
    config.agent.prompt_cache.clone(),
    config.agent.openrouter_base_url.clone(),
)
```

**Your Fix:**

Open `vtcode-llm-openrouter/src/provider.rs` and add this method:

```rust
impl OpenRouterProvider {
    /// Create a new OpenRouter provider with a specific model
    pub fn with_model(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        // Determine base URL
        let base_url = base_url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OPENROUTER_BASE_URL").ok())
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());

        // Extract prompt cache settings if available
        // OpenRouter doesn't have native prompt caching, but we keep the config for compatibility
        let prompt_cache_enabled = prompt_cache
            .as_ref()
            .map(|cfg| cfg.enabled)
            .unwrap_or(false);

        Self {
            api_key,
            http_client: HttpClient::new(),
            base_url,
            model,
            prompt_cache_enabled,
        }
    }

    // ... existing code ...
}
```

**Imports You May Need:**
```rust
use vtcode_config::PromptCachingConfig;
```

**Note:** If `OpenRouterProvider` struct doesn't have a `prompt_cache_enabled` field, you can:
1. Add it: `prompt_cache_enabled: bool,`
2. Or remove it from the constructor (check the struct definition first)

**Test:**
```bash
cargo build -p vtcode-llm-openrouter
```

**Checkpoint:** Should compile without the "with_model not found" error.

---

### Task 2.2: Add `from_config` Constructor (20 minutes)

**Current Error:**
```
error[E0599]: no function or associated item named `from_config` found for struct `OpenRouterProvider`
   --> vtcode-core/src/llm/factory.rs:322:38
    |
322 |         Box::new(OpenRouterProvider::from_config(
    |                                      ^^^^^^^^^^^ function or associated item not found in `OpenRouterProvider`
```

**What vtcode-core Expects:**
```rust
// vtcode-core/src/llm/factory.rs:322
OpenRouterProvider::from_config(
    model.to_string(),
    config,
)
```

**Your Fix:**

Add this method to `OpenRouterProvider` impl block:

```rust
impl OpenRouterProvider {
    /// Create a new OpenRouter provider from config
    pub fn from_config(model: String, config: &vtcode_config::Config) -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow::anyhow!(
                "OPENROUTER_API_KEY environment variable not set"
            ))?;

        Ok(Self::with_model(
            api_key,
            model,
            config.agent.prompt_cache.clone(),
            config.agent.openrouter_base_url.clone(),
        ))
    }

    // ... existing code ...
}
```

**Imports You May Need:**
```rust
use anyhow;
```

**Test:**
```bash
cargo build -p vtcode-llm-openrouter
```

**Checkpoint:** Should compile without the "from_config not found" error.

---

### Task 2.3: Verify LLMProvider Trait Implementation (15 minutes)

**Current Error:**
```
error[E0277]: the trait bound `OpenRouterProvider: llm::provider::LLMProvider` is not satisfied
   --> vtcode-core/src/llm/factory.rs:313:26
    |
313 | impl BuiltinProvider for OpenRouterProvider {
    |                          ^^^^^^^^^^^^^^^^^^ the trait `llm::provider::LLMProvider` is not implemented for `OpenRouterProvider`
```

**What This Means:**
The `OpenRouterProvider` struct should already have an `impl` block that implements the `LLMProvider` trait from `vtcode-llm-types`. This error suggests the trait implementation uses wrong types.

**Your Investigation:**

1. Search for the LLMProvider implementation:
```bash
cd vtcode-llm-openrouter
grep -n "impl LLMProvider" src/provider.rs
```

2. Check that the trait uses types from `vtcode_llm_types`:
```rust
// Should look like this:
use vtcode_llm_types::{LLMRequest, LLMResponse, LLMProvider, LLMStream};

#[async_trait::async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        // ... implementation ...
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream> {
        // ... implementation ...
    }
}
```

3. **Common Issues to Fix:**
   - If using `crate::llm::provider::LLMProvider`, change to `vtcode_llm_types::LLMProvider`
   - If using local types, change to `vtcode_llm_types::*`
   - Ensure `LLMRequest` and `LLMResponse` are from `vtcode_llm_types`

**Test:**
```bash
cargo build -p vtcode-llm-openrouter
cargo build -p vtcode-core
```

**Checkpoint:** Should compile without trait bound errors.

---

### Task 2.4: Fix ReasoningEffortLevel Issues (15 minutes)

**Potential Issue:**
The OpenRouter provider may have reasoning effort handling code that references removed functions or uses mismatched types.

**Your Investigation:**

Search for reasoning effort code:
```bash
cd vtcode-llm-openrouter
grep -n "reasoning_effort\|ReasoningEffortLevel" src/provider.rs
```

**Common Problems & Fixes:**

**Problem 1:** Type mismatch between `vtcode_config::ReasoningEffortLevel` and `vtcode_llm_types::ReasoningEffortLevel`

```rust
// If you see code that converts between them:
let reasoning_effort = request.reasoning_effort.map(|e| match e {
    vtcode_llm_types::ReasoningEffortLevel::Low => vtcode_config::ReasoningEffortLevel::Low,
    vtcode_llm_types::ReasoningEffortLevel::Medium => vtcode_config::ReasoningEffortLevel::Medium,
    vtcode_llm_types::ReasoningEffortLevel::High => vtcode_config::ReasoningEffortLevel::High,
});

// This is correct - keep it!
```

**Problem 2:** Reference to removed `reasoning_parameters_for` function

```rust
// If you see (WRONG):
let params = reasoning_parameters_for(model, effort);

// Replace with (CORRECT):
if let Some(effort) = request.reasoning_effort {
    if self.supports_reasoning_effort(model) {
        provider_request["reasoning"] = json!({ "effort": effort.as_str() });
    }
}
```

**Test:**
```bash
cargo build -p vtcode-llm-openrouter
```

**Checkpoint:** No reasoning-related errors.

---

### Task 2.5: Run Tests (10 minutes)

**Run the provider's tests:**
```bash
cd vtcode-llm-openrouter
cargo test
```

**Run integration with vtcode-core:**
```bash
cd ..
cargo build --all
```

**Expected Result:**
- ✅ All vtcode-llm-openrouter tests pass
- ✅ vtcode-core compiles (may still have errors from other providers - that's OK!)
- ✅ No OpenRouter-related errors

---

### Task 2.6: Commit Your Work (5 minutes)

```bash
git add vtcode-llm-openrouter/
git commit -m "fix(openrouter): Add missing constructors for vtcode-core integration

Add with_model and from_config constructors to OpenRouterProvider to support
vtcode-core's provider factory pattern.

Changes:
- Added with_model(api_key, model, prompt_cache, base_url) constructor
- Added from_config(model, config) constructor
- Verified LLMProvider trait implementation uses vtcode_llm_types
- Fixed reasoning effort handling

Fixes:
- E0599: with_model not found
- E0599: from_config not found
- E0277: LLMProvider trait bound

Testing:
- vtcode-llm-openrouter builds cleanly
- vtcode-llm-openrouter tests pass
- No OpenRouter-related errors in vtcode-core

Part of Phase 3 parallel compilation fixes (Agent 2/4)"

git push -u origin phase3-fix-openrouter
```

---

## Coordination Points

### With Agent 1 (Gemini)

**No coordination needed** - you're working on different crates.

**Optional:** Share status updates
```
Agent 2 Update (T+45min):
✅ Added with_model constructor
✅ Added from_config constructor
✅ Verified trait implementation
⏳ Running tests
ETA: 15 minutes
```

### With Agent 4 (Core Imports)

**Agent 4 depends on your work** - they need to know your constructors work before they update imports.

**At completion:** Post in coordination channel:
```
Agent 2 COMPLETE:
✅ OpenRouterProvider::with_model() ✅
✅ OpenRouterProvider::from_config() ✅
✅ All tests pass ✅
✅ Branch pushed: phase3-fix-openrouter

@Agent4 - You can start using the openrouter provider now!
```

---

## Success Criteria

Your task is complete when:

- [ ] ✅ `with_model` constructor added to OpenRouterProvider
- [ ] ✅ `from_config` constructor added to OpenRouterProvider
- [ ] ✅ LLMProvider trait properly implemented (uses vtcode_llm_types)
- [ ] ✅ Reasoning effort handling fixed (if applicable)
- [ ] ✅ `cargo build -p vtcode-llm-openrouter` succeeds
- [ ] ✅ `cargo test -p vtcode-llm-openrouter` passes
- [ ] ✅ No OpenRouter errors when building vtcode-core
- [ ] ✅ Changes committed to `phase3-fix-openrouter` branch
- [ ] ✅ Branch pushed to GitHub

---

## Troubleshooting

### Problem: Can't find vtcode_config types

**Solution:**
Check `vtcode-llm-openrouter/Cargo.toml` dependencies:
```toml
[dependencies]
vtcode-config = { path = "../vtcode-config" }
vtcode-llm-types = { path = "../vtcode-llm-types" }
```

### Problem: Trait implementation still fails

**Solution:**
Ensure ALL imports use `vtcode_llm_types`:
```rust
use vtcode_llm_types::{
    LLMProvider, LLMRequest, LLMResponse, LLMStream,
    Message, ContentPart, ToolCall, Usage
};
```

NOT:
```rust
use crate::types::*;  // ❌ Wrong
use vtcode_core::llm::*;  // ❌ Wrong
```

### Problem: Tests fail

**Solution:**
Run with more verbose output:
```bash
cargo test -- --nocapture
```

Check the specific test failure and fix it.

### Problem: vtcode-core still has import errors

**Solution:**
That's Agent 4's job! Just ensure:
```bash
cargo build -p vtcode-llm-openrouter  # Should pass ✅
```

Your crate works, Agent 4 will fix the imports.

---

## Reference: Expected File Structure

After your changes, `vtcode-llm-openrouter/src/provider.rs` should have:

```rust
use vtcode_llm_types::{LLMProvider, LLMRequest, LLMResponse, LLMStream, /* ... */};
use vtcode_config::{PromptCachingConfig, Config, /* ... */};

pub struct OpenRouterProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_enabled: bool,  // or removed if not used
}

impl OpenRouterProvider {
    /// Create with specific model
    pub fn with_model(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        // ... implementation from Task 2.1 ...
    }

    /// Create from config
    pub fn from_config(model: String, config: &Config) -> anyhow::Result<Self> {
        // ... implementation from Task 2.2 ...
    }

    // ... existing helper methods ...
}

#[async_trait::async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        // ... existing implementation ...
    }

    async fn stream(&self, request: LLMRequest) -> Result<LLMStream> {
        // ... existing implementation ...
    }
}
```

---

## Communication

**GitHub Issue:** [Link to coordination issue]

**Status Update Template:**
```
Agent 2 (OpenRouter) - T+[minutes]:
✅ Completed: [task]
⏳ In progress: [task]
❌ Blocked: [issue]
ETA: [minutes]
```

---

## Timeline

| Time | Activity | Checkpoint |
|------|----------|------------|
| 0-5 min | Setup, branch creation | Branch ready |
| 5-25 min | Add with_model | Constructor works |
| 25-45 min | Add from_config | Both constructors work |
| 45-60 min | Verify trait impl | Builds cleanly |
| 60-75 min | Fix reasoning handling | No reasoning errors |
| 75-85 min | Run tests | Tests pass |
| 85-90 min | Commit and push | Complete ✅ |

**Total: 1-1.5 hours**

---

**Ready to fix the OpenRouter provider? Let's go! 🚀**
