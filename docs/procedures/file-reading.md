---
type: standard-operating-procedure
id: file-reading
---

# Effective File Reading

## When to Use

When you need to read file contents to understand code, configuration, or documentation.

## Tools

- **`read_file`** - Primary tool for reading file contents
- **`grep_file`** - For searching within files (see code-search SOP)
- **`list_files`** - For discovering what files exist

## Basic Usage

### Reading a Single File

```json
{
  "tool": "read_file",
  "parameters": {
    "path": "/path/to/file.rs"
  }
}
```

### Reading with Line Limits

For large files, read in chunks:

```json
{
  "tool": "read_file",
  "parameters": {
    "path": "/path/to/large_file.rs",
    "start_line": 1,
    "end_line": 100
  }
}
```

## Best Practices

### 1. Read Before Edit

**Always** read a file before editing it:

```
✅ Correct workflow:
1. read_file("config.toml")
2. Understand structure
3. edit_file() with precise changes

❌ Wrong workflow:
1. edit_file() blindly
2. Cause syntax errors or data loss
```

**Why:** You need to see:
- Current file structure
- Indentation style (tabs vs spaces)
- Surrounding context
- Existing patterns to match

### 2. Avoid Re-reading Files

Once you've read a file in a conversation turn, **you have the full context**. Don't read it again unless:

- User modified it externally
- Another tool changed it
- Session was interrupted

```
❌ Inefficient:
read_file("main.rs")  # Line 1-500
read_file("main.rs")  # Reading again - wasteful!

✅ Efficient:
read_file("main.rs")  # Read once
# Use the content in memory for analysis
```

### 3. Targeted Reading for Large Files

For files > 500 lines, read strategically:

```
✅ Strategy 1: Read relevant section
read_file("large.rs", start_line=450, end_line=550)

✅ Strategy 2: Use grep_file first
grep_file(pattern="function_name", path="large.rs")
# Grep shows line numbers
read_file("large.rs", start_line=<grep_result - 20>, end_line=<grep_result + 20>)

❌ Don't read entire 5000-line file if you only need one function
```

### 4. Read Multiple Related Files in Parallel

When you need context from multiple files, read them together:

```
✅ Parallel reads (efficient):
- read_file("src/main.rs")
- read_file("src/config.rs")
- read_file("Cargo.toml")

❌ Sequential with delays:
read_file("src/main.rs")
# analyze
read_file("src/config.rs")  # Could have done earlier
```

### 5. File Types to Handle Specially

#### Binary Files
```
❌ Don't read binary files:
- .pdf, .png, .jpg, .zip, .tar.gz
- Compiled binaries (.exe, .so, .dylib)

✅ For these, just acknowledge existence via list_files
```

#### Generated Files
```
⚠️ Read with caution:
- package-lock.json (thousands of lines)
- Cargo.lock (can be very large)
- .min.js files (minified, unreadable)

✅ Better: Use grep_file to find specific entries
```

#### Configuration Files
```
✅ Always read config files completely:
- Cargo.toml, package.json, tsconfig.json
- .gitignore, .env.example
- vtcode.toml

Reason: Configs are usually small and context-critical
```

## Common Workflows

### Understanding a New Codebase

```
1. Read project root files:
   - README.md
   - Cargo.toml or package.json
   - .gitignore

2. List directory structure:
   - list_files("src/")

3. Read entry points:
   - main.rs, lib.rs, index.ts

4. Read specific modules as needed
```

### Debugging an Issue

```
1. Read the file with the bug:
   - read_file("src/buggy_module.rs")

2. Read related files:
   - Dependencies/imports
   - Test files
   - Configuration

3. Search for similar patterns:
   - grep_file(pattern="similar_function")
```

### Implementing a Feature

```
1. Read existing similar feature:
   - Find via grep_file
   - Read implementation

2. Read interfaces/traits:
   - Understand contracts

3. Read tests:
   - Understand expected behavior

4. Only then write new code
```

## Error Handling

### File Not Found

```json
Error: "File not found: /path/to/file.rs"

Solutions:
1. Use list_files to verify path
2. Check for typos in path
3. Confirm file exists in workspace
```

### Permission Denied

```json
Error: "Permission denied"

Solutions:
1. File may be system-protected
2. Check file ownership
3. Some files (in .git/) are intentionally restricted
```

### File Too Large

```json
Error: "File exceeds maximum size"

Solutions:
1. Read in chunks (start_line/end_line)
2. Use grep_file to find relevant sections
3. Consider if you really need entire file
```

## Anti-Patterns

❌ **Reading files you don't need**
```
# Don't read every file in a directory
for file in all_files:
    read_file(file)  # Wasteful!
```

❌ **Reading without purpose**
```
read_file("random.rs")  # Why? What are you looking for?
```

❌ **Ignoring file extensions**
```
read_file("image.png")  # Binary file, won't be useful
```

❌ **Not using line limits for large files**
```
read_file("10000_lines.rs")  # Read all? No!
read_file("10000_lines.rs", start_line=500, end_line=600)  # Better
```

## Performance Tips

1. **Batch related reads** - Read all needed files at start
2. **Cache in memory** - Don't re-read files in same conversation
3. **Use grep first** - For large files, grep narrows search
4. **Read incrementally** - For exploration, read sections as needed

## Quick Reference

| Scenario | Approach |
|----------|----------|
| Small file (< 200 lines) | Read entire file |
| Large file (> 500 lines) | Read in chunks or grep first |
| Unknown file size | Use list_files to check, then decide |
| Multiple related files | Read in parallel |
| Binary file | Don't read, just note existence |
| Before editing | **Always** read first |
| Already read in turn | Don't read again |

## Summary

1. **Always read before editing** - No exceptions
2. **Don't re-read unnecessarily** - You have the content
3. **Be strategic with large files** - Use chunks or grep
4. **Read with purpose** - Know what you're looking for
5. **Parallel reads** - Get all context upfront
