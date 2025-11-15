---
type: standard-operating-procedure
id: code-search
---

# Effective Code Search

## When to Use

When you need to find code, understand project structure, or locate specific patterns in the codebase.

## Tools Available

- **`grep_file`** - Search file contents (powered by ripgrep)
- **`list_files`** - List files and directories

## Tool Selection

```
What are you looking for?

├─ Specific code/text pattern?
│  └─ Use grep_file
│
├─ Files by name pattern?
│  └─ Use list_files with pattern
│
├─ Directory structure?
│  └─ Use list_files (no pattern)
│
└─ Don't know where to start?
   └─ Use list_files first, then grep_file
```

## 1. grep_file - Content Search

### When to Use

✅ **Use grep_file to find:**
- Function definitions
- Variable usages
- Import statements
- Error messages
- TODO comments
- Specific patterns in code

### Basic Usage

```json
{
  "tool": "grep_file",
  "parameters": {
    "pattern": "fn process_data",
    "path": "src/"
  }
}
```

### Pattern Types

#### Literal Text Search

```json
{
  "pattern": "DatabaseConnection",
  "path": "src/"
}
// Finds: "DatabaseConnection" anywhere in src/
```

#### Regex Patterns

```json
{
  "pattern": "fn \\w+_handler",
  "path": "src/"
}
// Finds: fn create_handler, fn update_handler, etc.
```

#### Case-Insensitive Search

```json
{
  "pattern": "(?i)error",
  "path": "src/"
}
// Finds: error, Error, ERROR, ErRoR
```

### Search Scoping

#### Search Entire Project

```json
{
  "pattern": "import React",
  "path": "."
}
// Searches all files from root
```

#### Search Specific Directory

```json
{
  "pattern": "async fn",
  "path": "src/api/"
}
// Only searches in src/api/ directory
```

#### Search Single File

```json
{
  "pattern": "struct User",
  "path": "src/models/user.rs"
}
// Only searches user.rs
```

#### Search by File Type

```json
{
  "pattern": "export default",
  "path": "src/",
  "file_pattern": "*.ts"
}
// Only searches TypeScript files
```

### Common Search Patterns

#### Finding Function Definitions

```
Rust:
pattern: "fn function_name"
pattern: "pub fn \\w+"  (all public functions)

TypeScript:
pattern: "function \\w+"
pattern: "const \\w+ = \\("  (arrow functions)

Python:
pattern: "def function_name"
```

#### Finding Imports/Uses

```
Rust:
pattern: "use crate::"
pattern: "use std::collections"

TypeScript:
pattern: "import.*from"
pattern: "import \\{ \\w+ \\}"

Python:
pattern: "import \\w+"
pattern: "from .* import"
```

#### Finding TODOs

```json
{
  "pattern": "TODO|FIXME|XXX|HACK",
  "path": "src/"
}
```

#### Finding Error Handling

```
Rust:
pattern: "Result<"
pattern: "\\.unwrap\\("
pattern: "panic!"

TypeScript:
pattern: "try \\{"
pattern: "catch \\("
pattern: "throw new"
```

#### Finding Tests

```
Rust:
pattern: "#\\[test\\]"
pattern: "#\\[tokio::test\\]"

TypeScript:
pattern: "describe\\("
pattern: "it\\("
pattern: "test\\("
```

### Performance Optimization

#### Use Specific Paths

```
❌ Slow (searches everything):
grep_file(pattern="fn foo", path=".")

✅ Fast (targeted search):
grep_file(pattern="fn foo", path="src/api/")
```

#### Use File Patterns

```
❌ Searches all files:
grep_file(pattern="interface", path="src/")

✅ Only searches TypeScript:
grep_file(
  pattern="interface",
  path="src/",
  file_pattern="*.ts"
)
```

#### Exclude Directories

```
✅ Skip node_modules, target, etc.:
grep_file(
  pattern="api_key",
  path=".",
  exclude_pattern="node_modules|target|.git"
)
```

### Getting Context

#### Show Surrounding Lines

```json
{
  "pattern": "fn critical_function",
  "path": "src/",
  "context_lines": 5
}
// Shows 5 lines before and after match
```

#### Find All Occurrences

```json
{
  "pattern": "deprecated_function",
  "path": "src/",
  "max_results": 100
}
// Shows up to 100 matches (default is usually 50)
```

## 2. list_files - File Discovery

### When to Use

✅ **Use list_files to:**
- Explore directory structure
- Find files by name
- Verify file existence
- Understand project layout

### Basic Usage

#### List Directory Contents

```json
{
  "tool": "list_files",
  "parameters": {
    "path": "src/"
  }
}
```

#### List with Pattern

```json
{
  "tool": "list_files",
  "parameters": {
    "path": "src/",
    "pattern": "*.rs"
  }
}
// Only shows Rust files
```

### Common Patterns

#### Find All Files of Type

```
Rust files:
pattern: "*.rs"

TypeScript files:
pattern: "*.ts"

Test files:
pattern: "*test.rs"
pattern: "*.spec.ts"

Config files:
pattern: "*.toml"
pattern: "*.json"
```

#### Find by Name Pattern

```json
{
  "path": "src/",
  "pattern": "*handler*"
}
// Finds: user_handler.rs, api_handler.ts, etc.
```

#### Recursive Listing

```json
{
  "path": "src/",
  "recursive": true
}
// Lists all files in src/ and subdirectories
```

## Search Strategies

### Strategy 1: Exploration

When you don't know the codebase:

```
1. List root:
   list_files(".")

2. Explore main directories:
   list_files("src/")

3. Find entry points:
   grep_file(pattern="fn main", path="src/")
   grep_file(pattern="mod \\w+", path="src/lib.rs")

4. Read key files
```

### Strategy 2: Feature Location

When finding where a feature is implemented:

```
1. Search for feature name:
   grep_file(pattern="user_authentication", path="src/")

2. Find related files:
   list_files(path="src/auth/")

3. Search for function names:
   grep_file(pattern="fn login", path="src/auth/")

4. Read implementation
```

### Strategy 3: Bug Tracking

When tracking down a bug:

```
1. Search for error message:
   grep_file(pattern="Connection refused", path="src/")

2. Search for related functions:
   grep_file(pattern="fn connect", path="src/")

3. Find all error handling:
   grep_file(pattern="Result<", path="src/database/")

4. Read suspicious files
```

### Strategy 4: Refactoring

When planning refactoring:

```
1. Find all usages:
   grep_file(pattern="OldStructName", path="src/")

2. Find all imports:
   grep_file(pattern="use.*OldStructName", path="src/")

3. Find all tests:
   grep_file(pattern="OldStructName", path="tests/")

4. Plan refactoring based on usage
```

## Common Workflows

### Understanding a New Codebase

```
1. Project structure:
   list_files(".")  # See root files
   list_files("src/")  # See source structure

2. Entry points:
   grep_file(pattern="fn main", path="src/")
   grep_file(pattern="pub mod", path="src/lib.rs")

3. Dependencies:
   read_file("Cargo.toml")  # Or package.json

4. Key modules:
   list_files("src/", pattern="mod.rs")
```

### Finding Implementation of Feature

```
1. Search by feature name:
   grep_file(pattern="feature_name", path="src/")

2. Search by function names:
   grep_file(pattern="fn process_feature", path="src/")

3. Find tests:
   grep_file(pattern="test.*feature", path="tests/")

4. Read relevant files
```

### Finding All TODOs

```
1. Search for TODO markers:
   grep_file(pattern="TODO|FIXME", path="src/")

2. Group by priority:
   grep_file(pattern="TODO.*HIGH", path="src/")

3. By module:
   grep_file(pattern="TODO", path="src/api/")
```

### Security Audit

```
1. Find sensitive patterns:
   grep_file(pattern="password|secret|key", path="src/")

2. Find unsafe code (Rust):
   grep_file(pattern="unsafe \\{", path="src/")

3. Find potential issues:
   grep_file(pattern="\\.unwrap\\(|panic!", path="src/")
```

## Error Handling

### "No matches found"

```
Solutions:
1. Check pattern syntax
2. Verify path exists
3. Try broader pattern
4. Use case-insensitive: (?i)pattern
```

### "Too many matches"

```
Solutions:
1. Narrow search path
2. Add file_pattern filter
3. Use more specific pattern
4. Increase max_results
```

### "Invalid regex"

```
Solutions:
1. Check regex syntax
2. Escape special characters: \. \( \)
3. Test pattern incrementally
```

## Anti-Patterns

❌ **Searching too broadly**
```
grep_file(pattern="x", path=".")  # Too generic!
```

❌ **Not using file patterns**
```
Searching for TypeScript patterns in all files
```

❌ **Re-searching same pattern**
```
grep_file same pattern multiple times
Better: Save results and analyze
```

❌ **Using grep when list_files would work**
```
Finding .rs files? Use list_files(pattern="*.rs")
Not grep_file(pattern=".*\\.rs")
```

## Performance Tips

1. **Narrow the search path** - Search specific directories
2. **Use file patterns** - Limit to relevant file types
3. **Exclude large directories** - Skip node_modules, target
4. **Use specific patterns** - Avoid overly broad regex
5. **Combine with list_files** - Find files first, then grep

## Quick Reference

| Task | Tool | Example |
|------|------|---------|
| Find function | `grep_file` | `pattern: "fn my_function"` |
| Find imports | `grep_file` | `pattern: "use crate::"` |
| Find files by name | `list_files` | `pattern: "*test.rs"` |
| Project structure | `list_files` | `path: "src/", recursive: true` |
| TODOs | `grep_file` | `pattern: "TODO\\|FIXME"` |
| All Rust files | `list_files` | `pattern: "*.rs"` |
| Regex search | `grep_file` | `pattern: "fn \\w+_handler"` |

## Summary

1. **grep_file for content** - Search inside files
2. **list_files for structure** - Find files by name/pattern
3. **Be specific** - Narrow paths and use file patterns
4. **Use regex wisely** - Balance power with precision
5. **Combine strategies** - list_files + grep_file + read_file
6. **Optimize searches** - Start narrow, expand if needed
