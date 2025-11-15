# Tool Registry Design Documentation

This directory contains comprehensive design documentation for building tool registries that provide the best experience for agentic LLMs and their users.

## Quick Navigation

### 📚 Core Documents

1. **[TOOL_REGISTRY_DESIGN.md](./TOOL_REGISTRY_DESIGN.md)** - Complete architectural design
   - Core principles (semantic clarity, appropriate granularity, fail-safe defaults)
   - Tool registration schema with rich metadata
   - Categorization and discovery mechanisms
   - Policy system architecture
   - Implementation checklist and success metrics

2. **[TOOL_REGISTRY_EXAMPLES.md](./TOOL_REGISTRY_EXAMPLES.md)** - Practical implementation examples
   - Basic tool registration patterns
   - Rich metadata configuration
   - LLM-optimized documentation
   - Policy configuration examples
   - Anti-pattern detection
   - Alternative suggestion systems

3. **[TOOL_BEST_PRACTICES.md](./TOOL_BEST_PRACTICES.md)** - Best practices and anti-patterns
   - Tool design principles
   - Common anti-patterns with corrections
   - LLM guidance patterns
   - Performance optimization
   - Safety and security
   - Error handling strategies
   - Evolution and deprecation

4. **[TOOL_POLICY_VS_SOP.md](./TOOL_POLICY_VS_SOP.md)** - Policy vs. SOP distinction
   - Core differences between policies and procedures
   - When to use each
   - How they interact
   - Real-world scenarios
   - Configuration examples

## Key Concepts

### Tool Registry Philosophy

The tool registry is **not just a function catalog** - it's a decision-support system for LLMs. The registry should:

1. **Guide decision-making** through rich metadata
2. **Ensure safety** through risk classification and policies
3. **Optimize performance** through caching and profiling
4. **Enable discovery** through categorization
5. **Support evolution** through versioning

### Core Design Principles

#### 1. Semantic Over Generic
Tools should express **clear intent**, not generic capabilities.
- ✅ `Read`, `Edit`, `Grep` (semantic)
- ❌ `FileOperation(mode="read")` (generic)

#### 2. Appropriate Granularity
Neither too fine-grained nor too coarse.
- Too fine: `ReadLine`, `ReadParagraph`, `ReadFunction`
- Too coarse: `DoAnything(operation, params)`
- Just right: `Read(file_path, offset?, limit?)`

#### 3. Fail-Safe Defaults
Design for safety when parameters are omitted.
- Read entire file by default (not first line)
- Deny risky operations by default
- Non-destructive modes as defaults

#### 4. Rich Decision Support
Provide context beyond "what the tool does":
- When to use / when NOT to use
- Anti-patterns and alternatives
- Cost and risk indicators
- Clear examples

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Tool Registry                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Inventory  │  │    Policy    │  │   Telemetry  │      │
│  │              │  │   Gateway    │  │              │      │
│  │ - Tools map  │  │ - Policies   │  │ - Metrics    │      │
│  │ - Aliases    │  │ - Risk score │  │ - Audit log  │      │
│  │ - Cache      │  │ - Approval   │  │ - Analytics  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Tool Registration                       │  │
│  ├──────────────────────────────────────────────────────┤  │
│  │ • Core Identity (name, category, capability)        │  │
│  │ • Metadata (description, examples, decision trees)  │  │
│  │ • Risk Profile (risk level, factors, audit level)   │  │
│  │ • Runtime Profile (latency, caching, parallelism)   │  │
│  │ • Lifecycle (version, stability, deprecation)       │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Tool Registration Structure

```rust
pub struct ToolRegistration {
    // Core Identity
    name: &'static str,
    category: ToolCategory,
    capability_level: CapabilityLevel,

    // Metadata for LLM Decision-Making
    metadata: ToolMetadata {
        description: &'static str,
        when_to_use: Vec<String>,
        when_not_to_use: Vec<String>,
        alternatives: Vec<ToolAlternative>,
        examples: Vec<ToolExample>,
        parameters_schema: Value,
        error_patterns: Vec<ErrorPattern>,
    },

    // Safety & Policy
    risk_profile: RiskProfile {
        risk_level: RiskLevel,
        risk_factors: Vec<RiskFactor>,
        requires_confirmation: bool,
        audit_level: AuditLevel,
    },

    // Runtime Characteristics
    runtime_profile: RuntimeProfile {
        expected_latency: LatencyCategory,
        cacheable: bool,
        can_run_parallel: bool,
        timeout_category: ToolTimeoutCategory,
    },

    // Lifecycle
    lifecycle: ToolLifecycle {
        status: ToolStatus,
        version: ToolVersion,
        stability: StabilityLevel,
        deprecation: Option<DeprecationInfo>,
    },
}
```

## Quick Reference Tables

### Tool Categories

| Category | Purpose | Risk Level | Examples |
|----------|---------|------------|----------|
| FileRead | Read file contents | Low | Read, Grep, List |
| FileWrite | Modify files | Medium | Write, Edit, ApplyPatch |
| ShellExecution | Execute commands | High | Bash, RunCommand |
| NetworkAccess | External requests | High | WebFetch, API calls |
| TaskManagement | Agent coordination | Low | TodoWrite, Task |

### Risk Levels

| Level | Description | Default Policy | Example Operations |
|-------|-------------|----------------|-------------------|
| Low | Read-only, no side effects | Allow | Read file, search |
| Medium | Create/modify within workspace | Prompt | Write file, edit |
| High | Execute code, network access | Prompt | Bash, web_fetch |
| Critical | System changes, destructive | Deny | rm -rf, format |

### Policy Types

| Policy | Behavior | Use Case |
|--------|----------|----------|
| Allow | Auto-approve | Safe, read-only operations |
| Prompt | Ask user | Modifications, execution |
| Deny | Block | Dangerous operations |
| Conditional | Context-dependent | Trusted workspaces only |

## Common Anti-Patterns

### ❌ Using Bash for File Operations
**Wrong:** `bash("cat file.txt")`
**Right:** `Read(file_path="file.txt")`

### ❌ Sequential Reads Instead of Search
**Wrong:** Read file1, Read file2, Read file3...
**Right:** `Grep(pattern="search_term")`

### ❌ Write on Existing Files
**Wrong:** `Write(file_path="existing.txt")`
**Right:** `Edit(file_path="existing.txt", old="...", new="...")`

### ❌ Echo for Communication
**Wrong:** `bash("echo 'Processing...'")`
**Right:** Direct text output

### ❌ Placeholder Parameters
**Wrong:** `Edit(file_path="{FROM_PREVIOUS}")`
**Right:** Wait for previous call, use actual value

## Implementation Checklist

### Phase 1: Core Registry ✓
- [x] ToolRegistration with rich metadata
- [x] ToolCategory taxonomy
- [x] RiskProfile system
- [x] RuntimeProfile for performance

### Phase 2: Discovery & Routing
- [ ] Intent-based indexing
- [ ] Alternative suggestion system
- [ ] Anti-pattern detection
- [ ] Decision tree documentation

### Phase 3: Policy System
- [ ] Tiered policy system
- [ ] Conditional policies
- [ ] Full-auto mode with allowlists
- [ ] Policy inheritance chain

### Phase 4: Documentation
- [x] Tool catalog with examples
- [ ] Decision trees for common tasks
- [x] Anti-pattern documentation
- [ ] Interactive tool selector

### Phase 5: Optimization
- [ ] Result caching layer
- [ ] Parallel execution hints
- [ ] Performance monitoring
- [ ] Usage analytics

## Usage Scenarios

### Scenario 1: Read-Only Exploration
```
User: "Find all TODO comments in the codebase"

Registry Guidance:
✓ Use Grep(pattern="TODO", output_mode="files_with_matches")
✗ Don't use bash grep
✗ Don't read files sequentially

Policy: Allow (read-only, low risk)
```

### Scenario 2: File Modification
```
User: "Update configuration file"

Registry Guidance:
1. Check if file exists
2. If exists → Edit tool
3. If new → Write tool

Policy: Prompt (modifies files, medium risk)
```

### Scenario 3: Code Execution
```
User: "Run the tests"

Registry Guidance:
✓ Use Bash("npm test") or RunCommand
✗ Don't use for file operations

Policy: Prompt (executes code, high risk)
Security: Validate command against deny list
```

## Success Metrics

### For LLMs
- **Decision Accuracy:** % correct tool selections
- **First-Try Success:** % calls that work without retry
- **Anti-Pattern Rate:** % calls matching known anti-patterns
- **Parallel Utilization:** % independent calls executed in parallel

### For Users
- **Task Completion Rate:** % requests successfully completed
- **Approval Friction:** Average approvals needed per task
- **Error Recovery Time:** Time from error to successful retry
- **Trust Progression:** Movement through capability levels

### For System
- **Tool Coverage:** % use cases covered
- **Performance:** P50/P95/P99 latency by category
- **Reliability:** Tool execution success rate
- **Safety:** Policy violation rate

## Related Documentation

### Within This Directory
- [TOOL_REGISTRY_DESIGN.md](./TOOL_REGISTRY_DESIGN.md) - Complete architecture
- [TOOL_REGISTRY_EXAMPLES.md](./TOOL_REGISTRY_EXAMPLES.md) - Implementation examples
- [TOOL_BEST_PRACTICES.md](./TOOL_BEST_PRACTICES.md) - Best practices catalog
- [TOOL_POLICY_VS_SOP.md](./TOOL_POLICY_VS_SOP.md) - Policy vs. procedure

### Implementation References
- `/vtcode-core/src/tools/registry/` - Core registry implementation
- `/vtcode-core/src/tool_policy.rs` - Policy system
- `/vtcode-tool-traits/` - Tool trait definitions
- `/docs/tools/TOOL_SPECS.md` - Individual tool specifications
- `/docs/vtcode_tools_policy.md` - Policy configuration guide

### Development Guides
- `/docs/development/tool-development.md` - Creating custom tools
- `/docs/development/JUSTIFICATION_SYSTEM.md` - Approval system
- `/docs/development/async-architecture.md` - Async patterns

## Contributing

When adding new tools or modifying the registry:

1. **Follow the design principles** in TOOL_REGISTRY_DESIGN.md
2. **Provide rich metadata** as shown in TOOL_REGISTRY_EXAMPLES.md
3. **Check anti-patterns** from TOOL_BEST_PRACTICES.md
4. **Configure policies** appropriately (see TOOL_POLICY_VS_SOP.md)
5. **Test with LLMs** to ensure guidance is clear
6. **Update documentation** to reflect changes

## Questions?

For specific topics:
- **"How do I design a new tool?"** → TOOL_REGISTRY_DESIGN.md + TOOL_REGISTRY_EXAMPLES.md
- **"What are common mistakes?"** → TOOL_BEST_PRACTICES.md
- **"Policy vs. procedure?"** → TOOL_POLICY_VS_SOP.md
- **"How do I configure policies?"** → /docs/vtcode_tools_policy.md
- **"How do I implement a tool?"** → /docs/development/tool-development.md

---

**Summary:** This documentation provides a complete framework for building tool registries that enable agentic LLMs to effectively and safely help users. The design balances semantic clarity, safety, performance, and discoverability to create the best possible experience.
