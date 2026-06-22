cap max 5 concurrent subagents at a time.

---

https://addyosmani.com/blog/loop-engineering/

---

idea: combine identical success tool calls. example:

```
Read AppFeatures/CTInsertAd/CTInsertAd/RevampV3/ViewController/IAV3ParamReviewViewController.swift (542 lines)
Read ChoTot/Features/NewInsertAd/CTLightAdManager+PostToListing.swift (44 lines)
Read AppFeatures/CTInsertAd/CTInsertAd/NewInsertAd/Assembler/IAViewModelAssembler.swift (350 lines)
Read Libraries/CTApiClient/AGENTS.md (100 lines)
Read Libraries/CTApiClient/CLAUDE.md (6 lines)
Read Libraries/CTCommon/AGENTS.md (109 lines)
```

---

check and prevent

https://news.ycombinator.com/item?id=48626930

also: https://news.ycombinator.com/item?id=48628777

---

---

https://github.com/vinhnx/VTCode/issues/675

# Implementation Plan: Custom model lists for built-in providers

## Problem

Built-in providers (opencode-zen, opencode-go) hardcode their model list in the Rust `ModelId` enum. Users cannot add custom models to these providers via `vtcode.toml`, unlike `[[custom_providers]]` which supports user-defined `models = [...]`.

## Proposed solution

Add a `[providers]` config section that allows overriding model lists for built-in providers, and optionally adding entirely new models with custom base URLs.

## Implementation steps

### Phase 1: Config schema (vtcode-config)

1. **Define provider override config struct** in `vtcode-config/src/core/provider_override.rs`:
    - `ProviderOverrideConfig` with fields: `models: Vec<String>`, `base_url: Option<String>`, `api_key_env: Option<String>`

2. **Add to VTCodeConfig** in `vtcode-config/src/loader/config.rs`:
    - New field: `pub provider_overrides: BTreeMap<String, ProviderOverrideConfig>` (serde keyed by provider name)
    - Deserialize from `[providers.opencode-zen]` etc.

3. **Update JSON schema export** if schema feature is enabled.

### Phase 2: Model resolution (vtcode-config + vtcode-core)

4. **Extend ModelId** to support runtime-defined variants:
    - Add `ModelId::Custom { provider: CompactStr, model: CompactStr }` variant
    - Implement all existing match arms for it (as_str, display, description, parse, provider, capabilities, defaults, collection)

5. **Add override-aware model collection** in `vtcode-config/src/models/model_id/`:
    - New function `all_models_with_overrides(overrides: &BTreeMap<...>) -> Vec<ModelId>`
    - Merges hardcoded models with user-defined models from overrides
    - For providers with `base_url` override, route custom models to custom endpoint

### Phase 3: Provider routing (vtcode-core)

6. **Update LLM factory** in `vtcode-core/src/llm/factory.rs`:
    - After `register_custom_providers()`, apply provider overrides
    - For overridden providers, register the custom models pointing to the configured base URL

7. **Update model resolver** in `vtcode-core/src/llm/model_resolver.rs`:
    - Handle `ModelId::Custom` in `heuristic_provider_from_model`
    - Route custom variants to their overridden provider endpoint

### Phase 4: Model picker (vtcode binary)

8. **Update model picker** in `src/agent/runloop/model_picker/`:
    - `options.rs`: Include user-defined custom ModelIds from overrides
    - `selection.rs`: Handle `Custom` variant display

### Phase 5: Documentation

9. **Document the new config surface** in `docs/guides/` and crate AGENTS.md files.

## Key files to modify

| File                                                | Change                                           |
| --------------------------------------------------- | ------------------------------------------------ |
| `vtcode-config/src/loader/config.rs`                | Add `provider_overrides` field to `VTCodeConfig` |
| `vtcode-config/src/core/provider_override.rs`       | New: `ProviderOverrideConfig` struct             |
| `vtcode-config/src/models/model_id/collection.rs`   | Add override-aware model collection              |
| `vtcode-config/src/models/model_id/enum.rs`         | Add `Custom` variant                             |
| `vtcode-config/src/models/model_id/as_str.rs`       | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/display.rs`      | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/description.rs`  | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/parse.rs`        | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/provider.rs`     | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/capabilities.rs` | Handle `Custom`                                  |
| `vtcode-config/src/models/model_id/defaults.rs`     | Handle `Custom`                                  |
| `vtcode-core/src/llm/factory.rs`                    | Apply overrides after custom providers           |
| `vtcode-core/src/llm/model_resolver.rs`             | Route `Custom` model IDs                         |
| `vtcode-core/src/models_manager/model_presets.rs`   | Dynamic preset generation for `Custom`           |
| `src/agent/runloop/model_picker/options.rs`         | Include custom models                            |
| `src/agent/runloop/model_picker/selection.rs`       | Handle `Custom` display                          |
| `vtcode-config/AGENTS.md`                           | Document override config                         |

## Usage (after implementation)

```toml
[providers.opencode-zen]
models = [
    "opencode/gpt-5.4",
    "opencode/gpt-5.4-mini",
    "opencode/glm-5.1",
    "my-custom-model",
]
base_url = "https://custom-endpoint.example.com"
api_key_env = "MY_CUSTOM_KEY"
```
