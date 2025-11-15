# Tool Registry Best Practices & Anti-Patterns

This document catalogs proven best practices and common anti-patterns for tool registry design and usage in agentic LLM systems.

## Table of Contents
1. [Tool Design Principles](#tool-design-principles)
2. [Common Anti-Patterns](#common-anti-patterns)
3. [LLM Guidance Patterns](#llm-guidance-patterns)
4. [Performance Optimization](#performance-optimization)
5. [Safety & Security](#safety--security)
6. [Error Handling](#error-handling)
7. [Evolution & Deprecation](#evolution--deprecation)

## Tool Design Principles

### ✅ DO: Design for Semantic Clarity

**Good:** Tools with clear, specific purposes
```
Read       - Read file contents
Edit       - Modify existing files with exact string replacement
Write      - Create new files
Grep       - Search content across files
```

**Bad:** Generic tools that require mode selection
```
FileOperation(mode="read"|"write"|"edit"|"search")
Execute(type="file"|"command"|"network")
```

**Why:** LLMs make better decisions with semantic tools. Generic tools shift complexity to parameter selection, increasing cognitive load and error rates.

### ✅ DO: Provide Rich, Action-Oriented Documentation

**Good:**
```
"Execute bash commands in PTY session.

WHEN TO USE:
- Running build tools (npm, cargo, make)
- Executing tests and linters
- Git operations

WHEN NOT TO USE:
- Reading files → Use Read tool
- Searching content → Use Grep tool
- Communicating with user → Output text directly

IMPORTANT: Commands are validated against security policies."
```

**Bad:**
```
"Runs commands in shell."
```

**Why:** LLMs need guidance on appropriate usage. Decision criteria prevent misuse and guide to better alternatives.

### ✅ DO: Fail-Safe by Default

**Good:**
```rust
// Write tool refuses to overwrite existing files
if file_exists(path) {
    return Err("File exists. Use Edit for modifications.");
}

// Read defaults to entire file, not partial
let limit = args.get("limit").unwrap_or(usize::MAX);

// Commands default to safe timeout
let timeout = args.get("timeout").unwrap_or(120_000); // 2 minutes
```

**Bad:**
```rust
// Dangerous: defaults to overwrite
let overwrite = args.get("overwrite").unwrap_or(true);

// Dangerous: defaults to first line only
let limit = args.get("limit").unwrap_or(1);

// Dangerous: no timeout default
let timeout = args.get("timeout")?; // Error if not provided
```

**Why:** LLMs may omit optional parameters. Defaults should favor safety and completeness.

### ✅ DO: Design for Composability

**Good:** Tools that work well together
```
# Workflow: Find files → Read contents → Modify
1. Glob(pattern="**/*_test.rs")      # Find test files
2. Read(file_path="path/from/glob")  # Read specific files
3. Edit(file_path="...", old="...", new="...")  # Modify
```

**Bad:** Tools that duplicate functionality or can't be chained
```
# Redundant functionality
SearchAndRead(pattern="*.rs", also_read=true)
FindAndModify(search="...", replace="...")
```

**Why:** Composable tools are more flexible and easier to understand. Each tool does one thing well.

### ✅ DO: Provide Appropriate Granularity

**Too Fine-Grained:**
```
ReadFirstLine(file_path)
ReadLastLine(file_path)
ReadLineRange(file_path, start, end)
ReadFunction(file_path, function_name)
ReadClass(file_path, class_name)
```

**Too Coarse:**
```
DoFileOperation(
    mode="read"|"write"|"edit"|"search"|"analyze",
    submode="...",
    ...50 different parameters...
)
```

**Just Right:**
```
Read(file_path, offset?, limit?)  # Flexible pagination
Edit(file_path, old_string, new_string)  # Clear intent
Grep(pattern, glob?, output_mode?)  # Controlled verbosity
```

**Why:** Fine-grained tools overwhelm selection; coarse tools obscure intent. Right granularity balances simplicity and power.

## Common Anti-Patterns

### ❌ ANTI-PATTERN: Using Bash for File Operations

**Wrong:**
```json
{
  "tool": "bash",
  "command": "cat /path/to/file.txt"
}
```

**Right:**
```json
{
  "tool": "read_file",
  "file_path": "/path/to/file.txt"
}
```

**Detection:**
```rust
if tool_name == "bash" && args["command"].contains("cat ") {
    warn!("Use Read tool instead of 'cat' command");
}
```

**Why:** Specialized tools provide better structure, error handling, and performance. Bash output requires parsing; tool output is structured.

### ❌ ANTI-PATTERN: Sequential File Reads Instead of Search

**Wrong:**
```
Read(file="file1.rs") → check content
Read(file="file2.rs") → check content
Read(file="file3.rs") → check content
...repeat for 50 files...
```

**Right:**
```
Grep(pattern="search_term", glob="**/*.rs", output_mode="content")
```

**Detection:**
```rust
if tool_name == "read_file" {
    let recent_reads = context.recent_tools
        .filter(|t| t.tool == "read_file")
        .count();

    if recent_reads > 3 {
        warn!("Consider using Grep for content search across files");
    }
}
```

**Why:** Grep is optimized for search; sequential reads waste tokens and time.

### ❌ ANTI-PATTERN: Using Write on Existing Files

**Wrong:**
```json
{
  "tool": "write_file",
  "file_path": "/existing/file.txt",
  "content": "new content"
}
```

**Right:**
```json
{
  "tool": "edit_file",
  "file_path": "/existing/file.txt",
  "old_string": "text to replace",
  "new_string": "new text"
}
```

**Detection:**
```rust
if tool_name == "write_file" {
    if file_exists(args["file_path"]) {
        return Err("File exists. Use Edit to modify existing files.");
    }
}
```

**Why:** Write tool overwrites completely; Edit ensures intentional changes with exact matching.

### ❌ ANTI-PATTERN: Using Echo for User Communication

**Wrong:**
```json
{
  "tool": "bash",
  "command": "echo 'I am analyzing the file now...'"
}
```

**Right:**
```
# Direct output in LLM response:
"I'm analyzing the file now..."
```

**Why:** Echo wastes a tool call. LLMs should output text directly to communicate with users.

### ❌ ANTI-PATTERN: Placeholder Parameters

**Wrong:**
```json
// Call tool with placeholder, planning to fill it in later
{
  "tool": "edit_file",
  "file_path": "{FILE_PATH_FROM_PREVIOUS_CALL}",
  "old_string": "...",
  "new_string": "..."
}
```

**Right:**
```
// Wait for previous call to complete, then use actual value
1. First call completes → get file_path
2. Second call uses actual file_path value
```

**Why:** Placeholders fail validation. Tools must be called sequentially when dependencies exist.

### ❌ ANTI-PATTERN: Batching Multiple Independent Calls

**Wrong:**
```
# Three separate messages
Message 1: Read(file="a.txt")
Message 2: Read(file="b.txt")
Message 3: Read(file="c.txt")
```

**Right:**
```
# Single message with parallel calls
Read(file="a.txt")
Read(file="b.txt")
Read(file="c.txt")
```

**Why:** Parallel calls in one message execute faster and reduce latency.

## LLM Guidance Patterns

### Pattern 1: Decision Trees in Documentation

```
Tool: Grep

DECISION TREE:
├─ Know exact file path? → Use Read instead
├─ Searching for filename? → Use Glob instead
├─ Searching content across files? → Use Grep ✓
├─ Need semantic understanding? → Use Tree-sitter
└─ Exploring unknown codebase? → Use Task agent

WHEN TO USE:
✓ Finding files containing specific text
✓ Searching across multiple files
✓ Pattern matching with regex

WHEN NOT TO USE:
✗ Searching single known file (Read is faster)
✗ Finding files by name (use Glob)
✗ Need AST/semantic info (use Tree-sitter)
```

### Pattern 2: Progressive Examples

```
BASIC USAGE:
  Grep(pattern="TODO")
  → Find files with TODOs

INTERMEDIATE:
  Grep(pattern="function.*auth", glob="**/*.ts", output_mode="content")
  → Find authentication functions in TypeScript files

ADVANCED:
  Grep(pattern="class\\s+\\w+", multiline=true, -C=3)
  → Find class definitions with context, multiline mode
```

### Pattern 3: Explicit Anti-Patterns

```
COMMON MISTAKES:

❌ DON'T: Grep(pattern="common_word")
   → Will match too many files, slow performance

✓ DO: Grep(pattern="common_word", glob="src/**/*.rs")
   → Filter by file pattern for better performance

❌ DON'T: bash("grep -r 'pattern' .")
   → Slower, output harder to parse

✓ DO: Grep(pattern="pattern", output_mode="content")
   → Native tool with structured output
```

### Pattern 4: Contextual Hints

```
HINTS:
- Large codebase? Use glob parameter to filter files
- Too many matches? Narrow your pattern or add context
- Need line numbers? Use output_mode="content" with -n flag
- Multiline pattern? Set multiline=true
```

## Performance Optimization

### ✅ DO: Parallel Execution When Possible

**Good:**
```
# Single message with 3 independent calls
<tool_calls>
  <tool name="read_file">file1.txt</tool>
  <tool name="read_file">file2.txt</tool>
  <tool name="read_file">file3.txt</tool>
</tool_calls>
```

**Bad:**
```
# Three sequential messages
Message: Read file1.txt
Wait for response...
Message: Read file2.txt
Wait for response...
Message: Read file3.txt
```

**Why:** Parallel execution is 3x faster for independent operations.

### ✅ DO: Use Appropriate Output Modes

**Good:**
```
# Discovery phase - find files
Grep(pattern="error_handler", output_mode="files_with_matches")
→ Returns file paths only

# Inspection phase - see content
Grep(pattern="error_handler", output_mode="content", file_path="found_file.rs")
→ Returns matching lines
```

**Bad:**
```
# Always requesting full content
Grep(pattern="error_handler", output_mode="content")
→ Returns thousands of lines across all files
```

**Why:** Output modes control token usage. Use minimal output for discovery, detailed for inspection.

### ✅ DO: Leverage Caching

**Good:**
```rust
RuntimeProfile {
    cacheable: true,
    cache_ttl: Some(Duration::from_secs(60)),
    cache_key_fn: Some(|args| {
        format!("{}:{}", args["file_path"], args["offset"])
    }),
}
```

**Bad:**
```rust
RuntimeProfile {
    cacheable: false,  // Always re-execute
}
```

**Why:** Read-only operations can be cached, reducing latency and load.

### ✅ DO: Set Appropriate Timeouts

**Good:**
```rust
match tool_category {
    FileRead => Duration::from_secs(10),
    Search => Duration::from_secs(30),
    BuildCommand => Duration::from_secs(300),
    Test => Duration::from_secs(600),
}
```

**Bad:**
```rust
// One timeout for all tools
Duration::from_secs(120)
```

**Why:** Different operations have different time requirements. Appropriate timeouts prevent premature cancellation or hanging.

## Safety & Security

### ✅ DO: Validate All Inputs

**Good:**
```rust
// Validate file paths
if !path.starts_with(&workspace_root) {
    return Err("Path outside workspace boundary");
}

// Validate commands against policy
if is_dangerous_command(&cmd) {
    return Err("Command blocked by security policy");
}

// Validate parameters
if pattern.is_empty() {
    return Err("Pattern cannot be empty");
}
```

**Bad:**
```rust
// Trust all inputs
execute_command(&args["command"])?;
```

**Why:** Validation prevents path traversal, command injection, and other security issues.

### ✅ DO: Implement Progressive Trust

**Good:**
```
Untrusted workspace:
  - Read: Allow
  - Search: Allow
  - Write: Prompt
  - Execute: Prompt

Trusted workspace:
  - Read: Allow
  - Search: Allow
  - Write: Allow
  - Execute: Prompt (still ask for commands)

Full-auto mode:
  - Read: Allow
  - Search: Allow
  - Write: Allow
  - Execute: Allow (safe commands only)
```

**Bad:**
```
# All-or-nothing trust
if trusted {
    allow_all_tools();
}
```

**Why:** Progressive trust balances security and usability. Even trusted workspaces should prompt for destructive operations.

### ✅ DO: Audit Sensitive Operations

**Good:**
```rust
match risk_level {
    RiskLevel::Low => AuditLevel::Basic,      // Log tool name
    RiskLevel::Medium => AuditLevel::Detailed, // Log parameters
    RiskLevel::High => AuditLevel::Full,       // Log params + output
    RiskLevel::Critical => AuditLevel::Full,   // + require approval
}
```

**Bad:**
```rust
// No audit trail
execute_tool(name, args)?;
```

**Why:** Audit trails enable debugging, compliance, and incident response.

## Error Handling

### ✅ DO: Provide Actionable Error Messages

**Good:**
```rust
if file_not_found {
    return Err(ToolError {
        error_type: ErrorType::FileNotFound,
        message: "File '/path/to/file.txt' not found",
        suggested_actions: vec![
            "Verify the file path is correct",
            "Use Glob to find files matching a pattern",
            "Check if file was moved or deleted"
        ],
        examples: vec![
            "Glob(pattern='**/file.txt')"
        ]
    });
}
```

**Bad:**
```rust
if file_not_found {
    return Err("Error: ENOENT");
}
```

**Why:** Actionable errors help LLMs recover automatically. Cryptic errors require user intervention.

### ✅ DO: Classify Errors by Type

**Good:**
```rust
pub enum ToolErrorType {
    FileNotFound,
    PermissionDenied,
    InvalidParameters,
    Timeout,
    PolicyViolation,
    ExecutionFailed,
    NetworkError,
}
```

**Bad:**
```rust
// All errors are generic
return Err("Tool execution failed");
```

**Why:** Error classification enables appropriate retry strategies. Network errors → retry; invalid parameters → don't retry.

### ✅ DO: Suggest Alternatives on Failure

**Good:**
```rust
if policy_denied {
    return Err(ToolError {
        message: "Tool 'write_file' denied by policy",
        alternatives: vec![
            Alternative {
                tool: "edit_file",
                reason: "If modifying existing file",
                example: "Edit(file_path='...', old_string='...', new_string='...')"
            }
        ]
    });
}
```

**Bad:**
```rust
if policy_denied {
    return Err("Permission denied");
}
```

**Why:** Suggesting alternatives guides LLMs to working solutions.

## Evolution & Deprecation

### ✅ DO: Deprecate Gracefully

**Good:**
```rust
ToolRegistration::new("old_tool", ...)
    .with_deprecated(true)
    .with_deprecation_message(
        "This tool is deprecated. Use 'new_tool' instead. \
         Old_tool will be removed in version 2.0. \
         Migration: Old_tool(x, y) → New_tool(x=x, y=y, mode='auto')"
    )
```

**Bad:**
```rust
// Immediate removal
// registry.register_tool("old_tool", ...); // deleted
```

**Why:** Graceful deprecation gives time for migration. Clear migration guides prevent disruption.

### ✅ DO: Version Tool Schemas

**Good:**
```rust
ToolVersion {
    major: 2,  // Breaking changes
    minor: 1,  // New features, backward compatible
    patch: 3,  // Bug fixes
}

// Changelog
"v2.0.0: Changed 'mode' parameter to 'output_mode' (breaking)
 v1.5.0: Added 'glob' parameter for file filtering
 v1.4.1: Fixed timeout handling bug"
```

**Bad:**
```
// No versioning, breaking changes without notice
```

**Why:** Versioning communicates change impact. Major versions signal breaking changes.

### ✅ DO: Maintain Backward Compatibility

**Good:**
```rust
// Support both old and new parameter names
let output_mode = args.get("output_mode")
    .or_else(|| args.get("mode"))  // old name
    .and_then(|v| v.as_str())
    .unwrap_or("files_with_matches");
```

**Bad:**
```rust
// Immediately break old parameter names
let output_mode = args["output_mode"].as_str()?;  // breaks if using "mode"
```

**Why:** Backward compatibility prevents breaking existing prompts and workflows.

## Summary Checklist

### Tool Design
- [ ] Clear semantic purpose (not generic)
- [ ] Appropriate granularity (not too fine or coarse)
- [ ] Fail-safe defaults
- [ ] Composable with other tools
- [ ] Rich documentation with decision trees

### Documentation
- [ ] Clear "when to use" criteria
- [ ] Explicit "when NOT to use" anti-patterns
- [ ] Working examples with expected output
- [ ] Anti-pattern examples with corrections
- [ ] Alternative tool suggestions

### Safety
- [ ] Input validation (paths, commands, patterns)
- [ ] Risk classification
- [ ] Progressive trust levels
- [ ] Audit logging for sensitive operations
- [ ] Policy enforcement

### Performance
- [ ] Parallel execution support
- [ ] Appropriate caching strategy
- [ ] Timeout configuration by category
- [ ] Resource usage profiling
- [ ] Output mode controls

### Error Handling
- [ ] Actionable error messages
- [ ] Error type classification
- [ ] Suggested recovery actions
- [ ] Alternative tool suggestions
- [ ] Retry guidance

### Evolution
- [ ] Version numbering
- [ ] Deprecation warnings with migration guides
- [ ] Backward compatibility for parameters
- [ ] Changelog maintenance
- [ ] Stability level indicators

---

Following these practices results in tool registries that are:
- **Easy for LLMs to use correctly**
- **Hard for LLMs to misuse**
- **Safe and secure by default**
- **Performant and scalable**
- **Evolvable without breaking changes**
