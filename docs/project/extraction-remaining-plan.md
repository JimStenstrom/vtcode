# Extraction Remaining Plan

**Date:** 2026-06-14
**Scope:** Mitigate all issues from vtcode-llm / vtcode-skills extraction review
**Status:** Complete

---

## Issue Tracker

| ID | Sev | Issue | Status |
|----|-----|-------|--------|
| H1 | HIGH | vtcode-llm is a dead dependency from vtcode-core | FIXED |
| H2 | HIGH | system_prompt.rs getters ignored OnceLock | FIXED |
| H3 | HIGH | Duplicate assets directory in vtcode-core | FIXED |
| M1 | MEDIUM | ProviderConfig struct duplicated across crates | FIXED |
| M2 | MEDIUM | RetryPolicy duplicated in gemini wire client | FIXED |
| M3 | MEDIUM | ProviderConfig naming confusion (struct vs trait) | FIXED |
| M4 | MEDIUM | Missing feature flags in vtcode-llm Cargo.toml | FIXED |
| L1 | LOW | Redundant explicit re-exports in skills/mod.rs | ACCEPTED |
| L2 | LOW | OnceLock setter silent failure | ACCEPTED |
| L3 | LOW | Pre-existing unsafe warnings in native_plugin.rs | ACCEPTED |

---

## Remaining Work (5 items)

### 1. Consolidate ProviderConfig into vtcode-commons (M1 + M3)

**Problem:** `ProviderConfig` struct is defined identically in vtcode-llm (`provider_config_types.rs`) and vtcode-core (`factory.rs`). Same name is also used for a trait in `config_adapter.rs`. This causes naming confusion and divergence risk.

**Approach:** Move the shared `ProviderConfig` struct to `vtcode-commons` as the canonical definition. Both crates import from there.

**Steps:**
1. Create `vtcode-commons/src/provider_config.rs`:
   ```rust
   use std::path::PathBuf;
   use vtcode_config::TimeoutsConfig;
   use vtcode_config::auth::{CopilotAuthConfig, OpenAIChatGptAuthHandle};
   use vtcode_config::core::{AnthropicConfig, ModelConfig, OpenAIConfig, PromptCachingConfig};

   #[derive(Debug, Clone)]
   pub struct ProviderConfig {
       pub api_key: Option<String>,
       pub openai_chatgpt_auth: Option<OpenAIChatGptAuthHandle>,
       pub copilot_auth: Option<CopilotAuthConfig>,
       pub base_url: Option<String>,
       pub model: Option<String>,
       pub prompt_cache: Option<PromptCachingConfig>,
       pub timeouts: Option<TimeoutsConfig>,
       pub openai: Option<OpenAIConfig>,
       pub anthropic: Option<AnthropicConfig>,
       pub model_behavior: Option<ModelConfig>,
       pub workspace_root: Option<PathBuf>,
   }
   ```
2. Add `pub mod provider_config;` to `vtcode-commons/src/lib.rs`
3. In vtcode-llm: replace `provider_config_types.rs` with `pub use vtcode_commons::provider_config::ProviderConfig;`
4. In vtcode-core `factory.rs`: replace local `ProviderConfig` with `pub use vtcode_commons::provider_config::ProviderConfig;`
5. In vtcode-core `config_adapter.rs`: update `crate::llm::factory::ProviderConfig` references to `vtcode_commons::provider_config::ProviderConfig`
6. Remove the `ProviderConfigData` alias from vtcode-llm's lib.rs (no longer needed)
7. Verify `cargo check --workspace` passes

**Estimated effort:** 30 minutes

---

### 2. Extract shared RetryPolicy to vtcode-commons (M2)

**Problem:** `RetryPolicy` is defined in vtcode-core (`retry.rs`) and duplicated differently in vtcode-llm's gemini wire client. Different fields, different semantics.

**Approach:** Extract a minimal `WireRetryPolicy` into vtcode-commons that both can use. The vtcode-core version keeps its richer API; the wire client uses the simpler version.

**Steps:**
1. Create `vtcode-commons/src/retry.rs` with a shared `RetryPolicy`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct RetryPolicy {
       pub max_retries: u32,
       pub base_delay_ms: u64,
       pub max_delay_ms: u64,
   }

   impl Default for RetryPolicy { /* ... */ }

   pub struct RetryDecision {
       pub retryable: bool,
       pub category: ErrorCategory,
   }

   impl RetryPolicy {
       pub fn classify_anyhow(&self, error: &anyhow::Error) -> RetryDecision { /* ... */ }
       pub fn classify_status(&self, status: u16) -> RetryDecision { /* ... */ }
   }
   ```
2. Add `pub mod retry;` to `vtcode-commons/src/lib.rs`
3. In vtcode-llm gemini wire client: replace local `RetryPolicy` with `use vtcode_commons::retry::RetryPolicy;`
4. In vtcode-core `retry.rs`: either re-export from vtcode-commons or keep as a richer wrapper. Prefer re-export if the API is compatible.
5. Verify `cargo check --workspace` passes

**Estimated effort:** 30 minutes

---

### 3. Move open_responses/ to vtcode-llm

**Problem:** `open_responses/` module (3.7K lines, 10 files) stays in vtcode-core but only depends on `crate::llm::*` (now in vtcode-llm). This is a clean extraction candidate with zero blockers.

**Approach:** Move the entire module to vtcode-llm. The staying provider file (`openresponses/provider.rs`) remains in vtcode-core and imports from `vtcode_llm::open_responses::*`.

**Steps:**
1. Copy `vtcode-core/src/open_responses/` to `vtcode-llm/src/open_responses/`
2. Fix imports: `crate::llm::` → internal `crate::` paths within vtcode-llm
3. Add `pub mod open_responses;` to vtcode-llm's `lib.rs`
4. In vtcode-core's staying `openresponses/provider.rs`: change `crate::open_responses::*` → `vtcode_llm::open_responses::*`
5. Remove `vtcode-core/src/open_responses/` (all files moved)
6. Remove `pub mod open_responses;` from vtcode-core's `llm/mod.rs` (now in vtcode-llm)
7. Add any missing vtcode-llm Cargo.toml deps (currently clean — only `anyhow`, `serde`, `serde_json`)
8. Verify `cargo check --workspace` passes

**Risk:** The staying provider in vtcode-core imports from `crate::llm::providers::common` and `crate::llm::providers::shared` which stay in vtcode-core. These cross-crate references need careful handling — the provider file may need to stay in vtcode-core while the data types move.

**Estimated effort:** 45 minutes

---

### 4. Move copilot/ to vtcode-llm

**Problem:** `copilot/` module (4.4K lines, 8 files) stays in vtcode-core but its only `crate::` dependencies are `crate::llm::*` (now in vtcode-llm) and `crate::config`/`crate::utils` (already extracted crates). The one external blocker is `portable_pty` which is only in vtcode-core's Cargo.toml.

**Approach:** Move the module to vtcode-llm, adding `portable-pty` as an optional dependency behind a `copilot` feature flag.

**Steps:**
1. Add to vtcode-llm Cargo.toml:
   ```toml
   [features]
   copilot = ["dep:portable-pty"]

   [dependencies]
   portable-pty = { version = "0.9.0", optional = true }
   ```
2. Copy `vtcode-core/src/copilot/` to `vtcode-llm/src/copilot/`
3. Fix imports: `crate::llm::provider::*` → internal `crate::provider::*`, `crate::config::*` → `vtcode_config::*`, `crate::utils::*` → `vtcode_commons::*`
4. Add `#[cfg(feature = "copilot")] pub mod copilot;` to vtcode-llm's `lib.rs`
5. In vtcode-core's staying `copilot.rs` provider: change `crate::copilot::*` → `vtcode_llm::copilot::*`
6. Remove `vtcode-core/src/copilot/` (all files moved)
7. In vtcode-core Cargo.toml: enable `vtcode-llm/copilot` feature
8. Verify `cargo check --workspace` passes

**Risk:** `portable_pty` is a system-level dependency (PTY abstraction). It may not compile on all platforms. Feature-gating it ensures non-copilot builds are unaffected.

**Estimated effort:** 45 minutes

---

### 5. Wire vtcode-core to consume vtcode-llm (H1)

**Problem:** After items 3 and 4 move copilot/ and open_responses/ to vtcode-llm, vtcode-core should import these from vtcode-llm instead of keeping local copies. This also addresses the dead-dependency issue.

**Approach:** Update vtcode-core's `llm/mod.rs` to re-export types from vtcode-llm and remove duplicate files.

**Steps:**
1. In `vtcode-core/src/llm/mod.rs`, add selective re-exports for types now in vtcode-llm:
   ```rust
   pub use vtcode_llm::types::{BackendKind, LLMError, LLMResponse};
   pub use vtcode_llm::provider::{FinishReason, LLMStream, LLMStreamEvent, Usage};
   pub use vtcode_llm::capabilities::ProviderCapabilities;
   pub use vtcode_llm::client::{AnyClient, ProviderClientAdapter};
   pub use vtcode_llm::tool_bridge::*;
   pub use vtcode_llm::model_resolver::*;
   pub use vtcode_llm::optimized_client::*;
   pub use vtcode_llm::single_response::collect_single_response;
   ```
2. Remove duplicate files from vtcode-core that are now re-exported:
   - `vtcode-core/src/llm/types.rs`
   - `vtcode-core/src/llm/capabilities.rs`
   - `vtcode-core/src/llm/client.rs`
   - `vtcode-core/src/llm/tool_bridge.rs`
   - `vtcode-core/src/llm/model_resolver.rs`
   - `vtcode-core/src/llm/optimized_client.rs`
   - `vtcode-core/src/llm/single_response.rs`
3. Update module declarations in `vtcode-core/src/llm/mod.rs`
4. Verify `cargo check --workspace` passes
5. Run `cargo test -p vtcode-core --lib` to catch regressions

**Depends on:** Items 3 and 4 (copilot/ and open_responses/ must move first)

**Estimated effort:** 1-2 hours

---

## Execution Order

```
Phase A (parallel, no deps):
  1. Consolidate ProviderConfig (M1+M3)
  2. Extract shared RetryPolicy (M2)
  3. Move open_responses/ to vtcode-llm

Phase B (after Phase A):
  4. Move copilot/ to vtcode-llm (needs portable-pty dep)

Phase C (after Phase B):
  5. Wire vtcode-core to consume vtcode-llm (H1)

Phase D (verification):
  - Audit staying file dependencies
  - Full test suite
```

Items 1, 2, and 3 are independent and can be done in parallel. Item 4 is independent but scheduled after 3 for logical grouping. Item 5 depends on items 3 and 4.

---

## Verification Checklist

After all items are complete:

- [ ] `cargo check --workspace` passes
- [ ] `cargo test -p vtcode-llm` passes
- [ ] `cargo test -p vtcode-skills` passes
- [ ] `cargo test -p vtcode-core --lib` passes
- [ ] `cargo clippy --workspace` has no new warnings
- [ ] No duplicate type definitions across crates (ProviderConfig, RetryPolicy)
- [ ] vtcode-core imports from vtcode-llm (not just a dead dependency)
- [ ] `open_responses/` lives in vtcode-llm, not vtcode-core
- [ ] `copilot/` lives in vtcode-llm behind `copilot` feature flag
- [ ] All staying files in vtcode-core/src/skills/ compile with facade
- [ ] Documentation reflects final state
