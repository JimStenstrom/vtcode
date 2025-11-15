# Tool Policy vs. Standard Operating Procedure (SOP)

## Overview

While both Tool Policies and Standard Operating Procedures guide tool usage in agentic LLM systems, they serve fundamentally different purposes and operate at different levels of the system.

## Core Distinction

### Tool Policy
**Governance layer that defines WHAT is allowed and WHEN**

- Specifies authorization and permissions
- Enforces constraints and boundaries
- Makes allow/deny decisions
- Protects against unauthorized or unsafe operations
- Operates at **decision-making time**

### Standard Operating Procedure (SOP)
**Execution layer that defines HOW to accomplish tasks**

- Specifies step-by-step processes
- Ensures consistency and repeatability
- Provides implementation patterns
- Guides workflow orchestration
- Operates at **execution time**

## Detailed Comparison

| Aspect | Tool Policy | Standard Operating Procedure |
|--------|-------------|------------------------------|
| **Purpose** | Control and authorization | Process and consistency |
| **Question Answered** | "May I do this?" | "How should I do this?" |
| **Enforcement** | Automatic (system-level) | Guidance (agent-level) |
| **Granularity** | Per-tool or per-category | Per-task or per-workflow |
| **Flexibility** | Rigid (security boundary) | Flexible (best practice) |
| **Override** | Requires privilege escalation | Can be adapted to context |
| **Failure Mode** | Block execution | Suboptimal execution |

## Examples

### Tool Policy Examples

```toml
# vtcode.toml
[tools.policies]

# POLICY: read_file is always allowed (read-only, low risk)
read_file = "allow"

# POLICY: write_file requires user confirmation (modifies files)
write_file = "prompt"

# POLICY: bash is denied by default in untrusted workspaces
bash = "deny"

# POLICY: web_fetch only allowed to specific domains
[tools.constraints.web_fetch]
allowed_domains = ["api.github.com", "docs.rs"]
```

**Characteristics:**
- Binary decisions (allow/prompt/deny)
- Enforced automatically before execution
- Based on risk, trust level, and workspace context
- Cannot be bypassed without changing configuration
- Applies universally to all tasks using the tool

### SOP Examples

```markdown
# SOP: Creating a Git Commit

PROCEDURE:
1. Run `git status` to see untracked files
2. Run `git diff` to see staged and unstaged changes
3. Review changes and determine commit message
4. Add untracked files to staging: `git add <files>`
5. Create commit: `git commit -m "message"`
6. Run `git status` again to verify success

IMPORTANT:
- Never skip review of changes
- Follow commit message style guide
- Do not commit secrets or credentials
- If pre-commit hook modifies files, verify before amending
```

```markdown
# SOP: Editing Files

PROCEDURE:
1. Read the file to understand current contents
2. Use Edit tool with exact old_string/new_string
3. Verify the change was applied correctly

ANTI-PATTERNS:
- Don't use Write on existing files
- Don't guess at old_string content
- Don't use bash sed/awk for editing

POLICY CHECK:
- Edit tool policy must be "allow" or "prompt"
```

**Characteristics:**
- Step-by-step instructions
- Includes best practices and anti-patterns
- Guides agent behavior but isn't enforced
- Can be adapted based on context
- Applies to specific workflows

## Interaction Between Policies and SOPs

### Hierarchical Relationship

```
┌─────────────────────────────────────────┐
│         User Request                     │
│    "Create a new feature"                │
└───────────────┬─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────┐
│   SOP: Feature Implementation            │
│                                          │
│   1. Create feature branch               │ ◄── SOP guides workflow
│   2. Implement code changes              │
│   3. Write tests                         │
│   4. Commit changes                      │
│   5. Create pull request                 │
└───────────────┬─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────┐
│   Individual Tool Calls                  │
│                                          │
│   - bash("git checkout -b feature")      │ ◄── Policy decides: allow/deny
│   - edit_file(...)                       │ ◄── Policy decides: allow/deny
│   - write_file(...)                      │ ◄── Policy decides: allow/deny
│   - bash("git commit ...")               │ ◄── Policy decides: allow/deny
└─────────────────────────────────────────┘
```

### Complementary Nature

**Policies** set boundaries; **SOPs** optimize within those boundaries.

Example:

```
POLICY: "bash tool requires user approval (prompt)"
   ↓
SOP: "When user approves bash, prefer these patterns:
      - Chain related commands with &&
      - Use absolute paths to maintain working directory
      - Avoid interactive commands (not supported)"
```

## Real-World Scenarios

### Scenario 1: File Modification

**Policy Layer:**
```rust
// System automatically enforces this
if tool == "write_file" && file_exists(path) {
    return PolicyDecision::Deny {
        reason: "write_file cannot overwrite existing files",
        alternative: "Use edit_file instead"
    };
}
```

**SOP Layer:**
```markdown
# When modifying files:
1. Check if file exists (use file_ops or try reading)
2. If exists → use Edit tool
3. If new → use Write tool
4. Always verify changes after modification
```

**Interaction:**
- Policy **prevents** using wrong tool (Write on existing file)
- SOP **guides** to use correct tool (Edit for existing files)
- Policy is enforced; SOP is guidance

### Scenario 2: Git Operations

**Policy Layer:**
```toml
[tools.policies]
bash = "prompt"  # All bash commands require approval

[tools.constraints.bash]
deny_patterns = ["push --force", "reset --hard", "rm -rf"]
```

**SOP Layer:**
```markdown
# Git Commit Workflow:
1. Review changes: `git status && git diff`
2. Stage files: `git add <files>`
3. Commit: `git commit -m "message"`
4. Verify: `git status`

POLICY NOTES:
- git commands will trigger user prompt (bash policy)
- destructive commands (force push) are blocked by policy
- follow SOP even if in full-auto mode
```

**Interaction:**
- Policy **blocks** dangerous commands unconditionally
- Policy **prompts** for all git operations (via bash)
- SOP **describes** the correct sequence and best practices

### Scenario 3: Search Operations

**Policy Layer:**
```toml
[tools.policies]
grep = "allow"  # Safe, read-only operation
bash = "prompt"

[tools.alternatives]
# Automatically suggest Grep when bash grep is used
"bash:grep" = "grep"
```

**SOP Layer:**
```markdown
# Searching for Code:

DECISION TREE:
├─ Known file path → Read file, manual inspection
├─ Search across files → Use Grep tool
├─ Complex exploration → Use Task agent
└─ Never use: bash grep command

GREP USAGE PATTERN:
1. Discovery: output_mode="files_with_matches"
2. Inspection: output_mode="content" on specific files
3. Always use glob parameter to filter files
```

**Interaction:**
- Policy **allows** Grep without prompt (safe)
- Policy **prompts** for bash (even grep command)
- SOP **guides** when to use Grep vs alternatives
- SOP **specifies** how to use Grep effectively

## Design Patterns

### Pattern 1: Policy-Enforced, SOP-Guided

```
┌─────────────────────────────────────┐
│  Policy: Hard boundary              │
│  "write_file requires prompt"       │
└──────────────┬──────────────────────┘
               │ Enforces
               ▼
┌─────────────────────────────────────┐
│  SOP: Best practice within policy   │
│  "When creating files:              │
│   1. Verify parent dir exists       │
│   2. Use Write for new files        │
│   3. Validate content before write" │
└─────────────────────────────────────┘
```

### Pattern 2: Policy as SOP Input

```
SOP references policy configuration:

"Before executing workflow:
 1. Check tool policy for required tools
 2. If any tool is 'deny', stop and ask user
 3. If 'prompt', warn user approval needed
 4. If 'allow', proceed with workflow"
```

### Pattern 3: SOP Prevents Policy Violations

```
SOP includes policy awareness:

"Don't attempt operations that will be blocked:
 ❌ Write to existing files (policy blocks)
 ✓ Edit existing files instead

 ❌ Force push to main (policy blocks)
 ✓ Push to feature branch

This prevents wasted approval requests."
```

## Configuration Examples

### Policy Configuration (vtcode.toml)

```toml
[tools]
default_policy = "prompt"

[tools.policies]
# Read-only operations: auto-allow
read_file = "allow"
list_files = "allow"
grep = "allow"

# Modifications: require approval
write_file = "prompt"
edit_file = "prompt"

# Execution: context-dependent
bash = "prompt"

[tools.trusted_workspaces]
"/home/user/trusted-project" = {
    write_file = "allow",
    edit_file = "allow",
    bash = "allow"
}

[tools.constraints.bash]
allow_commands = ["git", "npm", "cargo", "pytest"]
deny_patterns = [
    "rm -rf",
    "dd ",
    "mkfs",
    "format",
    ":(){ :|:& };:",  # fork bomb
]
```

### SOP Configuration (documentation/prompts)

```markdown
# File Operations SOP

## Reading Files
WHEN: Need to inspect file contents
TOOLS: read_file (policy: allow)
PROCEDURE:
1. Use Read with full file path
2. For large files, use offset/limit
3. Never use bash cat

## Writing Files
WHEN: Creating new files
TOOLS: write_file (policy: prompt in untrusted workspace)
PROCEDURE:
1. Verify file doesn't exist
2. Ensure parent directory exists
3. Call Write tool (may require approval)
4. Verify file was created

## Editing Files
WHEN: Modifying existing files
TOOLS: edit_file (policy: prompt in untrusted workspace)
PROCEDURE:
1. Read file to find exact old_string
2. Use Edit with exact match (no regex)
3. Verify change was applied
4. If multi-line, use heredoc format

## Anti-Patterns
❌ write_file on existing → Use edit_file
❌ bash cat → Use read_file
❌ bash sed/awk → Use edit_file
❌ Guessing old_string → Read first
```

## Benefits of Separation

### Clear Responsibility
- **Policies** = Security team
- **SOPs** = Engineering best practices
- No confusion about who owns what

### Independent Evolution
- Change SOP without security review
- Change policy without updating all SOPs
- Test policy changes without modifying workflows

### Layered Defense
- Policy catches unauthorized operations
- SOP prevents policy violations through guidance
- Policy violations trigger review of SOP effectiveness

### Flexibility
- Strict policies in untrusted contexts
- Relaxed policies in trusted contexts
- SOPs remain consistent across contexts

## Anti-Patterns

### ❌ Mixing Policy and SOP

**Wrong:**
```toml
# Policy file trying to specify procedures
[tools.procedures.write_file]
steps = [
    "verify_file_not_exists",
    "create_parent_dir",
    "write_content"
]
```

**Right:**
```toml
# Policy specifies WHAT is allowed
[tools.policies]
write_file = "prompt"

[tools.constraints.write_file]
must_not_exist = true
```

```markdown
<!-- SOP specifies HOW to use it -->
# Write File Procedure
1. Verify file doesn't exist
2. Create parent directory if needed
3. Call write_file (requires approval)
```

### ❌ SOP Without Policy Awareness

**Wrong:**
```markdown
# SOP that assumes all tools are allowed
1. Write new file
2. Edit existing file
3. Run bash command
(Doesn't mention any might be blocked)
```

**Right:**
```markdown
# SOP that acknowledges policies
1. Write new file (requires approval in untrusted workspace)
2. Edit existing file (requires approval in untrusted workspace)
3. Run bash command (always requires approval)

NOTE: In trusted workspaces, steps 1-2 auto-approve
```

### ❌ Policy as Complete Documentation

**Wrong:**
```toml
# Expecting policy to teach usage
[tools.policies]
grep = "allow"  # Use this for searching files
```

**Right:**
```toml
# Policy only specifies authorization
[tools.policies]
grep = "allow"
```

```markdown
<!-- SOP provides usage guidance -->
# Grep Tool Usage
PURPOSE: Search file contents across codebase
WHEN TO USE: Finding code patterns, TODO comments, etc.
HOW TO USE:
  - Discovery: output_mode="files_with_matches"
  - Content: output_mode="content"
```

## Summary

### Tool Policy
- **What:** Authorization framework
- **When:** At decision/execution time
- **Who:** System/security team
- **How:** Automatically enforced
- **Why:** Safety and security boundaries

### Standard Operating Procedure
- **What:** Best practice guidelines
- **When:** During workflow planning
- **Who:** Engineering team
- **How:** Agent guidance/documentation
- **Why:** Consistency and effectiveness

### Relationship
Policies and SOPs are **complementary**:
- Policies enforce boundaries (the "guardrails")
- SOPs optimize within boundaries (the "roadmap")
- Together they create safe, effective, consistent agent behavior

The best systems have **both**:
- Strong policies prevent catastrophic errors
- Clear SOPs prevent inefficient patterns
- Policy-aware SOPs minimize friction
- SOP-informed policies are appropriately restrictive

---

**Further Reading:**
- `/docs/design/TOOL_REGISTRY_DESIGN.md` - Complete registry architecture
- `/docs/design/TOOL_BEST_PRACTICES.md` - Detailed best practices
- `/vtcode-core/src/tool_policy.rs` - Policy implementation
- `/docs/vtcode_tools_policy.md` - Policy configuration guide
