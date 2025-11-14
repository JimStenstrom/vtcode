# Agent 3: Tools Enhancement - Task Brief

**Your Role:** Enhance vtcode-tools with plugin architecture
**Estimated Time:** 5-6 hours (with 1-2h wait for Agents 1 & 2)
**Branch Name:** `phase-4-tools`
**Working in Parallel With:** Agent 1 (tree-sitter), Agent 2 (patch) - then solo

---

## Your Mission

You are **Agent 3** in a 3-agent parallel execution of Phase 4. Your job is to enhance the vtcode-tools crate with a **plugin architecture**, **tool discovery**, and a clean **execution engine**.

This enhancement will:
- Refactor 10K LOC of tool infrastructure
- Enable custom tool plugins
- Create a clean tool registry and execution engine
- Make tools more modular and testable

---

## Context: The Big Picture

VTCode is transforming from a 95K LOC monolith to a modular architecture. You're in **Phase 4: Modularize Tools**.

**Current State:**
- Phase 1-2: ✅ Complete
- Phase 3: 🔄 In progress
- Phase 4: → Your task (tools enhancement)

**Your Part:**
- Phase 1 (1-2h): Analysis and design - **CAN START IMMEDIATELY**
- **WAIT** for Agents 1 & 2 to complete
- Phase 2 (3-4h): Implementation using vtcode-tree-sitter and vtcode-patch

---

## Required Reading

**IMPORTANT:** Read these before starting:

1. **docs/development/PHASE_4_AND_5_IMPLEMENTATION.md**
   - Section 4.3: Enhance vtcode-tools (your detailed task)
   - Complete implementation steps

2. **docs/development/ARCHITECTURE_TRANSFORMATION.md**
   - Phase 4 overview

3. **docs/development/PHASE_4_PARALLELIZATION_STRATEGY.md**
   - Your role: Analysis first, then wait, then implement
   - Strategy 2 (3 agents)

---

## Two-Phase Execution

### Phase 1: Analysis & Design (1-2 hours) ✅ START NOW

You can begin this immediately in parallel with Agents 1 & 2:

1. Analyze current tool architecture
2. Design plugin interface
3. Plan tool registry structure
4. Document tool execution flow
5. Create detailed implementation plan

### Phase 2: Implementation (3-4 hours) ⏳ WAIT FOR 1 & 2

You must wait for:
- ✅ Agent 1: vtcode-tree-sitter merged
- ✅ Agent 2: vtcode-patch merged

Then you can:
1. Implement plugin architecture
2. Extract tool registry
3. Build execution engine
4. Integrate with vtcode-tree-sitter and vtcode-patch
5. Test everything

---

## Phase 1: Analysis & Design (START NOW - 1-2 hours)

### Task 4.3.1: Analyze Current Architecture (30 minutes)

**Goal:** Understand the existing tool system.

**Explore the codebase:**

```bash
# Find all tools
find vtcode-core/src/tools -name "*.rs" -type f

# Count lines
find vtcode-core/src/tools -name "*.rs" | xargs wc -l

# Look for common patterns
rg "pub struct.*Tool" vtcode-core/src/tools
rg "impl.*Tool" vtcode-core/src/tools

# Find tool registry
rg "register.*tool" vtcode-core/src -i
rg "ToolRegistry" vtcode-core/src
```

**Document your findings:**

Create `docs/phase4_tool_analysis.md`:

```markdown
# Current Tool Architecture Analysis

## Tool Inventory

Found tools:
- Read (vtcode-core/src/tools/read.rs) - ~XXX lines
- Write (vtcode-core/src/tools/write.rs) - ~XXX lines
- Edit (vtcode-core/src/tools/edit.rs) - ~XXX lines
- MultiEdit (vtcode-core/src/tools/multi_edit.rs) - ~XXX lines
- Bash (vtcode-core/src/tools/bash.rs) - ~XXX lines
- Glob (vtcode-core/src/tools/glob.rs) - ~XXX lines
- Grep (vtcode-core/src/tools/grep.rs) - ~XXX lines
- WebFetch (vtcode-core/src/tools/web_fetch.rs) - ~XXX lines
- [Add others as you find them]

## Common Patterns

All tools seem to have:
1. Struct definition (e.g., `pub struct ReadTool`)
2. Constructor (e.g., `pub fn new()`)
3. Execute method (e.g., `pub async fn execute()`)
4. Request/Response types

## Current Registry

Location: [Find where tools are registered]

Current approach:
- [Describe how tools are currently registered]
- [Describe how tools are currently invoked]
- [Describe any configuration]

## Dependencies

Tools depend on:
- [List dependencies]
- vtcode-bash-runner (for Bash tool)
- File system operations
- HTTP client (for WebFetch)
- etc.

## Issues to Address

1. **Tight coupling**: Tools are tightly integrated into core
2. **No plugin system**: Can't add custom tools easily
3. **Inconsistent interfaces**: Tools have different patterns
4. **Hard to test**: Tools need full core context
5. **No discovery**: Tools are hardcoded

## Opportunities

1. Create common trait for all tools
2. Build plugin system for custom tools
3. Add tool discovery mechanism
4. Improve error handling
5. Add tool validation
6. Add tool permissions/capabilities
```

**Checkpoint:** Analysis document created with findings.

---

### Task 4.3.2: Design Plugin Interface (45 minutes)

**Goal:** Design the plugin architecture.

**Create `docs/phase4_plugin_design.md`:**

```markdown
# Tool Plugin Architecture Design

## Core Trait

```rust
pub trait ToolPlugin: Send + Sync {
    /// Unique tool name
    fn name(&self) -> &str;

    /// Tool description for users
    fn description(&self) -> &str;

    /// Execute the tool with given request
    fn execute(&self, request: ToolRequest)
        -> BoxFuture<'_, Result<ToolResponse>>;

    /// Validate request before execution
    fn validate(&self, request: &ToolRequest) -> Result<()> {
        Ok(()) // Default: no validation
    }

    /// Tool capabilities and permissions
    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities::default()
    }

    /// Tool version
    fn version(&self) -> &str {
        "0.1.0"
    }
}
```

## Tool Registry

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolPlugin>>,
    config: RegistryConfig,
}

impl ToolRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, tool: Arc<dyn ToolPlugin>);
    pub fn get(&self, name: &str) -> Option<&Arc<dyn ToolPlugin>>;
    pub fn list_tools(&self) -> Vec<ToolInfo>;
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn ToolPlugin>>;
}
```

## Execution Engine

```rust
pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
    config: ExecutorConfig,
}

impl ToolExecutor {
    pub async fn execute(
        &self,
        tool_name: &str,
        request: ToolRequest,
    ) -> Result<ToolResponse>;

    fn check_permissions(&self, tool: &dyn ToolPlugin, request: &ToolRequest)
        -> Result<()>;

    fn validate_request(&self, tool: &dyn ToolPlugin, request: &ToolRequest)
        -> Result<()>;
}
```

## Built-in Tool Migration

Each existing tool will implement `ToolPlugin`:

```rust
pub struct ReadTool {
    // existing fields
}

impl ToolPlugin for ReadTool {
    fn name(&self) -> &str {
        "Read"
    }

    fn description(&self) -> &str {
        "Read file contents"
    }

    fn execute(&self, request: ToolRequest)
        -> BoxFuture<'_, Result<ToolResponse>> {
        Box::pin(async move {
            // existing execute logic
        })
    }

    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities {
            supports_streaming: false,
            requires_approval: false,
            can_modify_filesystem: false,
            can_execute_code: false,
        }
    }
}
```

## Custom Tool Example

```rust
// In user's code or separate crate
pub struct MyCustomTool;

impl ToolPlugin for MyCustomTool {
    fn name(&self) -> &str {
        "CustomAnalyzer"
    }

    fn description(&self) -> &str {
        "Custom code analysis"
    }

    fn execute(&self, request: ToolRequest)
        -> BoxFuture<'_, Result<ToolResponse>> {
        Box::pin(async move {
            // Custom logic
            Ok(ToolResponse::success("Analysis complete"))
        })
    }
}

// Register it
let mut registry = ToolRegistry::new();
registry.register(Arc::new(MyCustomTool));
```

## Tool Discovery

```rust
pub struct ToolDiscovery {
    search_paths: Vec<PathBuf>,
}

impl ToolDiscovery {
    pub async fn discover(&self) -> Result<Vec<ToolMetadata>>;
    pub async fn load_tool(&self, metadata: &ToolMetadata)
        -> Result<Box<dyn ToolPlugin>>;
}
```

Tool metadata file format (TOML):

```toml
[tool]
name = "my-tool"
description = "My custom tool"
version = "1.0.0"
executable = "./my-tool"  # or path to .so/.dylib

[capabilities]
requires_approval = true
can_modify_filesystem = true
can_execute_code = false
```

## Benefits

1. **Extensibility**: Easy to add new tools
2. **Testability**: Tools can be tested independently
3. **Modularity**: Clean separation of concerns
4. **Reusability**: Tools can be used in other projects
5. **Safety**: Capabilities system for permissions
6. **Discovery**: Auto-discover custom tools
```

**Checkpoint:** Plugin design documented and reviewed.

---

### Task 4.3.3: Plan Implementation (30 minutes)

**Goal:** Create detailed implementation checklist.

**Create `docs/phase4_implementation_checklist.md`:**

```markdown
# Tools Enhancement Implementation Checklist

## Pre-Implementation (DONE)
- [x] Analyze current architecture
- [x] Design plugin interface
- [x] Document design decisions
- [x] Wait for Agents 1 & 2 to complete

## Implementation Phase

### Step 1: Create Plugin Trait (30 min)
- [ ] Create `vtcode-tools/src/plugin.rs`
- [ ] Define `ToolPlugin` trait
- [ ] Define `ToolCapabilities`
- [ ] Define `ToolRequest` and `ToolResponse` types
- [ ] Add comprehensive documentation

### Step 2: Extract Tool Registry (1.5 hours)
- [ ] Create `vtcode-tools/src/registry.rs`
- [ ] Implement `ToolRegistry` struct
- [ ] Implement registration methods
- [ ] Implement lookup methods
- [ ] Add thread-safe access
- [ ] Write unit tests

### Step 3: Build Execution Engine (1.5 hours)
- [ ] Create `vtcode-tools/src/executor.rs`
- [ ] Implement `ToolExecutor` struct
- [ ] Add permission checking
- [ ] Add request validation
- [ ] Add timeout handling
- [ ] Add error handling
- [ ] Write unit tests

### Step 4: Implement Tool Discovery (1 hour)
- [ ] Create `vtcode-tools/src/discovery.rs`
- [ ] Implement directory scanning
- [ ] Implement metadata parsing
- [ ] Implement tool loading
- [ ] Add caching
- [ ] Write integration tests

### Step 5: Migrate Built-in Tools (2 hours)
- [ ] Update `Read` tool to implement `ToolPlugin`
- [ ] Update `Write` tool
- [ ] Update `Edit` tool (uses vtcode-patch)
- [ ] Update `Bash` tool
- [ ] Update `Glob` tool
- [ ] Update `Grep` tool
- [ ] Update `WebFetch` tool
- [ ] Update all other tools
- [ ] Ensure all tests still pass

### Step 6: Integration with vtcode-core (1 hour)
- [ ] Update `vtcode-core/src/agent/tools.rs`
- [ ] Use new `ToolRegistry` and `ToolExecutor`
- [ ] Remove old tool registration code
- [ ] Update tests
- [ ] Verify no regressions

### Step 7: Testing & Validation (1 hour)
- [ ] Run all tool tests
- [ ] Run integration tests
- [ ] Test custom tool registration
- [ ] Test tool discovery
- [ ] Performance testing
- [ ] Documentation review

### Step 8: Documentation (30 min)
- [ ] Update vtcode-tools README
- [ ] Add plugin development guide
- [ ] Add custom tool examples
- [ ] Update architecture docs

## Success Criteria
- [ ] All existing tools work as before
- [ ] Custom tools can be registered
- [ ] Tool discovery works
- [ ] All tests pass
- [ ] No performance regressions
- [ ] Documentation complete
```

**Checkpoint:** Implementation plan ready, waiting for Agents 1 & 2.

---

## ⏸️ WAITING POINT

**Stop here until:**
1. ✅ Agent 1 completes and merges vtcode-tree-sitter
2. ✅ Agent 2 completes and merges vtcode-patch

**During the wait:**
- Review your design with Agents 1 & 2
- Refine your implementation plan
- Prepare any needed documentation
- Rest and prepare for implementation phase! ☕

**When both agents complete:**
1. Pull latest main: `git pull origin main`
2. Create your branch: `git checkout -b phase-4-tools`
3. Begin Phase 2 implementation

---

## Phase 2: Implementation (3-4 hours)

### Task 4.3.4: Create Plugin Trait (30 minutes)

**Goal:** Define the core plugin interface.

**Create vtcode-tools/src/plugin.rs:**

```rust
use std::future::Future;
use std::pin::Pin;
use anyhow::Result;

/// Future type for async tool execution
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Plugin interface for tools
pub trait ToolPlugin: Send + Sync {
    /// Unique tool name (e.g., "Read", "Write", "CustomAnalyzer")
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Execute the tool with given request
    fn execute(&self, request: ToolRequest) -> BoxFuture<'_, Result<ToolResponse>>;

    /// Validate request before execution (optional)
    fn validate(&self, request: &ToolRequest) -> Result<()> {
        let _ = request;
        Ok(())
    }

    /// Tool capabilities and permissions
    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities::default()
    }

    /// Tool version
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Tool author (optional)
    fn author(&self) -> Option<&str> {
        None
    }
}

/// Tool capabilities and permissions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCapabilities {
    /// Tool supports streaming responses
    pub supports_streaming: bool,

    /// Tool requires user approval before execution
    pub requires_approval: bool,

    /// Tool can modify the filesystem
    pub can_modify_filesystem: bool,

    /// Tool can execute arbitrary code
    pub can_execute_code: bool,

    /// Tool makes network requests
    pub makes_network_requests: bool,
}

impl Default for ToolCapabilities {
    fn default() -> Self {
        Self {
            supports_streaming: false,
            requires_approval: false,
            can_modify_filesystem: false,
            can_execute_code: false,
            makes_network_requests: false,
        }
    }
}

/// Tool request containing parameters
#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub tool: String,
    pub parameters: serde_json::Value,
}

/// Tool response with results
#[derive(Debug, Clone)]
pub struct ToolResponse {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ToolResponse {
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            success: true,
            content: Some(content.into()),
            error: None,
            metadata: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            content: None,
            error: Some(error.into()),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Tool information for listings
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub capabilities: ToolCapabilities,
}
```

**Update vtcode-tools/src/lib.rs:**

```rust
pub mod plugin;
pub mod registry;
pub mod executor;
pub mod discovery;

pub use plugin::*;
pub use registry::ToolRegistry;
pub use executor::ToolExecutor;
pub use discovery::ToolDiscovery;
```

**Checkpoint:** Plugin trait compiles.

---

### Task 4.3.5: Implement Tool Registry (1.5 hours)

**Goal:** Create the tool registry.

**Create vtcode-tools/src/registry.rs:**

```rust
use crate::plugin::{ToolPlugin, ToolInfo};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for tool registry
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Allow overwriting existing tools
    pub allow_overwrite: bool,

    /// Maximum number of tools
    pub max_tools: Option<usize>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            allow_overwrite: false,
            max_tools: Some(100),
        }
    }
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolPlugin>>,
    config: RegistryConfig,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            config: RegistryConfig::default(),
        };

        // Register built-in tools
        registry.register_builtin_tools();

        registry
    }

    /// Create with custom configuration
    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            tools: HashMap::new(),
            config,
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn ToolPlugin>) -> Result<(), String> {
        let name = tool.name().to_string();

        // Check if already exists
        if self.tools.contains_key(&name) && !self.config.allow_overwrite {
            return Err(format!("Tool '{}' already registered", name));
        }

        // Check max tools
        if let Some(max) = self.config.max_tools {
            if self.tools.len() >= max && !self.tools.contains_key(&name) {
                return Err(format!("Maximum number of tools ({}) reached", max));
            }
        }

        self.tools.insert(name, tool);
        Ok(())
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn ToolPlugin>> {
        self.tools.get(name)
    }

    /// Remove a tool
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn ToolPlugin>> {
        self.tools.remove(name)
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                version: tool.version().to_string(),
                author: tool.author().map(|s| s.to_string()),
                capabilities: tool.capabilities(),
            })
            .collect()
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get number of registered tools
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Register built-in tools
    fn register_builtin_tools(&mut self) {
        // TODO: Register built-in tools here
        // self.register(Arc::new(ReadTool::new())).ok();
        // self.register(Arc::new(WriteTool::new())).ok();
        // etc.
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{ToolPlugin, ToolRequest, ToolResponse, BoxFuture};

    struct MockTool;

    impl ToolPlugin for MockTool {
        fn name(&self) -> &str {
            "MockTool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn execute(&self, _request: ToolRequest) -> BoxFuture<'_, Result<ToolResponse, anyhow::Error>> {
            Box::pin(async move {
                Ok(ToolResponse::success("mock result"))
            })
        }
    }

    #[test]
    fn test_register_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool);

        assert!(registry.register(tool).is_ok());
        assert!(registry.contains("MockTool"));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = ToolRegistry::new();
        let tool1 = Arc::new(MockTool);
        let tool2 = Arc::new(MockTool);

        assert!(registry.register(tool1).is_ok());
        assert!(registry.register(tool2).is_err());
    }

    #[test]
    fn test_get_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool);
        registry.register(tool).unwrap();

        let retrieved = registry.get("MockTool");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool);
        registry.register(tool).unwrap();

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "MockTool");
    }
}
```

**Checkpoint:** Registry compiles and tests pass.

---

### [Continue with remaining tasks...]

Due to length limits, the complete implementation steps for:
- Task 4.3.6: Implement Execution Engine (1.5 hours)
- Task 4.3.7: Tool Discovery (1 hour)
- Task 4.3.8: Migrate Built-in Tools (2 hours)
- Task 4.3.9: Integration with vtcode-core (1 hour)
- Task 4.3.10: Testing & Documentation (1.5 hours)

Follow the same pattern as shown in `PHASE_4_AND_5_IMPLEMENTATION.md` Section 4.3.

---

## Coordination with Agents 1 & 2

### During Your Wait Time

**With Agent 1:**
- Review their CodeAnalyzer trait design
- Plan how to integrate tree-sitter analysis into tools
- Discuss any API questions

**With Agent 2:**
- Review their patch API
- Confirm Edit tool will work with new vtcode-patch
- Discuss error handling patterns

### After They Complete

**Pull their changes:**
```bash
git checkout main
git pull origin main
git checkout -b phase-4-tools
```

**Use their crates:**
```toml
# In vtcode-tools/Cargo.toml or vtcode-core/Cargo.toml
[dependencies]
vtcode-tree-sitter = { path = "../vtcode-tree-sitter", optional = true }
vtcode-patch = { path = "../vtcode-patch" }
```

---

## Success Criteria

- [ ] ✅ Plugin trait defined and documented
- [ ] ✅ Tool registry implemented with tests
- [ ] ✅ Execution engine with permissions
- [ ] ✅ Tool discovery functional
- [ ] ✅ All built-in tools migrated to plugin interface
- [ ] ✅ Custom tools can be registered
- [ ] ✅ Integration with vtcode-core complete
- [ ] ✅ All tests pass (existing + new)
- [ ] ✅ No regressions
- [ ] ✅ Documentation complete
- [ ] ✅ Plugin development guide written

---

## Timeline

### Phase 1 (Can Start Now)
| Hour | Activity |
|------|----------|
| 0-0.5 | Analyze architecture |
| 0.5-1.25 | Design plugin interface |
| 1.25-1.5 | Create implementation plan |

### Wait Period
| Time | Activity |
|------|----------|
| Wait | Review designs with Agents 1 & 2 |
| Wait | Refine plans |
| Wait | Prepare for implementation |

### Phase 2 (After Agents 1 & 2 Complete)
| Hour | Activity |
|------|----------|
| 0-0.5 | Create plugin trait |
| 0.5-2 | Implement registry |
| 2-3.5 | Implement executor |
| 3.5-4.5 | Tool discovery |
| 4.5-6.5 | Migrate tools |
| 6.5-7.5 | Integration & testing |
| 7.5-8 | Documentation |

**Total: 5-6 hours (not including wait time)**

---

## Questions?

Ask in the coordination channel! Your work depends on Agents 1 & 2, so clear communication is key.

---

**Ready to enhance the tool system? Let's create a world-class plugin architecture! 🛠️**
