# Agent 3: Fix Microsoft Provider Field Mismatches - Task Brief

**Your Role:** Fix Microsoft DirectLine provider field mismatches and missing fields
**Estimated Time:** 1-1.5 hours
**Branch Name:** `phase3-fix-microsoft`
**Working in Parallel With:** Agent 1 (Gemini), Agent 2 (OpenRouter)

---

## Your Mission

You are **Agent 3** in a 4-agent parallel execution to fix Phase 3 compilation errors. Your job is to fix the **Microsoft DirectLine provider** in vtcode-core to match the updated `vtcode-llm-types` field names and struct definitions.

**Current Status:**
- ❌ Microsoft provider uses old field names (`input_tokens` → should be `prompt_tokens`)
- ❌ Missing required struct fields (`origin_tool`, `reasoning`, `reasoning_details`, `parallel_tool_config`)
- ❌ Type mismatches (Option<String> vs String)
- ❌ Build fails with ~13 errors in Microsoft provider

**Your Goal:**
- ✅ Update all field names to match vtcode-llm-types
- ✅ Add all missing struct fields
- ✅ Fix type mismatches
- ✅ Add missing constructor (`from_config`)
- ✅ All Microsoft-related compilation errors resolved
- ✅ Tests pass

---

## Context: Phase 3 Architecture

VTCode completed Phase 3 refactoring where common LLM types were extracted to `vtcode-llm-types`. The Microsoft provider in vtcode-core wasn't fully updated to use the new field names and struct definitions.

**What Changed:**
- Before: Types had fields like `input_tokens`, `output_tokens`, `cached_prompt`
- After: Types now have `prompt_tokens`, `completion_tokens`, no `cached_prompt`
- Before: Message and LLMRequest had fewer fields
- After: New required fields: `origin_tool`, `reasoning`, `reasoning_details`, `parallel_tool_config`

---

## Required Reading

**IMPORTANT:** Quickly review these to understand the correct structure:

1. **vtcode-llm-types/src/request.rs** - LLMRequest, Message, ToolCall definitions (you already have access!)
2. **vtcode-llm-types/src/response.rs** - LLMResponse, Usage definitions
3. **vtcode-llm-types/src/message.rs** - Message, ContentPart definitions

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
git checkout -b phase3-fix-microsoft

# Verify environment
direnv allow .
cargo build 2>&1 | grep -A 2 "microsoft.rs"

# You should see errors like:
# error[E0609]: no field `input_tokens` on type `llm::provider::Usage`
# error[E0609]: no field `output_tokens` on type `llm::provider::Usage`
# error[E0560]: struct `llm::provider::LLMResponse` has no field named `cached_prompt`
# error[E0063]: missing fields `origin_tool`, `reasoning` and `reasoning_details`
```

---

### Task 3.1: Fix Usage Field Names (15 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Errors:**
```
error[E0609]: no field `input_tokens` on type `llm::provider::Usage`
   --> vtcode-core/src/llm/providers/microsoft.rs:530:34
    |
530 |                 prompt_tokens: u.input_tokens,
    |                                  ^^^^^^^^^^^^ unknown field

error[E0609]: no field `output_tokens` on type `llm::provider::Usage`
   --> vtcode-core/src/llm/providers/microsoft.rs:531:38
    |
531 |                 completion_tokens: u.output_tokens,
    |                                      ^^^^^^^^^^^^^ unknown field
```

**Problem:**
The code is trying to read `input_tokens` and `output_tokens` from Usage, but vtcode-llm-types defines them as `prompt_tokens` and `completion_tokens`.

**Your Fix:**

Find this code around line 530:
```rust
// OLD (WRONG):
Usage {
    prompt_tokens: u.input_tokens,
    completion_tokens: u.output_tokens,
    total_tokens: u.input_tokens + u.output_tokens,
}
```

Replace with:
```rust
// NEW (CORRECT):
Usage {
    prompt_tokens: u.prompt_tokens,
    completion_tokens: u.completion_tokens,
    total_tokens: u.prompt_tokens + u.completion_tokens,
}
```

**Search for ALL occurrences:**
```bash
cd vtcode-core/src/llm/providers
grep -n "input_tokens\|output_tokens" microsoft.rs
```

Replace **all** instances:
- `input_tokens` → `prompt_tokens`
- `output_tokens` → `completion_tokens`

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "input_tokens\|output_tokens"
```

**Checkpoint:** No more "input_tokens" or "output_tokens" errors.

---

### Task 3.2: Remove `cached_prompt` Field (5 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Error:**
```
error[E0560]: struct `llm::provider::LLMResponse` has no field named `cached_prompt`
   --> vtcode-core/src/llm/providers/microsoft.rs:453:33
    |
453 |                    cached_prompt: None,
    |                    ^^^^^^^^^^^^^ `llm::provider::LLMResponse` does not have this field
```

**Problem:**
LLMResponse no longer has a `cached_prompt` field (it was removed in Phase 3).

**Your Fix:**

Find this code around line 453:
```rust
// OLD (WRONG):
LLMResponse {
    content,
    cached_prompt: None,  // ← Remove this line
    // ...
}
```

Replace with:
```rust
// NEW (CORRECT):
LLMResponse {
    content,
    // cached_prompt removed - no longer in vtcode-llm-types
    // ...
}
```

**Search for it:**
```bash
grep -n "cached_prompt" vtcode-core/src/llm/providers/microsoft.rs
```

Remove the line entirely.

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "cached_prompt"
```

**Checkpoint:** No more "cached_prompt" errors.

---

### Task 3.3: Add Missing Message Fields (20 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Error:**
```
error[E0063]: missing fields `origin_tool`, `reasoning` and `reasoning_details` in initializer of `llm::provider::Message`
   --> vtcode-core/src/llm/providers/microsoft.rs:507:28
    |
507 |             messages: vec![Message {
    |                            ^^^^^^^ missing `origin_tool`, `reasoning` and `reasoning_details`
```

**Problem:**
The Message struct now requires three additional fields that weren't present before.

**Your Fix:**

Find the Message construction around line 507:
```rust
// OLD (WRONG):
messages: vec![Message {
    role: "user".to_string(),
    content: vec![ContentPart::Text {
        text: prompt.to_string(),
    }],
    // Missing: origin_tool, reasoning, reasoning_details
}]
```

Replace with:
```rust
// NEW (CORRECT):
messages: vec![Message {
    role: "user".to_string(),
    content: vec![ContentPart::Text {
        text: prompt.to_string(),
    }],
    origin_tool: None,           // ← Add this
    reasoning: None,             // ← Add this
    reasoning_details: None,     // ← Add this
}]
```

**Search for ALL Message constructions:**
```bash
grep -n "Message {" vtcode-core/src/llm/providers/microsoft.rs
```

Add these three fields to **every** Message construction:
```rust
origin_tool: None,
reasoning: None,
reasoning_details: None,
```

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "missing fields.*Message"
```

**Checkpoint:** No more "missing fields" errors for Message.

---

### Task 3.4: Add Missing LLMRequest Field (10 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Error:**
```
error[E0063]: missing field `parallel_tool_config` in initializer of `llm::provider::LLMRequest`
   --> vtcode-core/src/llm/providers/microsoft.rs:506:23
    |
506 |         let request = LLMRequest {
    |                       ^^^^^^^^^^ missing `parallel_tool_config`
```

**Problem:**
LLMRequest now requires a `parallel_tool_config` field.

**Your Fix:**

Find the LLMRequest construction around line 506:
```rust
// OLD (WRONG):
let request = LLMRequest {
    messages,
    system_prompt: None,
    tools: None,
    model: self.model.clone(),
    max_tokens: Some(1000),
    temperature: Some(0.7),
    stream: false,
    tool_choice: None,
    parallel_tool_calls: None,
    // Missing: parallel_tool_config
    reasoning_effort: None,
};
```

Replace with:
```rust
// NEW (CORRECT):
let request = LLMRequest {
    messages,
    system_prompt: None,
    tools: None,
    model: self.model.clone(),
    max_tokens: Some(1000),
    temperature: Some(0.7),
    stream: false,
    tool_choice: None,
    parallel_tool_calls: None,
    parallel_tool_config: None,  // ← Add this
    reasoning_effort: None,
};
```

**Search for ALL LLMRequest constructions:**
```bash
grep -n "LLMRequest {" vtcode-core/src/llm/providers/microsoft.rs
```

Add `parallel_tool_config: None,` to **every** LLMRequest construction.

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "parallel_tool_config"
```

**Checkpoint:** No more "parallel_tool_config" errors.

---

### Task 3.5: Fix ContentPart Field Access (10 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Error:**
```
error[E0609]: no field `text` on type `&ContentPart`
   --> vtcode-core/src/llm/providers/microsoft.rs:276:51
    |
276 |                         if let Some(text) = &part.text {
    |                                                   ^^^^ unknown field
```

**Problem:**
ContentPart is an enum, not a struct. You can't access `.text` directly.

**Your Fix:**

Find the code around line 276:
```rust
// OLD (WRONG):
if let Some(text) = &part.text {
    // ...
}
```

Replace with:
```rust
// NEW (CORRECT):
if let ContentPart::Text { text } = part {
    // Now `text` is the string content
    // ...
}
```

**Understanding ContentPart:**
```rust
// vtcode-llm-types/src/message.rs
pub enum ContentPart {
    Text { text: String },
    // ... other variants ...
}
```

So you need to pattern match, not field access.

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "no field.*text"
```

**Checkpoint:** No more "text field" errors.

---

### Task 3.6: Fix Content Type Mismatches (15 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Errors:**
```
error[E0308]: mismatched types
   --> vtcode-core/src/llm/providers/microsoft.rs:450:33
    |
450 | ...                   content,
    |                       ^^^^^^^ expected `Option<String>`, found `String`

error[E0308]: mismatched types
   --> vtcode-core/src/llm/providers/microsoft.rs:527:22
    |
527 |             content: response.content,
    |                      ^^^^^^^^^^^^^^^^ expected `String`, found `Option<String>`
```

**Problem:**
Type mismatches between String and Option<String>.

**Your Fix:**

**Error 1 (line 450):** Expected `Option<String>`, found `String`
```rust
// OLD (WRONG):
content: some_string_variable,

// NEW (CORRECT):
content: Some(some_string_variable),
// or if the variable might be optional:
content: some_optional_string_variable,
```

**Error 2 (line 527):** Expected `String`, found `Option<String>`
```rust
// OLD (WRONG):
content: response.content,

// NEW (CORRECT):
content: response.content.unwrap_or_default(),
// or for better error handling:
content: response.content.unwrap_or_else(|| "".to_string()),
```

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "mismatched types.*content"
```

**Checkpoint:** No more type mismatch errors for content.

---

### Task 3.7: Add `from_config` Constructor (10 minutes)

**Location:** `vtcode-core/src/llm/providers/microsoft.rs`

**Current Error:**
```
error[E0599]: no function or associated item named `from_config` found for struct `MicrosoftProvider`
   --> vtcode-core/src/llm/factory.rs:430:37
    |
430 |         Box::new(MicrosoftProvider::from_config(
    |                                     ^^^^^^^^^^^ function or associated item not found in `MicrosoftProvider`
```

**What vtcode-core Expects:**
```rust
// vtcode-core/src/llm/factory.rs:430
MicrosoftProvider::from_config(
    model.to_string(),
    config,
)
```

**Your Fix:**

Add this method to the `impl MicrosoftProvider` block:

```rust
impl MicrosoftProvider {
    /// Create a new Microsoft provider from config
    pub fn from_config(model: String, config: &vtcode_config::Config) -> anyhow::Result<Self> {
        // Microsoft DirectLine uses a different authentication approach
        // The API key comes from Azure Bot Service configuration
        let api_key = std::env::var("MICROSOFT_DIRECTLINE_SECRET")
            .or_else(|_| std::env::var("AZURE_BOT_SECRET"))
            .map_err(|_| anyhow::anyhow!(
                "MICROSOFT_DIRECTLINE_SECRET or AZURE_BOT_SECRET environment variable not set"
            ))?;

        // Get base URL from config or use default
        let base_url = config
            .agent
            .microsoft_base_url
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://directline.botframework.com/v3/directline".to_string());

        Ok(Self {
            api_key,
            http_client: HttpClient::new(),
            base_url,
            model,
        })
    }

    // ... existing methods ...
}
```

**Imports You May Need:**
```rust
use anyhow;
use vtcode_config::Config;
```

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "from_config.*Microsoft"
```

**Checkpoint:** No more "from_config not found" errors for MicrosoftProvider.

---

### Task 3.8: Run Tests (10 minutes)

**Build vtcode-core:**
```bash
cd vtcode-core
cargo build
```

**Run tests:**
```bash
cargo test --lib
```

**Run full workspace build:**
```bash
cd ..
cargo build --all
```

**Expected Result:**
- ✅ vtcode-core builds cleanly
- ✅ No Microsoft-related errors
- ✅ Tests pass (or at least no new failures)

---

### Task 3.9: Commit Your Work (5 minutes)

```bash
git add vtcode-core/src/llm/providers/microsoft.rs
git commit -m "fix(microsoft): Update field names and add missing fields for vtcode-llm-types compatibility

Update Microsoft DirectLine provider to match vtcode-llm-types v2 structure.

Changes:
- Fixed Usage field names: input_tokens → prompt_tokens, output_tokens → completion_tokens
- Removed cached_prompt field from LLMResponse (no longer exists)
- Added missing Message fields: origin_tool, reasoning, reasoning_details
- Added missing LLMRequest field: parallel_tool_config
- Fixed ContentPart access to use pattern matching instead of field access
- Fixed content type mismatches (String vs Option<String>)
- Added from_config(model, config) constructor

Fixes:
- E0609: input_tokens/output_tokens field errors (×4)
- E0560: cached_prompt field error
- E0063: missing Message fields
- E0063: missing LLMRequest field
- E0609: ContentPart.text field error
- E0308: content type mismatches (×2)
- E0599: from_config not found

Testing:
- vtcode-core builds cleanly
- No Microsoft-related compilation errors
- All field names match vtcode-llm-types

Part of Phase 3 parallel compilation fixes (Agent 3/4)"

git push -u origin phase3-fix-microsoft
```

---

## Coordination Points

### With Agent 1 (Gemini) & Agent 2 (OpenRouter)

**No coordination needed** - you're working on completely different files.

**Optional:** Share status updates
```
Agent 3 Update (T+45min):
✅ Fixed Usage field names
✅ Added missing Message fields
✅ Added missing LLMRequest fields
⏳ Fixing ContentPart access
ETA: 30 minutes
```

### With Agent 4 (Core Imports)

**No direct dependency** - Agent 4 works on imports, you work on the Microsoft provider file.

**At completion:** Post in coordination channel:
```
Agent 3 COMPLETE:
✅ All Microsoft field mismatches fixed ✅
✅ MicrosoftProvider::from_config() added ✅
✅ vtcode-core builds cleanly ✅
✅ Branch pushed: phase3-fix-microsoft
```

---

## Success Criteria

Your task is complete when:

- [ ] ✅ All Usage field names updated (prompt_tokens, completion_tokens)
- [ ] ✅ `cached_prompt` field removed from LLMResponse
- [ ] ✅ Missing Message fields added (origin_tool, reasoning, reasoning_details)
- [ ] ✅ Missing LLMRequest field added (parallel_tool_config)
- [ ] ✅ ContentPart field access fixed (pattern matching)
- [ ] ✅ Content type mismatches resolved
- [ ] ✅ `from_config` constructor added
- [ ] ✅ `cargo build -p vtcode-core` succeeds
- [ ] ✅ No Microsoft-related errors
- [ ] ✅ Changes committed to `phase3-fix-microsoft` branch
- [ ] ✅ Branch pushed to GitHub

---

## Troubleshooting

### Problem: Still seeing field errors after changes

**Solution:**
Make sure you found ALL occurrences:
```bash
cd vtcode-core/src/llm/providers
grep -n "input_tokens\|output_tokens\|cached_prompt" microsoft.rs
# Should return nothing
```

### Problem: ContentPart pattern matching doesn't work

**Solution:**
Check the exact enum definition:
```rust
use vtcode_llm_types::ContentPart;

// Then pattern match:
match part {
    ContentPart::Text { text } => {
        // use `text` here
    },
    _ => {
        // handle other variants
    }
}
```

### Problem: Can't find HttpClient type

**Solution:**
Check the imports at the top of microsoft.rs:
```rust
use crate::llm::http_client::HttpClient;
// or wherever HttpClient is defined
```

### Problem: from_config signature doesn't match

**Solution:**
Check vtcode-core/src/llm/factory.rs to see exactly how it's called:
```bash
grep -A 5 "MicrosoftProvider::from_config" vtcode-core/src/llm/factory.rs
```

Match the exact signature.

---

## Reference: Field Name Mapping

| Old Field Name | New Field Name | Type |
|----------------|----------------|------|
| `input_tokens` | `prompt_tokens` | usize |
| `output_tokens` | `completion_tokens` | usize |
| `cached_prompt` | [REMOVED] | N/A |
| N/A | `origin_tool` | Option<String> |
| N/A | `reasoning` | Option<String> |
| N/A | `reasoning_details` | Option<Value> |
| N/A | `parallel_tool_config` | Option<ParallelToolConfig> |

---

## Communication

**GitHub Issue:** [Link to coordination issue]

**Status Update Template:**
```
Agent 3 (Microsoft) - T+[minutes]:
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
| 5-20 min | Fix Usage field names | No token field errors |
| 20-25 min | Remove cached_prompt | No cached_prompt errors |
| 25-45 min | Add Message fields | No missing field errors |
| 45-55 min | Add LLMRequest field | No parallel_tool_config errors |
| 55-65 min | Fix ContentPart access | No text field errors |
| 65-80 min | Fix type mismatches | No type errors |
| 80-90 min | Add from_config | All errors fixed |
| 90-100 min | Run tests | Tests pass |
| 100-105 min | Commit and push | Complete ✅ |

**Total: 1-1.5 hours**

---

**Ready to fix the Microsoft provider? Let's go! 🚀**
