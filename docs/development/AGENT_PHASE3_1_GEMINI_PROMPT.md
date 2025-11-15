# Agent 1: Fix vtcode-llm-gemini Integration - Task Brief

**Your Role:** Fix vtcode-llm-gemini provider integration with vtcode-core
**Estimated Time:** 1-1.5 hours
**Branch Name:** `phase3-fix-gemini`
**Working in Parallel With:** Agent 2 (OpenRouter), Agent 3 (Microsoft)

---

## Your Mission

You are **Agent 1** in a 4-agent parallel execution to fix Phase 3 compilation errors. Your job is to fix the **vtcode-llm-gemini** provider crate so it properly integrates with vtcode-core's new architecture.

**Current Status:**
- ❌ vtcode-core cannot find GeminiProvider constructors (`with_model`, `from_config`)
- ❌ GeminiProvider doesn't properly implement LLMProvider trait
- ❌ Build fails with 3 errors related to Gemini

**Your Goal:**
- ✅ Add missing constructor methods to GeminiProvider
- ✅ Ensure LLMProvider trait is properly implemented
- ✅ All Gemini-related compilation errors resolved
- ✅ Tests pass

---

## Context: Phase 3 Architecture

VTCode completed Phase 3 refactoring to break circular dependencies by extracting LLM providers into separate crates. The gemini provider was extracted to `vtcode-llm-gemini/` but is missing some integration points.

**What Changed:**
- Before: Gemini code lived in `vtcode-core/src/llm/providers/gemini.rs`
- After: Gemini code moved to `vtcode-llm-gemini/` crate
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
git checkout -b phase3-fix-gemini

# Verify environment
direnv allow .
cargo build 2>&1 | grep -i gemini

# You should see errors like:
# error[E0599]: no function or associated item named `with_model` found for struct `GeminiProvider`
# error[E0599]: no function or associated item named `from_config` found for struct `GeminiProvider`
# error[E0277]: the trait bound `GeminiProvider: llm::provider::LLMProvider` is not satisfied
```

---

### Task 1.1: Add `with_model` Constructor (20 minutes)

**Location:** `vtcode-llm-gemini/src/provider.rs`

**Current Error:**
```
error[E0599]: no function or associated item named `with_model` found for struct `GeminiProvider`
  --> vtcode-core/src/llm/client.rs:24:54
   |
24 |         Provider::Gemini => Box::new(GeminiProvider::with_model(
   |                                                      ^^^^^^^^^^ function or associated item not found in `GeminiProvider`
```

**What vtcode-core Expects:**
```rust
// vtcode-core/src/llm/client.rs:24
GeminiProvider::with_model(
    api_key.to_string(),
    model.to_string(),
    config.agent.prompt_cache.clone(),
    config.agent.gemini_base_url.clone(),
)
```

**Your Fix:**

Open `vtcode-llm-gemini/src/provider.rs` and add this method:

```rust
impl GeminiProvider {
    /// Create a new Gemini provider with a specific model
    pub fn with_model(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        // Determine base URL
        let base_url = base_url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("GEMINI_BASE_URL").ok())
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());

        // Extract prompt cache settings
        let (prompt_cache_enabled, prompt_cache_settings) = if let Some(cfg) = prompt_cache {
            let settings = cfg.providers.gemini.clone();
            let enabled = cfg.enabled
                && settings.enabled
                && settings.mode != vtcode_config::GeminiPromptCacheMode::Off;
            (enabled, settings)
        } else {
            (false, vtcode_config::GeminiPromptCacheSettings::default())
        };

        Self {
            api_key,
            http_client: HttpClient::new(),
            base_url,
            model,
            prompt_cache_enabled,
            prompt_cache_settings,
        }
    }

    // ... existing code ...
}
```

**Imports You May Need:**
```rust
use vtcode_config::{PromptCachingConfig, GeminiPromptCacheMode, GeminiPromptCacheSettings};
```

**Test:**
```bash
cargo build -p vtcode-llm-gemini
```

**Checkpoint:** Should compile without the "with_model not found" error.

---

### Task 1.2: Add `from_config` Constructor (20 minutes)

**Current Error:**
```
error[E0599]: no function or associated item named `from_config` found for struct `GeminiProvider`
   --> vtcode-core/src/llm/factory.rs:232:34
    |
232 |         Box::new(GeminiProvider::from_config(
    |                                  ^^^^^^^^^^^ function or associated item not found in `GeminiProvider`
```

**What vtcode-core Expects:**
```rust
// vtcode-core/src/llm/factory.rs:232
GeminiProvider::from_config(
    model.to_string(),
    config,
)
```

**Your Fix:**

Add this method to `GeminiProvider` impl block:

```rust
impl GeminiProvider {
    /// Create a new Gemini provider from config
    pub fn from_config(model: String, config: &vtcode_config::Config) -> anyhow::Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .map_err(|_| anyhow::anyhow!(
                "GEMINI_API_KEY or GOOGLE_API_KEY environment variable not set"
            ))?;

        Ok(Self::with_model(
            api_key,
            model,
            config.agent.prompt_cache.clone(),
            config.agent.gemini_base_url.clone(),
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
cargo build -p vtcode-llm-gemini
```

**Checkpoint:** Should compile without the "from_config not found" error.

---

### Task 1.3: Verify LLMProvider Trait Implementation (15 minutes)

**Current Error:**
```
error[E0277]: the trait bound `GeminiProvider: llm::provider::LLMProvider` is not satisfied
   --> vtcode-core/src/llm/factory.rs:223:26
    |
223 | impl BuiltinProvider for GeminiProvider {
    |                          ^^^^^^^^^^^^^^ the trait `llm::provider::LLMProvider` is not implemented for `GeminiProvider`
```

**What This Means:**
The `GeminiProvider` struct should already have an `impl` block that implements the `LLMProvider` trait from `vtcode-llm-types`. This error suggests either:
1. The trait is implemented but with wrong method signatures
2. The trait implementation uses wrong types

**Your Investigation:**

1. Search for the LLMProvider implementation:
```bash
cd vtcode-llm-gemini
grep -n "impl LLMProvider" src/provider.rs
```

2. Check that the trait uses types from `vtcode_llm_types`:
```rust
// Should look like this:
use vtcode_llm_types::{LLMRequest, LLMResponse, LLMProvider, LLMStream};

#[async_trait::async_trait]
impl LLMProvider for GeminiProvider {
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
cargo build -p vtcode-llm-gemini
cargo build -p vtcode-core
```

**Checkpoint:** Should compile without trait bound errors.

---

### Task 1.4: Run Tests (10 minutes)

**Run the provider's tests:**
```bash
cd vtcode-llm-gemini
cargo test
```

**Run integration with vtcode-core:**
```bash
cd ..
cargo build --all
```

**Expected Result:**
- ✅ All vtcode-llm-gemini tests pass
- ✅ vtcode-core compiles (may still have errors from other providers - that's OK!)
- ✅ No Gemini-related errors

---

### Task 1.5: Commit Your Work (5 minutes)

```bash
git add vtcode-llm-gemini/
git commit -m "fix(gemini): Add missing constructors for vtcode-core integration

Add with_model and from_config constructors to GeminiProvider to support
vtcode-core's provider factory pattern.

Changes:
- Added with_model(api_key, model, prompt_cache, base_url) constructor
- Added from_config(model, config) constructor
- Verified LLMProvider trait implementation uses vtcode_llm_types

Fixes:
- E0599: with_model not found
- E0599: from_config not found
- E0277: LLMProvider trait bound

Testing:
- vtcode-llm-gemini builds cleanly
- vtcode-llm-gemini tests pass
- No Gemini-related errors in vtcode-core

Part of Phase 3 parallel compilation fixes (Agent 1/4)"

git push -u origin phase3-fix-gemini
```

---

## Coordination Points

### With Agent 2 (OpenRouter)

**No coordination needed** - you're working on different crates (vtcode-llm-gemini vs vtcode-llm-openrouter).

**Optional:** Share status updates
```
Agent 1 Update (T+45min):
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
Agent 1 COMPLETE:
✅ GeminiProvider::with_model() ✅
✅ GeminiProvider::from_config() ✅
✅ All tests pass ✅
✅ Branch pushed: phase3-fix-gemini

@Agent4 - You can start using the gemini provider now!
```

---

## Success Criteria

Your task is complete when:

- [ ] ✅ `with_model` constructor added to GeminiProvider
- [ ] ✅ `from_config` constructor added to GeminiProvider
- [ ] ✅ LLMProvider trait properly implemented (uses vtcode_llm_types)
- [ ] ✅ `cargo build -p vtcode-llm-gemini` succeeds
- [ ] ✅ `cargo test -p vtcode-llm-gemini` passes
- [ ] ✅ No Gemini errors when building vtcode-core
- [ ] ✅ Changes committed to `phase3-fix-gemini` branch
- [ ] ✅ Branch pushed to GitHub

---

## Troubleshooting

### Problem: Can't find vtcode_config types

**Solution:**
Check `vtcode-llm-gemini/Cargo.toml` dependencies:
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
cargo build -p vtcode-llm-gemini  # Should pass ✅
```

Your crate works, Agent 4 will fix the imports.

---

## Reference: Expected File Structure

After your changes, `vtcode-llm-gemini/src/provider.rs` should have:

```rust
use vtcode_llm_types::{LLMProvider, LLMRequest, LLMResponse, LLMStream, /* ... */};
use vtcode_config::{PromptCachingConfig, Config, /* ... */};

pub struct GeminiProvider {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
    model: String,
    prompt_cache_enabled: bool,
    prompt_cache_settings: vtcode_config::GeminiPromptCacheSettings,
}

impl GeminiProvider {
    /// Create with specific model
    pub fn with_model(
        api_key: String,
        model: String,
        prompt_cache: Option<PromptCachingConfig>,
        base_url: Option<String>,
    ) -> Self {
        // ... implementation from Task 1.1 ...
    }

    /// Create from config
    pub fn from_config(model: String, config: &Config) -> anyhow::Result<Self> {
        // ... implementation from Task 1.2 ...
    }

    // ... existing helper methods ...
}

#[async_trait::async_trait]
impl LLMProvider for GeminiProvider {
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
Agent 1 (Gemini) - T+[minutes]:
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
| 60-70 min | Run tests | Tests pass |
| 70-75 min | Commit and push | Complete ✅ |

**Total: 1-1.5 hours**

---

**Ready to fix the Gemini provider? Let's go! 🚀**
