# Web Fetch Tool Registration & HITL Popup Fix

## Problem

The web_fetch tool was not available as a builtin tool in vtcode. When users tried to use it, the tool was blocked by sandbox restrictions without showing the Human-In-The-Loop (HITL) confirmation popup.

## Root Cause Analysis

### Issue 1: Tool Not Registered

The `WebFetchTool` was implemented in `vtcode-core/src/tools/web_fetch/` but was never registered in the builtin tool registry (`builtins.rs`). Without registration, the tool could not be invoked through the normal tool pipeline.

### Issue 2: HITL Popup Not Showing

The safety validation pipeline had a two-phase design:

1. **Phase 1 (Safety Check)**: `check_tool_safety()` evaluates risk via `SafetyGateway`
2. **Phase 2 (Permission Check)**: `check_tool_permission()` shows HITL popup via `ToolPolicyGateway`

When `SafetyError::NeedsApproval` was returned (intended to signal "force a prompt"), it was being treated as a terminal error in the validation pipeline, preventing Phase 2 from executing. The structured intent in `NeedsApproval` was being swallowed by the generic error handler.

## Solution

### 1. Register WebFetchTool as Builtin

**File**: `vtcode-core/src/tools/registry/builtins.rs`

Added `register_web_fetch` function with:
- `#[distributed_slice(BUILTIN_TOOLS)]` macro for auto-registration
- Full parameter schema (url, prompt, max_bytes, timeout_secs)
- Tool description for LLM context
- Permission policy: `ToolPolicy::Prompt` (requires HITL approval)
- Aliases: `["fetch_url", "web"]`
- Capability level: `CapabilityLevel::Basic`

### 2. Propagate SafetyError::NeedsApproval Through Pipeline

**File**: `src/agent/runloop/unified/tool_pipeline/validation.rs`

Added `NeedsApproval(String)` variant to `SafetyValidationFailure` enum:

```rust
pub(crate) enum SafetyValidationFailure {
    SessionLimitNotIncreased,
    SessionLimitPromptFailed(anyhow::Error),
    /// The safety gateway requires human approval before this tool call can proceed.
    NeedsApproval(String),
    Validation(SafetyError),
}
```

Added match arm in `validate_tool_call_with_limit_prompt()`:

```rust
Err(SafetyError::NeedsApproval(justification)) => {
    return Err(SafetyValidationFailure::NeedsApproval(justification));
}
```

### 3. Forward Justification in All Three Execution Paths

#### Path 1: Main Interactive (`execution_run.rs`)

```rust
Err(SafetyValidationFailure::NeedsApproval(justification)) => Ok(Some(justification)),
```

Returns `Ok(Some(justification))` which flows to `check_tool_permission()` where the HITL popup is shown.

#### Path 2: Turn Processing (`handlers/mod.rs`)

Modified `run_safety_validation_loop` return type to `Result<Option<(ValidationResult, Option<String>)>>`:

```rust
Err(SafetyValidationFailure::NeedsApproval(justification)) => {
    Ok(Some((ValidationResult::Handled, Some(justification))))
}
```

Updated caller to capture justification and pass to permission context:

```rust
let mut safety_approval_justification = None;
if let Some((outcome, justification)) =
    run_safety_validation_loop(ctx, tool_call_id, &canonical_tool_name, effective_args).await?
{
    safety_approval_justification = justification;
    if matches!(outcome, ValidationResult::Blocked) {
        return Ok(outcome);
    }
}
let permission_result = ensure_tool_permission_with_call_id(
    build_tool_permissions_context_with_safety(ctx, safety_approval_justification.as_deref()),
    // ...
```

Added `build_tool_permissions_context_with_safety` function that accepts `safety_approval_justification: Option<&str>`.

#### Path 3: Copilot Runtime (`copilot_runtime.rs`)

Modified `prepare_vtcode_tool_execution` to store justification:

```rust
let safety_approval_justification = match validate_tool_call_with_limit_prompt(...).await {
    Ok(()) => None,
    Err(SafetyValidationFailure::NeedsApproval(justification)) => Some(justification),
    // ... other error arms return early with denied_tool_response
};
match ensure_tool_permission_with_call_id(
    self.tool_permissions_context_with_safety(renderer, safety_approval_justification.as_deref()),
    // ...
```

Added `tool_permissions_context_with_safety` method.

### 4. Tool Intent Classification

**File**: `vtcode-core/src/tools/tool_intent.rs`

Added web_fetch to ReadOnly behavior group:

```rust
tools::WEB_FETCH | tools::FETCH_URL => Some(ToolBehavior::function(
    ToolMutationModel::ReadOnly,
    false,  // requires_planning
    false,  // supports_parallel_calls
)),
```

### 5. Safety Caps

**File**: `vtcode-core/src/tools/web_fetch/mod.rs`

Added hard cap constants to prevent abuse:

```rust
const MAX_ALLOWED_BYTES: usize = 2_000_000; // 2MB hard cap
const MAX_ALLOWED_TIMEOUT_SECS: u64 = 120; // 2 minutes hard cap
```

Values are clamped in `run()`:

```rust
let max_bytes = args.max_bytes.map(|v| v.min(MAX_ALLOWED_BYTES)).unwrap_or(DEFAULT_CONTENT_SIZE);
let timeout_secs = args.timeout_secs.map(|v| v.min(MAX_ALLOWED_TIMEOUT_SECS)).unwrap_or(DEFAULT_TIMEOUT_SECS);
```

### 6. Domain-Scoped Permanent Approval Caching

When a user selects "Always approve and save to policy cache" for `web_fetch`, the approval is now scoped to the specific domain rather than the entire tool. This means:

- Approving `https://example.com` does NOT auto-approve `https://other.com`
- Approving `https://example.com` DOES auto-approve `https://example.com/any/path`

#### Cache Key Generation

**File**: `src/agent/runloop/unified/tool_routing/approval_cache.rs`

Added `web_fetch_domain()` helper that extracts the hostname from the URL argument:

```rust
pub(super) fn web_fetch_domain(tool_args: Option<&Value>) -> Option<String> {
    let url = tool_args?.as_object()?.get("url")?.as_str()?;
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    if host.is_empty() { return None; }
    Some(host.to_ascii_lowercase())
}
```

Updated `cache_key()` to use domain-based keys for web_fetch:

```rust
if (tool_name == WEB_FETCH || tool_name == FETCH_URL)
    && let Some(domain) = web_fetch_domain(tool_args)
{
    return format!("{tool_name}:{domain}");
}
```

#### Approval Learning Target

**File**: `src/agent/runloop/unified/tool_routing/shell_approval.rs`

Updated `approval_learning_target()` to use domain-based keys:

```rust
if (tool_name == WEB_FETCH || tool_name == FETCH_URL)
    && let Some(domain) = web_fetch_domain(tool_args)
{
    let approval_key = format!("{tool_name}:{domain}");
    let display_label = format!("fetch from {domain}");
    return ApprovalLearningTarget::new(approval_key, display_label);
}
```

#### Persistent Approval Target

Updated `persistent_approval_target()` to return `ExactInvocation` for web_fetch:

```rust
if (tool_name == WEB_FETCH || tool_name == FETCH_URL)
    && let Some(domain) = web_fetch_domain(tool_args)
{
    return PersistentApprovalTarget::ExactInvocation {
        display_label: format!("fetch from {domain}"),
    };
}
```

This ensures the HITL popup shows "Always approve and save to policy cache" with the subtitle "Remember approval for fetch from example.com in this workspace".

#### Exact Shell Approval Target

Updated `exact_shell_approval_target()` to return the domain-scoped target:

```rust
if (tool_name == WEB_FETCH || tool_name == FETCH_URL)
    && let Some(domain) = web_fetch_domain(tool_args)
{
    let approval_key = format!("{tool_name}:{domain}");
    let display_label = format!("fetch from {domain}");
    return Some(ApprovalLearningTarget::new(approval_key, display_label));
}
```

This ensures persisted approval lookups match the domain-specific key.

## Testing

- All existing tests pass (0 failures)
- Added test `max_bytes_and_timeout_are_clamped_to_hard_caps` for safety caps
- Binary compiles cleanly (only minor dead_code warnings for test-only functions)

## Files Changed

| File | Change |
|------|--------|
| `vtcode-core/src/tools/registry/builtins.rs` | Register WebFetchTool as builtin |
| `vtcode-core/src/tools/tool_intent.rs` | Classify as ReadOnly behavior |
| `vtcode-core/src/tools/web_fetch/mod.rs` | Add safety caps |
| `src/agent/runloop/unified/tool_pipeline/validation.rs` | Add NeedsApproval variant |
| `src/agent/runloop/unified/tool_pipeline/execution_run.rs` | Forward justification |
| `src/agent/runloop/unified/turn/tool_outcomes/handlers/mod.rs` | Forward justification |
| `src/agent/runloop/unified/turn/turn_processing/llm_request/copilot_runtime.rs` | Forward justification |
| `src/agent/runloop/unified/tool_routing/approval_cache.rs` | Domain-based cache keys for web_fetch |
| `src/agent/runloop/unified/tool_routing/shell_approval.rs` | Domain-scoped approval targets for web_fetch |

## How It Works Now

1. User invokes `fetch https://example.com/`
2. Tool enters safety validation pipeline
3. `SafetyGateway` evaluates risk (score: 40 for network tool)
4. `NeedsApproval` justification is returned
5. Justification flows through to permission check
6. `ToolPolicyGateway` evaluates policy (Prompt for web_fetch)
7. HITL popup appears asking user to approve/deny with options:
   - **Approve Once**: Allow this time only
   - **Allow for Session**: Allow for current session
   - **Always approve and save to policy cache**: Remember for `example.com` in this workspace
   - **Deny Once**: Reject this time (ask again next time)
8. User selects "Always approve" → approval cached under `web_fetch:example.com`
9. Next call to `fetch https://example.com/other` → auto-approved (same domain)
10. Call to `fetch https://other.com/` → still prompts (different domain)

## Notes

- The `NeedsApproval` variant was already designed for this purpose but was being swallowed by error handling
- Three execution paths needed fixes because vtcode has multiple code paths for tool invocation
- Domain-scoped caching prevents over-broad auto-approval while reducing prompt fatigue for trusted domains
- The domain key is normalized to lowercase so `https://Example.COM/` and `https://example.com/` share one cache entry
