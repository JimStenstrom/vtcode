# Tool Registry Design for Agentic LLMs

## Executive Summary

This document outlines a comprehensive tool registry architecture that optimizes the agentic LLM experience. The design balances **semantic clarity**, **safety**, **performance**, and **discoverability** to enable LLMs to effectively help users while maintaining security and reliability.

## Core Principles

### 1. Semantic Over Generic
**Tools should express clear intent, not generic capabilities.**

- ✅ Good: `Read`, `Edit`, `Grep`
- ❌ Bad: `FileOperation(mode="read")`, `Execute(type="search")`

**Why:** LLMs make better decisions when tools map to clear semantic actions. Generic tools force the LLM to manage mode selection, increasing cognitive load and error rates.

### 2. Appropriate Granularity
**Neither too fine-grained nor too coarse.**

- Too fine: `ReadLine`, `ReadParagraph`, `ReadFunction` (100+ micro-tools)
- Too coarse: `DoAnything(operation, params)` (1 mega-tool)
- Just right: `Read(file_path, offset?, limit?)` with sensible defaults

**Why:** Fine-grained tools overwhelm decision-making; coarse tools hide intent and complicate parameter construction.

### 3. Fail-Safe Defaults
**Design for safety when parameters are omitted.**

- Read entire file by default (not first line)
- Deny by default for risky operations
- Non-destructive modes as defaults

**Why:** LLMs may skip optional parameters; defaults should prefer completeness and safety.

### 4. Rich Metadata for Decision-Making
**Provide context beyond just "what the tool does."**

Every tool should include:
- **Purpose**: What problem it solves
- **When to use**: Decision criteria
- **When NOT to use**: Anti-patterns and alternatives
- **Cost indicators**: Latency, resource usage
- **Risk level**: Safety classification
- **Examples**: Positive and negative usage patterns

## Tool Registry Schema

### Tool Registration Structure

```rust
pub struct ToolRegistration {
    // Core Identity
    name: &'static str,
    category: ToolCategory,
    capability_level: CapabilityLevel,

    // Execution Handler
    handler: ToolHandler,

    // Metadata for LLM Decision-Making
    metadata: ToolMetadata,

    // Safety & Policy
    risk_profile: RiskProfile,
    default_policy: ToolPolicy,

    // Runtime Characteristics
    runtime_profile: RuntimeProfile,

    // Lifecycle
    lifecycle: ToolLifecycle,
}
```

### 1. Core Identity

```rust
pub enum ToolCategory {
    // File & Code Operations
    FileRead,           // read_file, grep, list_files
    FileWrite,          // write_file, edit_file, apply_patch
    FileSystem,         // create directory, delete, move

    // Execution
    ShellExecution,     // run_terminal_cmd, bash
    CodeExecution,      // Run tests, linters, formatters

    // Code Understanding
    CodeAnalysis,       // tree_sitter, ast parsing
    CodeSearch,         // semantic search, symbol lookup

    // External Integration
    NetworkAccess,      // web_fetch, API calls
    DatabaseAccess,     // SQL queries, data operations

    // Agent Coordination
    TaskManagement,     // todo_write, plan
    SubAgent,           // task delegation

    // User Interaction
    UserPrompt,         // ask_user_question
    Notification,       // Status updates, progress

    // Source Control
    GitOperation,       // Git commands

    // MCP/External
    McpTool,           // Tools from MCP servers
    CustomExtension,   // User-defined tools
}

pub enum CapabilityLevel {
    FileOperations,
    CodeSearch,
    CodeModification,
    ShellAccess,
    NetworkAccess,
    SystemAdmin,
}
```

### 2. Tool Metadata

```rust
pub struct ToolMetadata {
    // Human-readable information
    description: &'static str,
    detailed_description: Option<String>,

    // Decision Support
    usage_notes: Vec<UsageNote>,
    when_to_use: Vec<String>,
    when_not_to_use: Vec<String>,
    alternatives: Vec<ToolAlternative>,

    // Examples
    examples: Vec<ToolExample>,

    // Parameter Schema
    parameters_schema: serde_json::Value,
    required_parameters: Vec<&'static str>,
    optional_parameters: Vec<ParameterInfo>,

    // Output Schema
    output_schema: Option<serde_json::Value>,
    error_patterns: Vec<ErrorPattern>,
}

pub struct UsageNote {
    note_type: NoteType,  // Tip, Warning, Important, Deprecated
    content: String,
}

pub enum NoteType {
    Tip,           // "Use offset/limit for large files"
    Warning,       // "Do not use for binary files"
    Important,     // "Always verify changes before committing"
    Deprecated,    // "Use Edit instead of sed"
}

pub struct ToolAlternative {
    tool_name: String,
    when_better: String,
    example: String,
}

pub struct ToolExample {
    description: String,
    input: serde_json::Value,
    expected_output: Option<String>,
    is_anti_pattern: bool,  // Negative examples
}

pub struct ParameterInfo {
    name: String,
    description: String,
    default_value: Option<serde_json::Value>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ErrorPattern {
    pattern: String,
    meaning: String,
    suggested_action: String,
}
```

### 3. Risk Profile

```rust
pub struct RiskProfile {
    risk_level: RiskLevel,
    risk_factors: Vec<RiskFactor>,
    requires_confirmation: bool,
    audit_level: AuditLevel,
}

pub enum RiskLevel {
    Low,        // Read-only, no side effects
    Medium,     // Create/modify within workspace
    High,       // Execute code, network access
    Critical,   // System changes, destructive operations
}

pub enum RiskFactor {
    ModifiesFiles,
    ExecutesCode,
    NetworkAccess,
    DestructivePotential,
    PrivilegeEscalation,
    DataExfiltration,
}

pub enum AuditLevel {
    None,           // No logging needed
    Basic,          // Log invocation
    Detailed,       // Log parameters
    Full,           // Log parameters + output
}
```

### 4. Runtime Profile

```rust
pub struct RuntimeProfile {
    // Performance Characteristics
    expected_latency: LatencyCategory,
    resource_usage: ResourceUsage,

    // Execution Context
    requires_pty: bool,
    requires_workspace: bool,
    can_run_parallel: bool,

    // Timeout Configuration
    timeout_category: ToolTimeoutCategory,
    default_timeout: Option<Duration>,

    // Caching
    cacheable: bool,
    cache_key_fn: Option<CacheKeyFn>,
    cache_ttl: Option<Duration>,
}

pub enum LatencyCategory {
    Instant,      // < 100ms (cached operations)
    Fast,         // < 1s (file reads, simple operations)
    Medium,       // 1-5s (searches, analysis)
    Slow,         // 5-30s (compilations, network calls)
    VerySlow,     // > 30s (long-running commands)
}

pub struct ResourceUsage {
    cpu_intensity: CpuIntensity,
    memory_requirement: MemoryRequirement,
    io_intensity: IoIntensity,
}

pub enum CpuIntensity { Low, Medium, High }
pub enum MemoryRequirement { Minimal, Moderate, High }
pub enum IoIntensity { Minimal, Moderate, Heavy }
```

### 5. Tool Lifecycle

```rust
pub struct ToolLifecycle {
    status: ToolStatus,
    deprecation: Option<DeprecationInfo>,
    version: ToolVersion,
    stability: StabilityLevel,
}

pub enum ToolStatus {
    Active,
    Deprecated,
    Experimental,
    Disabled,
}

pub struct DeprecationInfo {
    deprecated_since: String,
    removal_planned: Option<String>,
    replacement: Option<String>,
    migration_guide: Option<String>,
    warning_message: String,
}

pub struct ToolVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

pub enum StabilityLevel {
    Stable,       // Production-ready
    Beta,         // Mostly stable, may change
    Alpha,        // Experimental, expect changes
    Internal,     // Not for general use
}
```

## Tool Categorization & Discovery

### Capability-Based Organization

Tools are organized by capability levels, creating a hierarchy of access:

```rust
pub struct CapabilityMatrix {
    file_operations: Vec<Tool>,     // Read, List, basic file ops
    code_search: Vec<Tool>,          // Grep, Tree-sitter, search
    code_modification: Vec<Tool>,    // Edit, Write, Apply Patch
    shell_access: Vec<Tool>,         // run_terminal_cmd, Bash
    network_access: Vec<Tool>,       // web_fetch, API calls
    system_admin: Vec<Tool>,         // System-level operations
}
```

**Progressive Access Model:**
- Start with read-only tools (file_operations, code_search)
- Unlock modification tools as trust builds
- Require explicit approval for high-risk categories

### Intent-Based Discovery

LLMs should be guided to the right tool through clear intent mapping:

```rust
pub struct IntentIndex {
    // "I need to find code" -> Grep, Tree-sitter, CodeSearch
    search_intent: Vec<Tool>,

    // "I need to modify files" -> Edit, Write, ApplyPatch
    modification_intent: Vec<Tool>,

    // "I need to execute something" -> Bash, RunCommand
    execution_intent: Vec<Tool>,

    // "I need to understand structure" -> Tree-sitter, ListFiles
    analysis_intent: Vec<Tool>,
}
```

### Smart Tool Suggestions

The registry should provide **contextual alternatives**:

```rust
impl ToolRegistry {
    /// Given a tool request, suggest better alternatives based on context
    pub fn suggest_alternatives(
        &self,
        requested_tool: &str,
        context: &ExecutionContext,
    ) -> Vec<ToolSuggestion> {
        // Examples:
        // - User requests "Bash cat file.txt" -> suggest Read instead
        // - User requests "Bash grep pattern" -> suggest Grep instead
        // - User requests "Read" for 50 files -> suggest Task agent
        // - User requests "Write" on existing file -> suggest Edit instead
    }
}

pub struct ToolSuggestion {
    tool_name: String,
    reason: String,
    confidence: f32,
    example_usage: String,
}
```

## Tool Selection & Routing Guidelines

### Decision Framework for LLMs

Provide explicit decision trees in tool documentation:

```
DECISION: Need to read a file?
├─ File path known and specific?
│  ├─ YES -> Use Read tool
│  └─ NO -> Use Glob or Grep to find it first
├─ Need to search content across files?
│  └─ Use Grep with appropriate output_mode
├─ Need to understand code structure?
│  └─ Use Tree-sitter for AST analysis
└─ Exploring unknown codebase?
   └─ Use Task agent with subagent_type=Explore
```

### Anti-Pattern Detection

Registry should warn against common mistakes:

```rust
pub struct AntiPattern {
    pattern: String,
    why_bad: String,
    correct_approach: String,
    example: String,
}

// Examples:
// - Using Bash for file operations -> Use dedicated tools
// - Using Read in loop for search -> Use Grep
// - Using echo for communication -> Output text directly
// - Chaining too many commands -> Use parallel tool calls
```

### Performance Optimization Rules

```
1. PARALLEL EXECUTION:
   - If tools are independent -> Call in single message
   - If tools depend on each other -> Call sequentially
   - Never use placeholders or guess parameters

2. TOOL SELECTION:
   - Prefer specialized tools over Bash
   - Use Task agent for complex multi-step operations
   - Use appropriate output_mode to minimize tokens

3. CONTEXT MANAGEMENT:
   - Large files: Use offset/limit parameters
   - Many files: Use file pattern matching
   - Deep exploration: Delegate to Task agents
```

## Policy System

### Policy Levels

```rust
pub enum ToolPolicy {
    Allow,      // Auto-approve
    Prompt,     // Ask user
    Deny,       // Block execution
    Conditional(PolicyCondition),  // Context-dependent
}

pub struct PolicyCondition {
    condition_type: ConditionType,
    allow_if: Box<dyn Fn(&ExecutionContext) -> bool>,
}

pub enum ConditionType {
    WorkspaceTrusted,
    ParametersMatch(Vec<String>),
    RiskLevelBelow(RiskLevel),
    RecentlyApproved,
    BatchOperation,
}
```

### Policy Inheritance

```
1. Global Default Policy
   ↓
2. Category Policy (all file operations)
   ↓
3. Tool-Specific Policy (write_file)
   ↓
4. Context-Specific Override (in trusted workspace)
```

### Full-Auto Mode

Special mode for trusted environments:

```rust
pub struct FullAutoMode {
    enabled: bool,
    allowlist: Vec<String>,
    deny_overrides: Vec<String>,  // Always deny even in full-auto
}

// Example configuration:
// Allow: Read, Edit, Grep, Bash (safe commands)
// Always Deny: rm -rf, dd, format, system modifications
```

## Best Practices

### For Tool Designers

1. **Single Responsibility**: Each tool should do one thing well
2. **Composability**: Tools should work well together
3. **Idempotency**: When possible, repeated calls should be safe
4. **Clear Failures**: Errors should be actionable and specific
5. **Progressive Disclosure**: Simple cases should be simple; complex cases should be possible

### For Tool Documentation

1. **Lead with Intent**: "Use this when..." before "This does..."
2. **Show, Don't Tell**: Include examples for common patterns
3. **Warn Explicitly**: Call out anti-patterns and common mistakes
4. **Provide Alternatives**: Guide to better tools when applicable
5. **Update Examples**: Keep examples current with actual usage patterns

### For LLM Prompting

Tools should include **inline guidance** in descriptions:

```
"Read file contents. USAGE: For specific known files only.
For searching use Grep. For exploration use Task agent.
IMPORTANT: Prefer this over 'cat' command. Can read up to
2000 lines by default; use offset/limit for large files."
```

**Structure:**
1. What (1 sentence)
2. Usage criteria (when to use)
3. Important notes (constraints, warnings)
4. Parameters (with defaults and validation)

## Implementation Checklist

### Phase 1: Core Registry
- [ ] Implement ToolRegistration with rich metadata
- [ ] Create ToolCategory taxonomy
- [ ] Build RiskProfile system
- [ ] Add RuntimeProfile for performance metadata

### Phase 2: Discovery & Routing
- [ ] Implement intent-based indexing
- [ ] Create alternative suggestion system
- [ ] Build anti-pattern detection
- [ ] Add decision tree documentation

### Phase 3: Policy System
- [ ] Implement tiered policy system
- [ ] Add conditional policies
- [ ] Create full-auto mode with allowlists
- [ ] Build policy inheritance chain

### Phase 4: Documentation
- [ ] Generate tool catalog with examples
- [ ] Create decision trees for common tasks
- [ ] Document anti-patterns
- [ ] Build interactive tool selector

### Phase 5: Optimization
- [ ] Add result caching layer
- [ ] Implement parallel execution hints
- [ ] Create performance monitoring
- [ ] Build usage analytics for improvement

## Metrics for Success

### For LLMs
- **Decision Accuracy**: % of correct tool selections
- **First-Try Success**: % of tool calls that work without retry
- **Anti-Pattern Rate**: % of calls that match known anti-patterns
- **Parallel Utilization**: % of independent calls executed in parallel

### For Users
- **Task Completion Rate**: % of user requests successfully completed
- **Approval Friction**: Average approvals needed per task
- **Error Recovery Time**: Time from error to successful retry
- **User Trust Level**: Progression through capability levels

### For System
- **Tool Coverage**: % of use cases covered by available tools
- **Performance**: P50/P95/P99 latency per tool category
- **Reliability**: Tool execution success rate
- **Safety**: Policy violation rate

## Conclusion

A well-designed tool registry is foundational to effective agentic LLM systems. The registry should:

1. **Guide decision-making** through rich metadata and clear documentation
2. **Ensure safety** through risk classification and policy enforcement
3. **Optimize performance** through caching, parallel execution, and profiling
4. **Enable discovery** through categorization and intent mapping
5. **Support evolution** through versioning and deprecation management

The vtcode implementation demonstrates these principles in practice, balancing the needs of LLM agents, end users, and system administrators.

---

**References:**
- `/vtcode-core/src/tools/registry/` - Core implementation
- `/vtcode-core/src/tool_policy.rs` - Policy system
- `/docs/tools/TOOL_SPECS.md` - Tool specifications
- `/docs/vtcode_tools_policy.md` - Policy documentation
