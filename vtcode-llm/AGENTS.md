# vtcode-llm

[Root AGENTS.md](../AGENTS.md) | **Canonical** LLM provider trait, types, and implementations.

## Key Modules

`provider/` trait + shared types | `providers/` per-provider impls | `provider.rs` re-exports | `client.rs` + `optimized_client.rs` | `copilot/` (feature-gated) | `open_responses/` | `factory_types.rs` + `provider_config_types.rs` config | `system_prompt.rs` injection | `http_client.rs` | `types.rs` shared types | `utils.rs` + `single_response.rs` + `tool_bridge.rs` + `config_adapter.rs` + `rig_adapter.rs` + `provider_base.rs` + `error_display.rs` + `model_resolver.rs` infra (merged from core)

## Architecture Notes

- **Canonical home** for all provider code. Core's `llm/` is a thin re-export layer + factory/CGP.
- `system_prompt.rs` provides stub getters with `OnceLock` setters; vtcode-core overrides at init.
- Uses `compact_str::CompactString` (aliased `CompactStr` from `vtcode_core::types`) for small string fields.

## Dependencies

`vtcode-commons` (HTTP, CGP, types) | `vtcode-config` (provider config, timeouts) | `vtcode-utility-tool-specs` (schemas) | `vtcode-exec-events` | `vtcode-macros`

## Coding Conventions

Providers in `providers/<name>/mod.rs`. Use `anyhow::Result`, `tracing`, not `println!`. Provider-specific types stay local; shared go in `types.rs` or `provider/`.
