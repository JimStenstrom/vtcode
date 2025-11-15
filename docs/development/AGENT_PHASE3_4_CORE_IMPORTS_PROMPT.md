# Agent 4: Fix vtcode-core Module Structure and Imports - Task Brief

**Your Role:** Fix vtcode-core module declarations and imports after provider extraction
**Estimated Time:** 0.5-1 hour
**Branch Name:** `phase3-fix-core-imports`
**Depends On:** Agent 1 (Gemini) and Agent 2 (OpenRouter) completing their work
**Working in Parallel With:** Agent 3 (Microsoft) - but independent

---

## Your Mission

You are **Agent 4** in a 4-agent parallel execution to fix Phase 3 compilation errors. Your job is to fix **vtcode-core's module structure** after providers were extracted to separate crates.

**IMPORTANT:** You must wait for Agents 1 & 2 to complete before starting. They are adding constructors that you will use.

**Current Status:**
- ❌ vtcode-core tries to import `gemini` module that doesn't exist anymore
- ❌ vtcode-core imports `sanitize_function_parameters` from wrong location
- ❌ Build fails with 3 module/import errors

**Your Goal:**
- ✅ Remove incorrect gemini module declaration
- ✅ Update imports to use vtcode-llm-gemini crate instead of local module
- ✅ Fix sanitize_function_parameters imports
- ✅ All module/import errors resolved
- ✅ Build succeeds

---

## Context: Phase 3 Architecture

Phase 3 extracted provider implementations from vtcode-core into separate crates. However, vtcode-core still has leftover module declarations and imports pointing to the old locations.

**What Changed:**
- Before: `vtcode-core/src/llm/providers/gemini.rs` existed (local module)
- After: Gemini provider moved to `vtcode-llm-gemini/` crate (external dependency)
- Problem: vtcode-core still declares `pub mod gemini;` and imports from it

---

## Pre-Requirements

**WAIT FOR AGENTS 1 & 2 TO COMPLETE!**

Before you start, verify in the coordination channel:
- ✅ Agent 1 has pushed `phase3-fix-gemini` branch
- ✅ Agent 2 has pushed `phase3-fix-openrouter` branch
- ✅ Both agents confirm their constructors work

**Why wait?**
You'll be updating imports to use their constructors. If they're not done, you won't know the correct function signatures.

---

## Your Tasks (0.5-1 hour)

### ✅ Pre-Flight Checklist (5 minutes)

```bash
# Clone repo (if not already)
cd /path/to/vtcode

# Checkout base branch
git checkout claude/main
git pull

# OPTIONAL: Pull Agents 1 & 2 work locally to test (recommended)
git fetch origin
git checkout phase3-fix-gemini
git checkout claude/main
git merge phase3-fix-gemini  # Local test merge
git merge origin/phase3-fix-openrouter  # Local test merge

# Create your branch
git checkout -b phase3-fix-core-imports

# Verify environment
direnv allow .
cargo build 2>&1 | grep -E "gemini|sanitize_function"

# You should see errors like:
# error[E0583]: file not found for module `gemini`
# error[E0432]: unresolved import `crate::llm::providers::gemini`
# error[E0433]: failed to resolve: could not find `gemini` in `providers`
```

---

### Task 4.1: Remove Gemini Module Declaration (5 minutes)

**Location:** `vtcode-core/src/lib.rs`

**Current Error:**
```
error[E0583]: file not found for module `gemini`
   --> vtcode-core/src/lib.rs:135:1
    |
135 | pub mod gemini;
    | ^^^^^^^^^^^^^^^
```

**Problem:**
vtcode-core declares `pub mod gemini;` but the `gemini.rs` or `gemini/` directory no longer exists (it's in vtcode-llm-gemini crate now).

**Your Fix:**

Open `vtcode-core/src/lib.rs` and find line ~135:
```rust
// OLD (WRONG):
pub mod gemini;  // ← Remove this line
```

**Search for it:**
```bash
grep -n "pub mod gemini" vtcode-core/src/lib.rs
```

**Remove the line entirely:**
```rust
// NEW (CORRECT):
// gemini module removed - now in vtcode-llm-gemini crate
```

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "file not found for module.*gemini"
```

**Checkpoint:** No more "file not found for module gemini" errors.

---

### Task 4.2: Fix Gemini Provider Imports (15 minutes)

**Location:** `vtcode-core/src/core/agent/runner.rs`

**Current Errors:**
```
error[E0432]: unresolved import `crate::llm::providers::gemini`
    --> vtcode-core/src/core/agent/runner.rs:1894:36
     |
1894 |         use crate::llm::providers::gemini::sanitize_function_parameters;
     |                                    ^^^^^^ could not find `gemini` in `providers`

error[E0433]: failed to resolve: could not find `gemini` in `providers`
   --> vtcode-core/src/core/agent/runner.rs:715:56
    |
715 |                     parameters: crate::llm::providers::gemini::sanitize_function_parameters(
    |                                                        ^^^^^^ could not find `gemini` in `providers`
```

**Problem:**
Code tries to import `sanitize_function_parameters` from the old local gemini module, but it's now in the vtcode-llm-gemini crate.

**Your Investigation:**

First, check if vtcode-llm-gemini exports this function:
```bash
grep -n "pub fn sanitize_function_parameters" vtcode-llm-gemini/src/*.rs
```

**Possible Scenarios:**

**Scenario A:** Function exists in vtcode-llm-gemini
```rust
// OLD (WRONG):
use crate::llm::providers::gemini::sanitize_function_parameters;

// NEW (CORRECT):
use vtcode_llm_gemini::sanitize_function_parameters;

// Usage:
parameters: sanitize_function_parameters(params),
```

**Scenario B:** Function doesn't exist (was removed)
```rust
// Function was removed during refactoring
// Solution: Remove the sanitize_function_parameters calls
parameters: params,  // Use params directly
```

**Scenario C:** Function should be local to vtcode-core
```rust
// Create a local helper function in runner.rs:
fn sanitize_function_parameters(params: Value) -> Value {
    // Implementation - copy from git history if needed
    params  // Minimal version - just return as-is
}

// Usage:
parameters: sanitize_function_parameters(params),
```

**Your Fix:**

1. **Find ALL usages:**
```bash
cd vtcode-core
grep -rn "gemini::sanitize_function_parameters" src/
```

2. **Choose the correct scenario** (likely A or C)

3. **Update imports and usages:**

For **Scenario A** (function in vtcode-llm-gemini):
```rust
// At top of vtcode-core/src/core/agent/runner.rs:
use vtcode_llm_gemini::sanitize_function_parameters;

// Then later in the file (line 715 and 1894):
// No change needed to usage - just the import
```

For **Scenario C** (make it local):
```rust
// Add somewhere in vtcode-core/src/core/agent/runner.rs or in a helper module:
/// Sanitize function parameters for Gemini provider compatibility
fn sanitize_function_parameters(params: serde_json::Value) -> serde_json::Value {
    // Basic implementation - just pass through
    params

    // OR if you need the full implementation, check:
    // git log --all --full-history -- "**/gemini*" | grep sanitize -A 20
}

// Remove the old import line
```

**Test:**
```bash
cargo build -p vtcode-core 2>&1 | grep "sanitize_function_parameters"
```

**Checkpoint:** No more "sanitize_function_parameters" or "gemini" import errors.

---

### Task 4.3: Verify GeminiProvider Usage (10 minutes)

**Location:** `vtcode-core/src/llm/client.rs` and `vtcode-core/src/llm/factory.rs`

**What to Check:**

Now that Agents 1 & 2 have added constructors, verify the imports are correct:

```bash
cd vtcode-core
grep -n "GeminiProvider\|OpenRouterProvider" src/llm/client.rs src/llm/factory.rs
```

**Expected Pattern:**

```rust
// vtcode-core/src/llm/client.rs or factory.rs

use vtcode_llm_gemini::GeminiProvider;
use vtcode_llm_openrouter::OpenRouterProvider;

// Usage (from Agent 1's work):
GeminiProvider::with_model(api_key, model, prompt_cache, base_url)
GeminiProvider::from_config(model, config)

// Usage (from Agent 2's work):
OpenRouterProvider::with_model(api_key, model, prompt_cache, base_url)
OpenRouterProvider::from_config(model, config)
```

**Your Fix:**

If imports are missing or wrong, add/update them:
```rust
// At the top of the file:
use vtcode_llm_gemini::GeminiProvider;
use vtcode_llm_openrouter::OpenRouterProvider;
```

**Test:**
```bash
cargo build -p vtcode-core
```

**Checkpoint:** vtcode-core builds successfully (or only has Microsoft errors from Agent 3's work).

---

### Task 4.4: Run Full Build (5 minutes)

**Test the entire workspace:**
```bash
cd /path/to/vtcode
cargo build --all
```

**Expected Result:**
- ✅ vtcode-llm-gemini builds ✅ (Agent 1's work)
- ✅ vtcode-llm-openrouter builds ✅ (Agent 2's work)
- ✅ vtcode-core builds ✅ (your work + Agent 3's work)
- ✅ All other crates build ✅

If there are still errors, they should only be in areas you didn't touch (Agent 3's responsibility or other providers).

---

### Task 4.5: Run Tests (5 minutes)

**Run core tests:**
```bash
cd vtcode-core
cargo test --lib
```

**Run integration tests:**
```bash
cd ..
cargo test --all
```

**Expected:**
- Tests should pass (or have the same failures as before your changes)
- No new test failures introduced

---

### Task 4.6: Commit Your Work (5 minutes)

```bash
git add vtcode-core/src/lib.rs vtcode-core/src/core/agent/runner.rs vtcode-core/src/llm/
git commit -m "fix(core): Remove gemini module declaration and fix imports

Update vtcode-core to use extracted provider crates instead of local modules.

Changes:
- Removed pub mod gemini declaration from lib.rs (module extracted to vtcode-llm-gemini)
- Fixed sanitize_function_parameters imports to use vtcode-llm-gemini or local helper
- Verified GeminiProvider and OpenRouterProvider imports use external crates
- Updated all gemini:: references to use vtcode_llm_gemini::

Fixes:
- E0583: file not found for module gemini
- E0432: unresolved import crate::llm::providers::gemini
- E0433: could not find gemini in providers

Testing:
- vtcode-core builds cleanly
- All provider crates integrate correctly
- No gemini module/import errors

Depends on:
- Agent 1: phase3-fix-gemini (GeminiProvider constructors)
- Agent 2: phase3-fix-openrouter (OpenRouterProvider constructors)

Part of Phase 3 parallel compilation fixes (Agent 4/4)"

git push -u origin phase3-fix-core-imports
```

---

## Coordination Points

### With Agent 1 (Gemini) **[REQUIRED]**

**You MUST wait for Agent 1 to finish!**

Before starting:
```
@Agent1 - Are you done? I need:
✅ GeminiProvider::with_model() implemented
✅ GeminiProvider::from_config() implemented
✅ Branch phase3-fix-gemini pushed

Please confirm!
```

After Agent 1 confirms:
```
Agent 4 STARTING:
✅ Agent 1 complete, pulling their work
⏳ Removing gemini module declaration
⏳ Fixing imports
ETA: 30-45 minutes
```

### With Agent 2 (OpenRouter) **[REQUIRED]**

**You MUST wait for Agent 2 to finish!**

Before starting:
```
@Agent2 - Are you done? I need:
✅ OpenRouterProvider::with_model() implemented
✅ OpenRouterProvider::from_config() implemented
✅ Branch phase3-fix-openrouter pushed

Please confirm!
```

### With Agent 3 (Microsoft)

**No direct dependency** - you can work in parallel with Agent 3.

Your changes (module imports) don't affect their changes (field names).

---

## Success Criteria

Your task is complete when:

- [ ] ✅ Waited for Agents 1 & 2 to complete
- [ ] ✅ Removed `pub mod gemini;` from vtcode-core/src/lib.rs
- [ ] ✅ Fixed all `gemini::` imports (use vtcode_llm_gemini:: or local helper)
- [ ] ✅ Verified GeminiProvider imports work
- [ ] ✅ Verified OpenRouterProvider imports work
- [ ] ✅ `cargo build -p vtcode-core` succeeds
- [ ] ✅ `cargo build --all` succeeds
- [ ] ✅ No gemini module/import errors
- [ ] ✅ Changes committed to `phase3-fix-core-imports` branch
- [ ] ✅ Branch pushed to GitHub

---

## Troubleshooting

### Problem: Still can't find gemini module

**Solution:**
Make sure you:
1. Removed the `pub mod gemini;` line from lib.rs
2. Updated ALL imports from `crate::llm::providers::gemini::` to `vtcode_llm_gemini::`

```bash
# Check for remaining bad imports:
grep -rn "crate::llm::providers::gemini" vtcode-core/src/
grep -rn "use.*gemini::" vtcode-core/src/
```

### Problem: vtcode_llm_gemini not found

**Solution:**
Check vtcode-core/Cargo.toml dependencies:
```toml
[dependencies]
vtcode-llm-gemini = { path = "../vtcode-llm-gemini" }
vtcode-llm-openrouter = { path = "../vtcode-llm-openrouter" }
```

If missing, add them.

### Problem: sanitize_function_parameters not found

**Solution:**

**Option A:** Check if it's exported from vtcode-llm-gemini:
```bash
grep -rn "pub fn sanitize_function_parameters" vtcode-llm-gemini/
```

If yes, use: `use vtcode_llm_gemini::sanitize_function_parameters;`

**Option B:** Create a local helper in runner.rs:
```rust
fn sanitize_function_parameters(params: serde_json::Value) -> serde_json::Value {
    params  // Simplest implementation
}
```

### Problem: GeminiProvider::with_model not found

**Solution:**
This means Agent 1 hasn't finished yet. **Wait for them!**

Check the coordination channel for their status.

---

## Reference: Import Pattern

**Before (WRONG):**
```rust
// vtcode-core/src/lib.rs
pub mod gemini;  // ❌ Module doesn't exist

// vtcode-core/src/core/agent/runner.rs
use crate::llm::providers::gemini::sanitize_function_parameters;  // ❌ Wrong path
```

**After (CORRECT):**
```rust
// vtcode-core/src/lib.rs
// gemini module removed - now in vtcode-llm-gemini crate  // ✅

// vtcode-core/src/core/agent/runner.rs
use vtcode_llm_gemini::sanitize_function_parameters;  // ✅ External crate
// OR
fn sanitize_function_parameters(params: Value) -> Value { params }  // ✅ Local helper
```

---

## Dependency Graph

```
Agent 4 (You)
     ↑
     │ DEPENDS ON
     ├─────────────┬─────────────┐
     ↓             ↓             ↓
  Agent 1      Agent 2      Agent 3
  (Gemini)   (OpenRouter)  (Microsoft)
  [REQUIRED]  [REQUIRED]   [OPTIONAL]

You MUST wait for Agents 1 & 2
You CAN run in parallel with Agent 3
```

---

## Communication

**GitHub Issue:** [Link to coordination issue]

**Status Update Template:**
```
Agent 4 (Core Imports) - T+[minutes]:
✅ Agents 1 & 2 confirmed complete
✅ Completed: [task]
⏳ In progress: [task]
ETA: [minutes]
```

**Before Starting (REQUIRED):**
```
Agent 4 STATUS CHECK:
❓ @Agent1 - Have you pushed phase3-fix-gemini?
❓ @Agent2 - Have you pushed phase3-fix-openrouter?

WAITING for confirmations before starting...
```

---

## Timeline

| Time | Activity | Checkpoint |
|------|----------|------------|
| T+0 | WAIT for Agents 1 & 2 | Confirmations received |
| T+0-5 min | Setup, branch creation | Branch ready |
| T+5-10 min | Remove gemini module | Module removed |
| T+10-25 min | Fix imports | Imports updated |
| T+25-35 min | Verify usage | Providers work |
| T+35-40 min | Full build | Build succeeds |
| T+40-45 min | Run tests | Tests pass |
| T+45-50 min | Commit and push | Complete ✅ |

**Total: 0.5-1 hour** (after Agents 1 & 2 finish)

---

## Critical Reminder

**DO NOT START UNTIL AGENTS 1 & 2 ARE DONE!**

Your work depends on their constructors existing. If you start too early:
- ❌ Imports will fail (functions don't exist yet)
- ❌ You'll waste time debugging
- ❌ You might implement wrong solutions

**Wait for the "Agent X COMPLETE" messages in the coordination channel!**

---

**Ready to fix the core imports? Let's go! 🚀**

(But seriously, wait for Agents 1 & 2 first! ⏳)
