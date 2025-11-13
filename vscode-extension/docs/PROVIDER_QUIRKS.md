# Provider-Specific Quirks and Edge Cases

**Version**: 1.0
**Date**: November 13, 2025
**Status**: Critical Documentation for Phase 3

---

## Overview

This document catalogs provider-specific quirks, edge cases, and undocumented behaviors in the VTCode VSCode extension. Understanding these quirks is essential for maintaining, debugging, and extending the extension.

**Critical**: This documentation resolves the Phase 3 critical issue: "PROVIDER-SPECIFIC QUIRKS NOT DOCUMENTED"

---

## Table of Contents

1. [Document Selector Quirks](#1-document-selector-quirks)
2. [Regex Pattern Edge Cases](#2-regex-pattern-edge-cases)
3. [MCP Provider Quirks](#3-mcp-provider-quirks)
4. [File Size and Line Count Limits](#4-file-size-and-line-count-limits)
5. [Context Truncation Quirks](#5-context-truncation-quirks)
6. [File Exclusion Patterns](#6-file-exclusion-patterns)
7. [Terminal Tool Detection](#7-terminal-tool-detection)
8. [Shell Command Extraction](#8-shell-command-extraction)
9. [Workspace Trust Dependencies](#9-workspace-trust-dependencies)
10. [Language Provider Hover Patterns](#10-language-provider-hover-patterns)
11. [Symbol Provider Quirks](#11-symbol-provider-quirks)
12. [Code Participant Language Filtering](#12-code-participant-language-filtering)
13. [Git Status Parsing](#13-git-status-parsing)
14. [Conversation Context Message Limit](#14-conversation-context-message-limit)
15. [MCP Server Definition Provider](#15-mcp-server-definition-provider)
16. [Config File Selection Priority](#16-config-file-selection-priority)
17. [Human-in-the-Loop Setting Regex](#17-human-in-the-loop-setting-regex)
18. [Provider Name Normalization](#18-provider-name-normalization)
19. [Tool Approval Detail Length](#19-tool-approval-detail-length)
20. [CLI Detection Timeout](#20-cli-detection-timeout)
21. [Context7 Integration Limits](#21-context7-integration-limits)
22. [Selection vs Visible Range Fallback](#22-selection-vs-visible-range-fallback)

---

## 1. Document Selector Quirks

**Location**: `src/languageFeatures.ts:306-310`

### Quirk: Triple Selector Pattern

The VTCode language provider uses THREE different matching patterns:

```typescript
export const VT_CODE_DOCUMENT_SELECTOR: vscode.DocumentSelector = [
    { language: 'vtcode-config', scheme: 'file' },
    { language: 'vtcode-config', scheme: 'untitled' },
    { pattern: '**/vtcode.toml', scheme: 'file' }
];
```

### Edge Cases

1. **Untitled Documents**: Untitled documents with `vtcode-config` language ID will match, even without a filename
2. **Pattern Matching**: Pattern matching (`**/vtcode.toml`) requires `file` scheme only
3. **Match Logic**: Files match by EITHER language ID OR filename pattern, not both

### Impact

- A file named `vtcode.toml` without the language ID will still get language features
- An untitled document with language ID `vtcode-config` gets language features
- Files in memory (non-file schemes) won't match the pattern selector

### Best Practice

When testing language features, ensure you test all three matching scenarios:
- File-based `vtcode.toml`
- Untitled documents with language ID
- Documents with language ID but different filenames

---

## 2. Regex Pattern Edge Cases

### 2.1 Mention Parsing

**Location**: `src/utils/mentionParser.ts:25`

```typescript
const MENTION_REGEX = /@(\w+)(?!\w)/g;
```

### Edge Cases

1. **Character Restrictions**: Only matches `[A-Za-z0-9_]` (alphanumeric + underscore)
2. **No Hyphens**: `@workspace-special` will NOT match (hyphen breaks the pattern)
3. **Numeric Mentions**: `@workspace123` WILL match
4. **Email Protection**: `user@example.com` will NOT match (good!)
5. **Global State**: Regex has `.lastIndex` state that must be reset between uses

### Valid Mentions

- `@workspace` ✅
- `@code` ✅
- `@terminal` ✅
- `@git` ✅
- `@myParticipant123` ✅

### Invalid Mentions

- `@workspace-special` ❌ (hyphen)
- `@my-participant` ❌ (hyphen)
- `@code!` ❌ (special char)
- `@git.branch` ❌ (dot breaks it)

### Best Practice

Always reset regex state:
```typescript
MENTION_REGEX.lastIndex = 0;
```

---

### 2.2 Section Header Regex

**Location**: `src/languageFeatures.ts:432`

```typescript
const match = text.match(/^\s*(\[\[|\[)\s*([^\]]+?)\s*\]{1,2}\s*$/);
```

### Edge Cases

1. **Non-greedy Match**: `[^\]]+?` can cause issues with malformed headers
2. **Nested Brackets**: Matches both `[section]` and `[[section]]`
3. **Whitespace Tolerance**: Leading/trailing whitespace is allowed
4. **Empty Sections**: `[ ]` would match but capture empty string

### Malformed Examples

```toml
[section     # Missing closing bracket - won't match
[section]]   # Extra bracket - won't match correctly
[[section]   # Mismatched brackets - unpredictable
```

---

### 2.3 Word Range Regex Inconsistency

**Location**: `src/languageFeatures.ts:455,477`

```typescript
// Section hover uses:
/[A-Za-z0-9_.]+/

// Key hover uses:
/[A-Za-z_.][A-Za-z0-9_.]+/
```

### Edge Cases

1. **Starting Character**: Key hover requires letter/underscore/dot first
2. **Numbers**: Section hover allows starting with numbers, key hover doesn't
3. **Single Character**: Key hover requires at least 2 characters

### Impact

- Section `[1agent]` would get hover, but key `1provider` would not
- This inconsistency may confuse users

---

## 3. MCP Provider Quirks

**Location**: `src/mcpTools.ts`

### 3.1 Hardcoded Timeouts

```typescript
// Tool discovery timeout
const DISCOVERY_TIMEOUT = 5000; // 5 seconds

// Tool execution timeout
const EXECUTION_TIMEOUT = 30000; // 30 seconds
```

### Edge Cases

1. **Not Configurable**: No user setting to adjust these timeouts
2. **Slow Providers**: Providers taking >5s for discovery will fail
3. **Long Operations**: Tools running >30s will be terminated
4. **No Retry Logic**: Failed timeout = immediate failure

### Impact

- Users with slow MCP providers cannot increase timeout
- Long-running tools (e.g., large file processing) will timeout

### Recommendation

Make timeouts configurable in `vtcode.toml`:
```toml
[mcp]
discovery_timeout_seconds = 10
execution_timeout_seconds = 60
```

---

### 3.2 Provider/Tool Name Parsing

**Location**: `src/mcpTools.ts:50-61`

```typescript
if (invocation.tool.includes("/")) {
    // Treat as provider/tool format
    const [provider, tool] = invocation.tool.split("/");
}
```

### Edge Cases

1. **Slash Ambiguity**: Tool names containing `/` are interpreted as `provider/tool`
2. **No Escaping**: No way to use literal slash in tool name
3. **Multiple Slashes**: `provider/category/tool` will split incorrectly

### Examples

- `filesystem/read` → provider: `filesystem`, tool: `read` ✅
- `path/to/file` → provider: `path`, tool: `to/file` ❌
- `my-tool` → No split, used as-is ✅

### Best Practice

Avoid slashes in MCP tool names, or use explicit provider specification.

---

### 3.3 TOML Parser Limitations

**Location**: `src/mcpTools.ts:299-392`

**⚠️ WARNING**: Line 301 states: "This is a simplified parser - in production, use a proper TOML library"

### Known Limitations

1. **No Multiline Strings**: Cannot parse multiline strings with `'''` or `"""`
2. **No Inline Tables**: `{ key = "value" }` syntax not supported
3. **No Complex Arrays**: Nested arrays may parse incorrectly
4. **Minimal Escaping**: Only basic string quote removal: `/^["']|["']$/g`
5. **No Comments**: May not handle inline comments correctly
6. **No Date/Time**: TOML date/time types not supported

### Examples of Unsupported TOML

```toml
# Multiline string - WILL FAIL
description = '''
This is a long
multiline description
'''

# Inline table - WILL FAIL
server = { host = "localhost", port = 8080 }

# Complex nested - MAY FAIL
[[mcp.providers]]
args = [["--nested", "value"], ["--other"]]
```

### Fallback Behavior

**Location**: `src/mcpTools.ts:258`

If JSON parsing fails, falls back to raw stdout:
```typescript
} catch {
    return stdout; // Returns unparsed output
}
```

### Recommendation

Replace custom TOML parser with `@iarna/toml` or similar library:
```typescript
import * as TOML from '@iarna/toml';
const config = TOML.parse(content);
```

---

## 4. File Size and Line Count Limits

### 4.1 Active File Line Limit

**Location**: `src/chatView.ts:488`

```typescript
if (lineCount <= 1000) {
    // Include full file content
}
```

### Edge Cases

1. **Silent Exclusion**: Files >1000 lines excluded from context WITHOUT warning
2. **No Configuration**: Limit is hardcoded
3. **No Partial Include**: File is either fully included or completely excluded

### Impact

Users working on large files won't understand why context is missing.

---

### 4.2 IDE Context Limits

**Location**: `src/extension.ts:166-169`

```typescript
const MAX_IDE_CONTEXT_CHARS = 6000;
const MAX_FULL_DOCUMENT_CONTEXT_LINES = 400;
const ACTIVE_EDITOR_CONTEXT_WINDOW = 80;
const MAX_VISIBLE_EDITOR_CONTEXTS = 3;
```

### Multi-Layered Limits

1. **Character Limit**: Maximum 6000 characters total
2. **Line Limit**: Full document only if ≤400 lines
3. **Window Size**: 80 lines around cursor when document >400 lines
4. **Editor Count**: Only 3 visible editors included

### Edge Cases

1. **Priority Order**: Character limit applied AFTER line selection
2. **No Smart Truncation**: May cut mid-sentence
3. **Editor Priority**: First 3 visible editors, others ignored

### Example Scenario

```
User has 5 editors open:
- Editor 1: 500 lines → 80-line window around cursor
- Editor 2: 300 lines → Full content
- Editor 3: 100 lines → Full content
- Editor 4: 50 lines  → Ignored (4th editor)
- Editor 5: 50 lines  → Ignored (5th editor)
```

---

### 4.3 Participant-Specific Limits

**Code Participant** (`src/participants/codeParticipant.ts:68`):
- Maximum 50 lines per code snippet
- No indication when truncated

**Terminal Participant** (`src/participants/terminalParticipant.ts:39`):
- Last 20 lines of terminal output
- Last 5 commands from history

**Workspace Participant** (`src/participants/workspaceParticipant.ts:36`):
- Maximum 100 files in workspace scan
- Hardcoded exclusions: `**/node_modules/**`

### Impact Table

| Participant | Limit | Truncation Indicator | Configurable |
|------------|-------|---------------------|--------------|
| Code | 50 lines | No | No |
| Terminal | 20 lines | No | No |
| Workspace | 100 files | No | No |
| Active File | 1000 lines | No | No |

---

## 5. Context Truncation Quirks

### 5.1 Tool Approval Detail Truncation

**Location**: `src/chatView.ts:357-363`

```typescript
let detail = JSON.stringify(invocation.arguments, null, 2);
if (detail.length > 1200) {
    detail = detail.slice(0, 1200) + '...';
}
```

### Edge Cases

1. **Hard Limit**: 1200 characters for modal display
2. **Includes Marker**: The `...` counts toward the limit (should be `1197 + '...'`)
3. **Mid-JSON Cut**: May truncate in middle of JSON structure
4. **No Smart Truncation**: Doesn't respect JSON boundaries

### Example

```json
// If arguments are large, user might see:
{
  "file": "/path/to/very/long/filename",
  "content": "This is a very long content string that gets cut off in the middle of the sen...
```

---

### 5.2 Conversation Context Truncation

**Location**: `src/chatView.ts:557-562`

```typescript
private truncateForContext(content: string, limit: number): string {
    if (content.length <= limit) return content;
    return `${content.slice(0, limit - 20)}… [truncated]`;
}
```

Used with 2000 character limit (Line 549).

### Edge Cases

1. **Marker Overhead**: Reserves 20 characters for ` [truncated]` marker
2. **No Word Boundary**: May cut mid-word
3. **Ellipsis Character**: Uses Unicode ellipsis `…` (U+2026), not three dots

### Best Practice

Consider word-boundary aware truncation:
```typescript
private truncateForContext(content: string, limit: number): string {
    if (content.length <= limit) return content;
    const truncated = content.slice(0, limit - 20);
    const lastSpace = truncated.lastIndexOf(' ');
    const boundary = lastSpace > limit * 0.9 ? lastSpace : truncated.length;
    return `${content.slice(0, boundary)}… [truncated]`;
}
```

---

### 5.3 Documentation Content Truncation

**Location**: `src/context7Integration.ts:290`

```typescript
const truncatedDocs = docs.slice(0, 2000);
```

### Edge Cases

1. **Character-Based**: Truncates at exactly 2000 characters
2. **No Smart Boundary**: May cut mid-sentence or mid-word
3. **No Indicator**: Truncated content has no marker
4. **Silent Truncation**: User/LLM doesn't know content was truncated

---

## 6. File Exclusion Patterns

### 6.1 VTCode Config Search

**Location**: `src/vtcodeConfig.ts:85,340`

```typescript
await vscode.workspace.findFiles(
    "**/vtcode.toml",
    "**/{node_modules,dist,out,.git,target}/**",
    10
);
```

### Hardcoded Exclusions

1. `node_modules` - JavaScript dependencies
2. `dist` - Build output (JavaScript convention)
3. `out` - Build output (TypeScript convention)
4. `.git` - Git repository internals
5. `target` - Rust/Java build output

### Edge Cases

1. **Not Configurable**: Cannot customize exclusion list
2. **10 File Limit**: Only first 10 `vtcode.toml` files found
3. **Missing Exclusions**: Doesn't exclude `build/`, `vendor/`, `venv/`, etc.
4. **Monorepo Issues**: Large monorepos may hit 10-file limit

### Impact

```
Workspace with 12 vtcode.toml files:
- 10 will be found (which 10 is undefined)
- 2 will be silently ignored
- User won't know about missing configs
```

### Recommendation

1. Increase limit or make configurable
2. Add common exclusion patterns for other languages:
   - `**/{build,vendor,venv,.venv,__pycache__}/**`
3. Warn user when limit is reached

---

## 7. Terminal Tool Detection

**Location**: `src/chatView.ts:297-305`

```typescript
private isTerminalTool(toolName: string): boolean {
    const normalized = toolName.toLowerCase();
    return (
        normalized === "run_terminal_cmd" ||
        normalized === "run_shell_command" ||
        normalized === "shell" ||
        normalized === "terminal"
    );
}
```

### Quirks

1. **Case-Insensitive**: All tool names converted to lowercase
2. **Four Aliases**: Recognizes 4 different terminal tool names
3. **Exact Match**: No partial matching or fuzzy logic

### Edge Cases

| Tool Name | Matches | Reason |
|-----------|---------|--------|
| `run_terminal_cmd` | ✅ | Exact match |
| `RUN_TERMINAL_CMD` | ✅ | Case-insensitive |
| `terminal` | ✅ | Alias |
| `terminal_cmd` | ❌ | Not in list |
| `run_terminal` | ❌ | Not exact match |
| `bash` | ❌ | Different tool |

### Impact

If MCP provider uses slightly different naming (e.g., `run_terminal` or `exec_shell`), it won't be recognized as a terminal tool and may have different UI treatment.

---

## 8. Shell Command Extraction

**Location**: `src/chatView.ts:334-342`

```typescript
const candidates = ["command", "cmd", "script", "shell_command", "run"];
for (const key of candidates) {
    if (invocation.arguments[key]) {
        return invocation.arguments[key];
    }
}
```

### Quirks

1. **Priority Order**: First matching parameter wins
2. **Multiple Parameters**: If both `command` and `cmd` exist, only `command` is returned
3. **Five Candidates**: Tries 5 different parameter names

### Priority Order

1. `command` (highest priority)
2. `cmd`
3. `script`
4. `shell_command`
5. `run` (lowest priority)

### Edge Cases

```typescript
// Example 1: Multiple parameters
{
    "command": "ls -la",
    "cmd": "pwd"
}
// Returns: "ls -la" (cmd ignored)

// Example 2: Only lower priority
{
    "run": "echo hello"
}
// Returns: "echo hello"

// Example 3: Non-standard name
{
    "exec": "date"
}
// Returns: undefined (no match)
```

### Impact

Tool providers must use one of the 5 recognized parameter names, or shell command won't be extracted for display.

---

## 9. Workspace Trust Dependencies

### Features Disabled Without Trust

1. **MCP Providers** (`extension.ts:2345`): Only load when workspace trusted
2. **Chat Features** (`chatView.ts:137-142`): Requires trust for activation
3. **Commands** (`package.json:289-343`): Filtered by trust state

### Quirk: No Graceful Degradation

**Location**: Multiple files

```typescript
if (!vscode.workspace.isTrusted) {
    // Complete feature disable, no fallback
    return;
}
```

### Edge Cases

1. **Silent Failure**: Features unavailable with minimal feedback
2. **No Read-Only Mode**: Can't use extension in read-only capacity
3. **Trust Prompt**: User must trust entire workspace, can't trust partially

### Impact

```
Untrusted workspace:
- No MCP tools available
- Chat completely disabled
- Most commands unavailable
- No warning or guidance to user
```

### Recommendation

Implement graceful degradation:
```typescript
if (!vscode.workspace.isTrusted) {
    // Enable read-only features
    enableReadOnlyMode();
    showTrustPrompt("Some features require workspace trust");
}
```

---

## 10. Language Provider Hover Patterns

### 10.1 Section Hover Requirements

**Location**: `src/languageFeatures.ts:454-473`

```typescript
const range = document.getWordRangeAtPosition(position, /[A-Za-z0-9_.]+/);
if (!range) {
    return undefined;
}

const lineText = document.lineAt(position.line).text.trim();
if (!lineText.startsWith('[')) {
    return undefined;  // No hover if line doesn't start with [
}

const section = lineText.replace(/^\[+/, '').replace(/\]+$/, '');
```

### Edge Cases

1. **Line Must Start with Bracket**: Hover only works if trimmed line starts with `[`
2. **String Replacement Logic**: Uses replacement, not regex capture groups
3. **Greedy Bracket Removal**: Removes all leading `[` and trailing `]`

### Examples

```toml
[agent]              # ✅ Hover works
  [agent]            # ✅ Hover works (trimmed)
# [agent]            # ❌ No hover (starts with #)
agent = "test"       # ❌ No hover (no bracket)
[[mcp.providers]]    # ✅ Hover works
[[[malformed]]]      # ⚠️  Hover works but extracts "malformed"
```

---

### 10.2 Key Hover Word Range

**Location**: `src/languageFeatures.ts:476-497`

```typescript
const range = document.getWordRangeAtPosition(position, /[A-Za-z_.][A-Za-z0-9_.]+/);
```

### Edge Cases

1. **Minimum Length**: Requires at least 2 characters
2. **Starting Character**: Must start with letter, underscore, or dot
3. **Cannot Start with Number**: `1provider` won't get hover

### Examples

```toml
provider = "openai"       # ✅ Hover works
enabled = true            # ✅ Hover works
api_key_env = "KEY"       # ✅ Hover works
1provider = "test"        # ❌ No hover (starts with number)
p = "test"                # ❌ No hover (only 1 char)
```

---

## 11. Symbol Provider Quirks

**Location**: `src/languageFeatures.ts:499-560`

### 11.1 Duplicate Section Handling

```typescript
if (!sectionLines.some((entry) => entry.section === section)) {
    sectionLines.push({ section, startLine: line });
}
```

### Quirks

1. **First Occurrence Wins**: Duplicate sections are filtered, first occurrence kept
2. **Range Determined by Order**: Section range extends to next section or EOF
3. **No Warning**: Duplicate sections silently ignored

### Example

```toml
[agent]
provider = "openai"

[security]
human_in_the_loop = true

[agent]  # Duplicate - will be ignored
theme = "dark"
```

Result:
- Only first `[agent]` section appears in symbols
- Second `[agent]` settings still valid TOML but not in symbol tree
- User may not notice duplicate sections

---

### 11.2 Nested Section Creation

**Location**: `src/languageFeatures.ts:522-557`

```typescript
const segments = entry.section.split('.');
```

### Quirks

1. **Automatic Nesting**: `agent.onboarding` creates nested structure
2. **Path-Based Lookup**: Symbols stored by full path (e.g., `agent.onboarding`)
3. **Lazy Creation**: Parent symbols created on-demand if missing

### Edge Cases

```toml
# Defines child before parent
[agent.onboarding]
enabled = true

[agent]
provider = "openai"
```

Result:
- Symbol tree still creates proper nesting
- `agent` symbol created automatically even if not defined first
- Range of parent symbol may not include all children

---

## 12. Code Participant Language Filtering

**Location**: `src/participants/codeParticipant.ts:13-18`

```typescript
canHandle(context: ParticipantContext): boolean {
    return context.activeFile !== undefined &&
           context.activeFile.language !== 'text' &&
           context.activeFile.language !== 'markdown';
}
```

### Quirks

1. **Explicit Exclusions**: `text` and `markdown` files explicitly excluded
2. **No Configuration**: Cannot override exclusion list
3. **Language ID Based**: Uses VS Code language ID, not file extension

### Edge Cases

| File | Language ID | Code Participant |
|------|-------------|-----------------|
| `file.py` | `python` | ✅ Handles |
| `file.js` | `javascript` | ✅ Handles |
| `file.txt` | `text` | ❌ Rejected |
| `file.md` | `markdown` | ❌ Rejected |
| `README` | `text` | ❌ Rejected |
| `code.md` | `markdown` | ❌ Rejected |

### Impact

Users asking `@code` about markdown files won't get code participant context, even if markdown contains code blocks.

### Recommendation

Consider including markdown for code blocks:
```typescript
canHandle(context: ParticipantContext): boolean {
    if (!context.activeFile) return false;

    // Exclude plain text but allow markdown (may contain code)
    if (context.activeFile.language === 'text') return false;

    return true;
}
```

---

## 13. Git Status Parsing

**Location**: `src/participants/gitParticipant.ts:58-76`

```typescript
const statusLines = stdout.split("\n").filter((line) => line.trim());
for (const line of statusLines) {
    const parts = line.trim().split(/\s+/);
    const status = parts[0];
    const file = parts.slice(1).join(" ");

    if (status === "M") modified.push(file);
    else if (status === "A") added.push(file);
    else if (status === "D") deleted.push(file);
    else if (status === "??") untracked.push(file);
}
```

### Quirks

1. **Whitespace Splitting**: Uses regex `/\s+/` to split lines
2. **Limited Status Codes**: Only recognizes `M`, `A`, `D`, `??`
3. **Ignores Other Codes**: `R` (renamed), `C` (copied), `U` (unmerged) ignored

### Git Status Codes

| Code | Meaning | Recognized |
|------|---------|-----------|
| `M` | Modified | ✅ Yes |
| `A` | Added | ✅ Yes |
| `D` | Deleted | ✅ Yes |
| `??` | Untracked | ✅ Yes |
| `R` | Renamed | ❌ No |
| `C` | Copied | ❌ No |
| `U` | Unmerged | ❌ No |
| `!` | Ignored | ❌ No |

### Edge Cases

```bash
# Git output:
R  old-name.txt -> new-name.txt   # Ignored (renamed)
U  conflict.txt                    # Ignored (unmerged)
M  "file with spaces.txt"          # May parse incorrectly
```

### Impact

- Renamed files not reported to user
- Merge conflicts not visible in git participant
- Files with special characters may parse incorrectly

### Recommendation

Use `git status --porcelain=v1` for consistent parsing:
```typescript
const result = await exec("git status --porcelain=v1");
// Porcelain format: XY PATH
// X = index status, Y = working tree status
```

---

## 14. Conversation Context Message Limit

**Location**: `src/chatView.ts:533-540`

```typescript
const relevantMessages = this.messages
    .filter((message) =>
        message.role === "user" ||
        message.role === "assistant" ||
        message.role === "tool"
    )
    .slice(-12);  // Last 12 messages only
```

### Hardcoded Limits

1. **12 Message Limit**: Only last 12 messages included in context
2. **Role Filtering**: Only `user`, `assistant`, and `tool` roles
3. **Excludes System/Error**: System and error messages not in context

### Edge Cases

```
Conversation with 20 messages:
- 3 system messages
- 15 user/assistant messages
- 2 error messages

Context includes:
- Last 12 of the 15 user/assistant messages
- 0 system messages
- 0 error messages
- First 3 user/assistant messages lost
```

### Impact

1. **Context Loss**: Long conversations lose early context
2. **No Configuration**: Cannot adjust message limit
3. **Token vs Message**: Limits messages, not tokens (12 short messages < 12 long messages)

### Recommendation

Use token-based context limit:
```typescript
const MAX_CONTEXT_TOKENS = 4000;
let tokenCount = 0;
const relevantMessages = [];

for (const message of this.messages.reverse()) {
    const messageTokens = estimateTokens(message.content);
    if (tokenCount + messageTokens > MAX_CONTEXT_TOKENS) break;

    relevantMessages.unshift(message);
    tokenCount += messageTokens;
}
```

---

## 15. MCP Server Definition Provider

**Location**: `src/extension.ts:2337-2376`

```typescript
if ("lm" in vscode &&
    typeof vscode.lm?.registerMcpServerDefinitionProvider === "function") {
    // Register MCP provider
}
```

### Quirks

1. **Feature Detection**: Uses runtime feature detection with optional chaining
2. **Silent Fallback**: If API unavailable, provider not registered (no error)
3. **Version Dependent**: Requires specific VS Code API version

### Edge Cases

1. **Older VS Code**: Feature silently unavailable in older versions
2. **No User Feedback**: User doesn't know MCP features are disabled
3. **API Changes**: If VS Code changes API, silent failure

### VS Code Version Compatibility

| VS Code Version | MCP Support |
|----------------|-------------|
| < 1.85 | ❌ No (silent fail) |
| >= 1.85 | ✅ Yes |
| Unknown | ❓ Silent fail |

### Recommendation

Add explicit version check and user notification:
```typescript
const minVersion = '1.85.0';
if (!checkVSCodeVersion(minVersion)) {
    vscode.window.showWarningMessage(
        `MCP features require VS Code ${minVersion} or higher. ` +
        `Current version: ${vscode.version}`
    );
}
```

---

## 16. Config File Selection Priority

**Location**: `src/vtcodeConfig.ts:337-363`

### Priority Algorithm

```typescript
// 1. If preferred URI matches one of the found files
if (preferredUri && foundFiles.some(f => f.toString() === preferredUri.toString())) {
    return preferredUri;
}

// 2. If only one file found
if (foundFiles.length === 1) {
    return foundFiles[0];
}

// 3. Check for workspace root vtcode.toml
for (const file of foundFiles) {
    if (isAtWorkspaceRoot(file)) {
        return file;
    }
}

// 4. Return shortest path
foundFiles.sort((a, b) => a.fsPath.length - b.fsPath.length);
return foundFiles[0];
```

### Priority Order

1. **Preferred URI** (explicitly selected by user)
2. **Single File** (only one config found)
3. **Workspace Root** (config at workspace root)
4. **Shortest Path** (fallback heuristic)

### Edge Cases

```
Workspace structure:
/workspace/vtcode.toml          (20 chars)
/workspace/project/vtcode.toml  (30 chars)
/workspace/sub/vtcode.toml      (25 chars)

Priority:
1. Workspace root: /workspace/vtcode.toml ✅
   (Even though not shortest)

Without root config:
1. Shortest path: /workspace/sub/vtcode.toml ✅
   (25 < 30 chars)
```

### Quirk: Shortest Path Heuristic

The "shortest path" heuristic may not always select the most relevant config:

```
/workspace/a/vtcode.toml         # 25 chars
/workspace/backend/vtcode.toml   # 30 chars

User working in /workspace/backend/
Selected config: /workspace/a/vtcode.toml ❌
(Shortest path, but wrong directory)
```

### Recommendation

Use "closest ancestor" instead of "shortest path":
```typescript
// Find config in closest ancestor directory
const currentDir = vscode.window.activeTextEditor?.document.uri.fsPath;
if (currentDir) {
    return findClosestAncestor(currentDir, foundFiles);
}
```

---

## 17. Human-in-the-Loop Setting Regex

**Location**: `src/vtcodeConfig.ts:183`

```typescript
const match = text.match(/^(\s*human_in_the_loop\s*=\s*)(true|false)/m);
```

### Quirks

1. **Multiline Mode**: Uses `/m` flag (multiline)
2. **Anchored to Line Start**: `^` requires start of line
3. **Exact Spacing**: Flexible whitespace around `=`
4. **Boolean Only**: Must be exactly `true` or `false`

### Edge Cases

```toml
# ✅ Matches
human_in_the_loop = true
human_in_the_loop=false
  human_in_the_loop  =  true

# ❌ Doesn't match
human_in_the_loop = true  # with comment
human_in_the_loop = 1
human_in_the_loop = "true"
# human_in_the_loop = true
```

### Impact

Inline comments prevent matching:
```toml
human_in_the_loop = true  # Enable safety
```
This line won't match, and toggle command will add duplicate setting instead of updating existing one.

### Recommendation

Handle inline comments:
```typescript
const match = text.match(/^(\s*human_in_the_loop\s*=\s*)(true|false)(\s*#.*)?$/m);
```

---

## 18. Provider Name Normalization

**Location**: `src/vtcodeConfig.ts:456,508`

```typescript
const normalizedProvider = providerName.trim().toLowerCase();
```

### Quirks

1. **Case-Insensitive**: All provider names converted to lowercase
2. **Whitespace Trimmed**: Leading/trailing whitespace removed
3. **No Validation**: Any string accepted after normalization

### Edge Cases

| Input | Normalized | Matches |
|-------|-----------|---------|
| `"OpenAI"` | `"openai"` | ✅ `openai` |
| `"  anthropic  "` | `"anthropic"` | ✅ `anthropic` |
| `"ANTHROPIC"` | `"anthropic"` | ✅ `anthropic` |
| `"open ai"` | `"open ai"` | ❌ (space kept) |
| `"OpenAI "` | `"openai"` | ✅ `openai` |

### Impact

```toml
# All equivalent:
provider = "openai"
provider = "OpenAI"
provider = "OPENAI"
provider = "  openai  "

# Not equivalent:
provider = "open ai"  # Space not trimmed from middle
```

---

## 19. Tool Approval Detail Length

**Location**: `src/chatView.ts:361-362`

```typescript
let detail = JSON.stringify(invocation.arguments, null, 2);
if (detail.length > 1200) {
    detail = detail.slice(0, 1200) + '...';
}
```

### Quirks

1. **1200 Character Limit**: Hardcoded for modal display
2. **Simple Truncation**: No smart JSON-aware truncation
3. **Marker Included**: `...` counts toward limit (should be `1197 + '...'` for exact 1200)

### Edge Cases

```typescript
// Large tool arguments
{
    "file": "/very/long/path/to/file.txt",
    "content": "Lorem ipsum dolor sit amet..." // 2000 chars
}

// User sees truncated JSON:
{
    "file": "/very/long/path/to/file.txt",
    "content": "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua ut enim ad minim veniam quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur excepteur sint occaecat cupidatat non proident sunt in culpa qui officia deserunt mollit anim id est laborum sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium totam rem aperiam eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt neque porro quisquam est qui dolorem ipsum quia dolor sit amet consectetur adipisci velit sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem ut enim ad minima veniam quis nostrum exercitationem ullam corporis suscipit laboriosam nisi ut aliquid ex ea commodi consequatur quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur vel illum qui dolorem eum fugiat quo voluptas nulla pariatur at vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis...
```

### Impact

User cannot see full tool arguments, making informed approval decisions difficult.

### Recommendation

Implement smart truncation:
```typescript
function truncateToolDetail(args: any, maxLength: number): string {
    const full = JSON.stringify(args, null, 2);
    if (full.length <= maxLength) return full;

    // Try truncating large values
    const truncated = JSON.stringify(args, (key, value) => {
        if (typeof value === 'string' && value.length > 100) {
            return value.slice(0, 100) + '... [truncated]';
        }
        return value;
    }, 2);

    if (truncated.length <= maxLength) return truncated;

    // Last resort: simple truncation
    return full.slice(0, maxLength - 20) + '\n... [truncated]';
}
```

---

## 20. CLI Detection Timeout

**Location**: `src/extension.ts:151`

```typescript
const CLI_DETECTION_TIMEOUT_MS = 4000; // 4 seconds
```

### Quirks

1. **Hardcoded**: No configuration option
2. **Single Timeout**: Same timeout for all detection attempts
3. **No Retry**: Failed detection = CLI unavailable

### Edge Cases

```
Slow system scenarios:
- Network-mounted home directory: >4s
- Antivirus scanning executables: >4s
- Cold start after reboot: >4s
- WSL file system delays: >4s

Result: CLI marked unavailable even if installed
```

### Impact

Users on slower systems or with network latency may experience false negatives.

### Recommendation

```typescript
// Make configurable
const config = vscode.workspace.getConfiguration('vtcode');
const timeout = config.get('cliDetectionTimeout', 4000);

// Add retry with exponential backoff
async function detectCLIWithRetry(maxAttempts = 3): Promise<boolean> {
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
        const timeout = CLI_DETECTION_TIMEOUT_MS * Math.pow(2, attempt);
        const detected = await detectCLI(timeout);
        if (detected) return true;
    }
    return false;
}
```

---

## 21. Context7 Integration Limits

**Location**: `src/context7Integration.ts:64-66`

```typescript
maxTokens: config?.maxTokens ?? 5000,
cacheResults: config?.cacheResults ?? true,
cacheTTLSeconds: config?.cacheTTLSeconds ?? 3600,
```

### Default Limits

1. **5000 Token Limit**: Default maximum tokens
2. **1 Hour Cache TTL**: Results cached for 3600 seconds
3. **Cache Enabled by Default**: `cacheResults: true`

### Edge Cases

1. **Token Counting Mismatch**: Internal token counting may not match LLM tokenization
2. **Cache Invalidation**: No automatic invalidation when files change
3. **Memory Usage**: Large cache may consume significant memory

### Token Counting Quirk

The extension's token estimation may differ from actual LLM tokenization:

```typescript
// Extension estimates tokens (approximate)
function estimateTokens(text: string): number {
    return Math.ceil(text.length / 4); // Rough estimate
}

// Actual LLM tokenization may differ
// "Hello world" → Extension: 3 tokens, GPT-4: 2 tokens
```

### Impact

- Context may be truncated unexpectedly
- Cache may serve stale results for modified files
- Memory usage not bounded

### Recommendation

```typescript
// Use actual tokenizer
import { encode } from 'gpt-tokenizer';

function countTokens(text: string, model: string): number {
    return encode(text).length;
}

// Implement cache invalidation
class SmartCache {
    onFileChange(uri: vscode.Uri) {
        this.invalidateCache(uri);
    }
}
```

---

## 22. Selection vs Visible Range Fallback

**Location**: `src/extension.ts:2818-2840`

### Context Selection Priority

```typescript
// 1. Explicit selection (if not empty)
if (!editor.selection.isEmpty) {
    return editor.document.getText(editor.selection);
}

// 2. Visible ranges
if (editor.visibleRanges.length > 0) {
    const nonEmptyRanges = editor.visibleRanges.filter(r => !r.isEmpty);
    // Combine visible ranges
}

// 3. Full document if ≤400 lines
if (editor.document.lineCount <= MAX_FULL_DOCUMENT_CONTEXT_LINES) {
    return editor.document.getText();
}

// 4. 80-line window around cursor
const cursorLine = editor.selection.active.line;
const start = Math.max(0, cursorLine - 40);
const end = Math.min(editor.document.lineCount - 1, cursorLine + 40);
```

### Priority Order

1. **Selection** (if not empty)
2. **Visible Ranges** (if any non-empty)
3. **Full Document** (if ≤400 lines)
4. **Cursor Window** (80 lines: ±40 from cursor)

### Edge Cases

```
Scenario 1: Large file with selection
- File: 1000 lines
- Selection: Lines 500-510
- Result: Lines 500-510 (selection wins)

Scenario 2: Large file, no selection, visible lines 100-150
- File: 1000 lines
- Selection: Empty
- Visible: Lines 100-150
- Result: Lines 100-150 (visible range)

Scenario 3: Large file, no selection, no visible ranges
- File: 1000 lines
- Selection: Empty
- Visible: None
- Cursor: Line 500
- Result: Lines 460-540 (cursor window)

Scenario 4: Small file
- File: 300 lines
- Selection: Empty
- Result: Full 300 lines
```

### Quirk: Empty Visible Ranges Filtered

```typescript
const nonEmptyRanges = editor.visibleRanges.filter(r => !r.isEmpty);
```

Folded code regions may create empty visible ranges, which are filtered out.

### Impact

- User may not understand which content is being used as context
- Folded regions always excluded
- Large files automatically windowed, potentially missing important context

### Recommendation

Add context indicator to UI:
```typescript
function getContextDescription(editor: vscode.TextEditor): string {
    if (!editor.selection.isEmpty) {
        return `Selection (${editor.selection.end.line - editor.selection.start.line + 1} lines)`;
    }
    if (editor.visibleRanges.some(r => !r.isEmpty)) {
        return 'Visible content';
    }
    if (editor.document.lineCount <= 400) {
        return 'Full document';
    }
    return `Context window (80 lines around cursor)`;
}
```

---

## Summary of Critical Quirks

### High Priority Issues

| # | Issue | Impact | Recommended Action |
|---|-------|--------|-------------------|
| 3.3 | Custom TOML parser with known limitations | MCP config may fail to parse | Replace with proper TOML library |
| 4 | Multiple hardcoded file/line limits | Silent context exclusion | Make configurable + warn users |
| 9 | No graceful degradation without workspace trust | Features completely unavailable | Implement read-only mode |
| 14 | Fixed 12-message context limit | Context loss in long conversations | Use token-based limit |
| 16 | Shortest path config selection | Wrong config in monorepos | Use closest ancestor algorithm |

### Medium Priority Issues

| # | Issue | Impact | Recommended Action |
|---|-------|--------|-------------------|
| 2 | Mention regex doesn't support hyphens | Limits participant naming | Document limitation |
| 3.1 | Hardcoded MCP timeouts | Slow providers fail | Make configurable |
| 5 | Various truncation without indicators | User unaware of missing content | Add truncation indicators |
| 12 | Code participant excludes markdown | Can't analyze markdown with code | Reconsider exclusion |
| 13 | Git status only recognizes 4 codes | Missing renamed/merged files | Use porcelain format |

### Low Priority Issues

| # | Issue | Impact | Recommended Action |
|---|-------|--------|-------------------|
| 1 | Triple document selector pattern | Complexity | Document behavior |
| 7 | Four terminal tool aliases | Minor confusion | Document aliases |
| 10 | Hover regex inconsistencies | Edge cases | Standardize patterns |
| 17 | Regex doesn't handle inline comments | Toggle creates duplicates | Update regex |
| 21 | Token counting approximation | Estimation errors | Use real tokenizer |

---

## Testing Recommendations

### Test Categories

1. **Edge Case Tests**: Test all documented edge cases
2. **Limit Tests**: Verify behavior at and beyond limits
3. **Quirk Tests**: Ensure quirks work as documented
4. **Regression Tests**: Prevent quirks from being "fixed" and breaking compatibility

### Example Test Suite

```typescript
describe('Provider Quirks', () => {
    describe('Mention Regex', () => {
        it('should match alphanumeric mentions', () => {
            expect('@workspace123'.match(MENTION_REGEX)).toBeTruthy();
        });

        it('should not match hyphenated mentions', () => {
            expect('@workspace-special'.match(MENTION_REGEX)).toBeFalsy();
        });
    });

    describe('File Size Limits', () => {
        it('should exclude files > 1000 lines from active context', async () => {
            const largeFile = createMockDocument(1001);
            const context = await buildContext(largeFile);
            expect(context.includes('activeFileContent')).toBe(false);
        });

        it('should warn user when file excluded', async () => {
            const largeFile = createMockDocument(1001);
            await buildContext(largeFile);
            expect(warningShown).toBe(true);
        });
    });
});
```

---

## Configuration Recommendations

### Proposed vtcode.toml Additions

```toml
[extension]
# File and context limits
max_active_file_lines = 1000
max_full_document_lines = 400
active_editor_context_window = 80
max_visible_editor_contexts = 3
max_ide_context_chars = 6000
max_conversation_messages = 12

# Timeouts
cli_detection_timeout_ms = 4000
mcp_discovery_timeout_ms = 5000
mcp_execution_timeout_ms = 30000

# Truncation
tool_approval_detail_max_chars = 1200
conversation_context_max_chars = 2000
documentation_max_chars = 2000

# Participant limits
code_participant_max_lines = 50
terminal_participant_output_lines = 20
terminal_participant_history_commands = 5
workspace_participant_max_files = 100

# Behavior
enable_graceful_degradation_without_trust = false
warn_on_context_truncation = true
warn_on_file_exclusion = true
```

---

## Migration Guide

### For Extension Developers

When modifying code with documented quirks:

1. **Check This Document First**: Ensure you understand the quirk's purpose
2. **Update Documentation**: If changing quirk behavior, update this document
3. **Add Configuration**: Consider making hardcoded values configurable
4. **Maintain Compatibility**: Quirks may be relied upon by users/tools

### For Extension Users

When encountering unexpected behavior:

1. **Consult This Document**: Check if behavior is a documented quirk
2. **Check Limits**: Verify you're within documented limits
3. **Review Configuration**: Ensure settings are as expected
4. **Report Issues**: If behavior doesn't match documentation, report it

---

## Changelog

### Version 1.0 (November 13, 2025)

- Initial documentation of 22 major quirk categories
- Comprehensive edge case analysis
- Recommendations for future improvements
- Critical issues identified for Phase 3

---

## References

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Extension architecture overview
- [VSCode Extension API](https://code.visualstudio.com/api)
- [TOML Specification](https://toml.io/)
- [Git Status Porcelain Format](https://git-scm.com/docs/git-status#_porcelain_format_version_1)

---

**Document Maintained By**: VTCode Development Team
**Last Updated**: November 13, 2025
**Review Cycle**: Quarterly or with major releases
