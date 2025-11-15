---
type: standard-operating-procedure
id: file-editing
---

# File Editing Best Practices

## When to Use

When modifying, creating, or deleting files in the codebase.

## Tools Available

- **`edit_file`** - Modify existing files (PREFERRED)
- **`create_file`** - Create new files
- **`write_file`** - Overwrite entire file contents
- **`delete_file`** - Delete files

## Tool Selection Decision Tree

```
Need to modify a file?
├─ File exists?
│  ├─ YES → Use edit_file (precise changes)
│  └─ NO  → Use create_file
│
└─ Complete rewrite needed?
   ├─ File is small (< 50 lines) → write_file acceptable
   └─ File is large → edit_file with multiple edits
```

## 1. edit_file - Primary Editing Tool

### When to Use

✅ **Always prefer edit_file for existing files:**
- Modifying functions
- Adding imports
- Changing configuration
- Fixing bugs
- Refactoring

### Basic Usage

```json
{
  "tool": "edit_file",
  "parameters": {
    "path": "/path/to/file.rs",
    "old_text": "fn old_function() {\n    // old code\n}",
    "new_text": "fn new_function() {\n    // new code\n}"
  }
}
```

### Critical Rules

#### Rule 1: Always Read Before Edit

```
❌ NEVER edit without reading:
edit_file(...)  # Error! You don't know what's in the file

✅ ALWAYS read first:
read_file("config.rs")  # See current contents
edit_file(...)          # Make precise changes
```

#### Rule 2: Match Exact Text

```
❌ Wrong (approximate matching):
old_text: "function foo"  # Missing exact whitespace

✅ Correct (exact copy from read_file):
old_text: "fn foo() {\n    let x = 1;\n}"
```

**Why:** edit_file requires **character-perfect** matching including:
- Exact indentation (spaces AND tabs)
- Newlines (`\n`)
- All whitespace
- Capitalization

#### Rule 3: Preserve Line Numbers Format

When you read a file, output shows:
```
   45→    fn example() {
   46→        let x = 1;
   47→    }
```

The `45→` is the line number prefix. **DO NOT include this in old_text:**

```
❌ Wrong:
old_text: "   45→    fn example()"

✅ Correct:
old_text: "    fn example()"
```

#### Rule 4: Include Enough Context

Make `old_text` unique in the file:

```
❌ Too little context (ambiguous):
old_text: "let x = 1;"  # Might appear multiple times

✅ Enough context (unique):
old_text: "fn foo() {\n    let x = 1;\n    let y = 2;\n}"
```

#### Rule 5: Preserve Indentation Style

```
File uses spaces:
old_text: "    fn foo()"  # 4 spaces
new_text: "    fn bar()"  # 4 spaces - match it!

File uses tabs:
old_text: "\tfn foo()"    # Tab character
new_text: "\tfn bar()"    # Tab character - match it!
```

### Common edit_file Patterns

#### Adding a function

```json
{
  "old_text": "impl MyStruct {\n    fn existing() {}\n}",
  "new_text": "impl MyStruct {\n    fn existing() {}\n\n    fn new_function() {\n        // implementation\n    }\n}"
}
```

#### Modifying imports

```json
{
  "old_text": "use std::collections::HashMap;",
  "new_text": "use std::collections::{HashMap, HashSet};"
}
```

#### Changing a function signature

```json
{
  "old_text": "pub fn process(data: String) -> Result<()> {",
  "new_text": "pub fn process(data: &str) -> Result<Vec<u8>> {"
}
```

## 2. create_file - For New Files

### When to Use

✅ **Use create_file when:**
- File doesn't exist
- Creating new modules
- Adding new tests
- Creating new configuration files

### Basic Usage

```json
{
  "tool": "create_file",
  "parameters": {
    "path": "/path/to/new_file.rs",
    "content": "// File contents here\n"
  }
}
```

### Best Practices

#### Check if file exists first

```
✅ Proper workflow:
1. list_files("src/") or read_file("new.rs")  # Check existence
2. If file doesn't exist → create_file
3. If file exists → use edit_file instead

❌ Wrong:
create_file(...)  # Might overwrite existing file!
```

#### Use appropriate file extensions

```
✅ Correct extensions:
- Rust: .rs
- TypeScript: .ts
- JavaScript: .js
- Markdown: .md
- TOML: .toml

❌ Wrong:
"config" (no extension)
"script" (ambiguous)
```

#### Include file headers

```rust
✅ Good new file:
//! Module documentation
//!
//! This module handles user authentication.

use std::collections::HashMap;

pub struct AuthManager {
    // ...
}
```

## 3. write_file - Complete Rewrites

### When to Use

⚠️ **Use sparingly:**
- Small config files (< 50 lines)
- Complete rewrites of tiny files
- Generated content

### When NOT to Use

❌ **Never use write_file for:**
- Large files (use edit_file)
- Partial modifications (use edit_file)
- Files you haven't read (use edit_file)

### Basic Usage

```json
{
  "tool": "write_file",
  "parameters": {
    "path": "/path/to/file.toml",
    "content": "[package]\nname = \"example\"\nversion = \"0.1.0\"\n"
  }
}
```

### Comparison: edit_file vs write_file

```
Scenario: Change one line in 200-line file

❌ write_file approach:
- Rewrite all 200 lines
- High risk of errors
- Loses formatting

✅ edit_file approach:
- Change only 1 line
- Precise and safe
- Preserves formatting
```

## 4. delete_file - Removing Files

### When to Use

✅ **Use delete_file for:**
- Removing obsolete files
- Cleaning up test artifacts
- Deleting empty files

### Basic Usage

```json
{
  "tool": "delete_file",
  "parameters": {
    "path": "/path/to/obsolete.rs"
  }
}
```

### Safety Checks

```
✅ Before deleting:
1. Confirm file is not imported elsewhere
   - grep_file(pattern="use.*obsolete")

2. Check if referenced in config
   - grep_file(pattern="obsolete", path="Cargo.toml")

3. Verify it's safe to remove
   - Ask user if uncertain
```

### What NOT to Delete

❌ **Never delete without asking:**
- .git directory or files
- node_modules/package-lock.json (use package manager)
- Cargo.lock (use cargo clean)
- User data files
- Configuration files (unless obsolete)

## Error Prevention

### Common edit_file Errors

#### Error: "old_text not found"

```
Cause: Mismatch in exact text

Solutions:
1. Re-read the file to get exact text
2. Check indentation (spaces vs tabs)
3. Check for hidden characters
4. Expand context to make match unique
```

#### Error: "File has been modified"

```
Cause: File changed since you read it

Solutions:
1. Re-read the file
2. Check if linter auto-formatted it
3. Retry edit with current contents
```

### Common create_file Errors

#### Error: "File already exists"

```
Cause: File was already created

Solutions:
1. Use edit_file instead
2. Check with list_files first
3. Consider if you want to overwrite
```

#### Error: "Directory doesn't exist"

```
Cause: Parent directory missing

Solutions:
1. Create parent directories first
2. Verify path structure with list_files
```

## Workflow Examples

### Fixing a Bug

```
1. Read the buggy file:
   read_file("src/auth.rs")

2. Locate the bug (lines 45-47)

3. Edit precisely:
   edit_file(
     path="src/auth.rs",
     old_text="    if password == stored {\n        true\n    }",
     new_text="    if password == stored_hash {\n        verify_hash(password, stored)\n    }"
   )

4. Read again to verify (optional)
```

### Adding a New Feature

```
1. Create new module:
   create_file(
     path="src/features/new_feature.rs",
     content="// New feature implementation\n"
   )

2. Add to module tree:
   read_file("src/features/mod.rs")
   edit_file(
     path="src/features/mod.rs",
     old_text="pub mod existing;",
     new_text="pub mod existing;\npub mod new_feature;"
   )

3. Update main imports:
   edit_file(...)
```

### Refactoring

```
1. Read all affected files:
   - read_file("src/old_module.rs")
   - read_file("src/lib.rs")

2. Create new structure:
   - create_file("src/refactored/mod.rs")

3. Move code with edit_file:
   - edit_file(remove from old location)
   - edit_file(add to new location)

4. Update imports everywhere:
   - grep_file(pattern="use.*old_module")
   - edit_file(for each match)
```

## Quick Reference

| Task | Tool | Must Read First? |
|------|------|------------------|
| Modify existing file | `edit_file` | ✅ YES |
| Create new file | `create_file` | ❌ No (but check existence) |
| Complete small rewrite | `write_file` | ⚠️ Recommended |
| Delete file | `delete_file` | ⚠️ Check references |

## Anti-Patterns

❌ **Using write_file for large files**
```
Don't rewrite 500 lines to change 5
```

❌ **Editing without reading**
```
Always read first to understand context
```

❌ **Approximate text matching**
```
old_text must be EXACTLY character-perfect
```

❌ **Creating files that already exist**
```
Check with list_files or read_file first
```

❌ **Deleting files carelessly**
```
Verify no imports/references exist
```

## Summary

1. **edit_file is king** - Use for all modifications to existing files
2. **Always read before edit** - Understand context and get exact text
3. **Match text exactly** - Character-perfect, including whitespace
4. **create_file for new files** - Check existence first
5. **write_file sparingly** - Only for small complete rewrites
6. **delete_file carefully** - Check for references first
7. **Preserve indentation** - Match the file's style exactly
