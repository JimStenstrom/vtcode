# VT Code Tool Specifications

This document provides comprehensive specifications for all tools available to VTCode agents. Tool schemas follow Anthropic's best practices for agent tools.

**See Also:**
- **[Web Fetch Security](./web_fetch_security.md)** - Security configuration for the web_fetch tool
- **[Git Commands Reference](../user-guide/GIT_QUICK_REFERENCE.md)** - Git command usage guide
- **[Tool Development Guide](../development/tool-development.md)** - Guide for implementing custom tools

## Common Conventions

-   Arguments are unambiguous (e.g., `path`, `max_results`, `response_format`).
-   Default `response_format` is `"concise"`. Use `"detailed"` only when necessary.
-   Long-listing tools support pagination via `page` (1-based) and `per_page`.
-   Errors are actionable and include examples to retry with corrected inputs.

## Tools

-   grep_file

    -   Purpose: Unified code search. Modes: `exact` | `fuzzy` | `multi` | `similarity`.
    -   Key args: `pattern` (string), `path` (string, default "."), `max_results` (int), `mode` (string), `response_format` (string: concise|detailed).
    -   Multi-mode: `patterns: string[]`, `logic: 'AND'|'OR'`.
    -   Similarity-mode: `reference_file` (string), `content_type: 'structure'|'imports'|'functions'|'all'`.
    -   Returns: `matches` with file, line, text (concise: `[ { path, line_number, text } ]`) or raw rg JSON (detailed). Adds guidance when results hit caps.

-   list_files

    -   Purpose: File discovery. Modes: `list` | `recursive` | `find_name` | `find_content`.
    -   Key args: `path` (string), `max_items` (int), `page` (int), `per_page` (int), `include_hidden` (bool), `response_format` (string: concise|detailed).
    -   Mode args: `name_pattern` (string), `content_pattern` (string), `file_extensions` (string[]), `case_sensitive` (bool).
    -   Patterns are matched with `nucleo-matcher` fuzzy scoring over a corpus gathered via the
        `ignore` crate (respects `.gitignore`, global ignores, and hidden file rules).
    -   Returns: Paginated items with guidance (`message`) when more pages are available. Concise output omits low-signal fields.

-   read_file

    -   Purpose: Read a file with optional `max_bytes` to conserve tokens.
    -   Key args: `path` (string), `max_bytes` (int, optional).

-   write_file

    -   Purpose: Write content to a file.
    -   Key args: `path` (string), `content` (string), `mode` (string: overwrite|append|skip_if_exists).

-   edit_file

    -   Purpose: Replace specific text in a file.
    -   Key args: `path` (string), `old_str` (string), `new_str` (string).

-   run_terminal_cmd

    -   Purpose: Execute a program with arguments.
    -   Key args: `command` (string|string[]), `working_dir` (string), `timeout_secs` (int), `mode` (string: pty|terminal|streaming), `response_format`.
    -   Default mode is `pty` so output retains ANSI styling.

-   apply_patch

    -   Purpose: Apply unified diff patches to files.
    -   Key args: `patch` (string), `working_dir` (string), `dry_run` (bool).
    -   Supports multi-file patches with context matching.
    -   Returns: List of files modified with status (success/failure/skipped).

-   web_fetch

    -   Purpose: Fetch content from URLs with security controls.
    -   Key args: `url` (string), `method` (string: GET|POST|HEAD), `headers` (object), `body` (string).
    -   Security modes: `restricted` (default) | `open` | `custom`.
    -   See [Web Fetch Security](./web_fetch_security.md) for configuration details.

-   tree_sitter

    -   Purpose: Syntax-aware code analysis and querying.
    -   Key args: `path` (string), `query` (string), `language` (string).
    -   Supports structural queries for code navigation and refactoring.
    -   Returns: AST nodes matching the query pattern.

## Additional Tools

The following tools are available but may require specific configuration or MCP server integration:

-   **plan** - Generate execution plans for complex multi-step tasks
-   **git_status**, **git_diff**, **git_log** - Git operations (see [Git Commands](../user-guide/GIT_QUICK_REFERENCE.md))
-   **MCP tools** - Model Context Protocol servers can provide additional tools (filesystem, database access, etc.)

## Policy Constraints (scoped)

The workspace `.vtcode/tool-policy.json` may include constraints like:

```json
{
    "constraints": {
        "run_terminal_cmd": {
            "allowed_modes": ["pty", "terminal", "streaming"],
            "default_response_format": "concise"
        },
        "list_files": {
            "max_items_per_call": 500,
            "default_response_format": "concise"
        },
        "grep_file": {
            "max_results_per_call": 200,
            "default_response_format": "concise"
        },
        "read_file": { "max_bytes_per_read": 200000 }
    }
}
```

These are applied automatically by the ToolRegistry at runtime.

## Error Style

-   Include missing-field names, allowed values, and a concrete example.
-   Example: `Error: Missing 'name_pattern'. Example: list_files(path='.', mode='recursive', name_pattern='*.rs')`.

## Evaluation Tips

-   Use real tasks that chain tools (search → read → edit → write) and require multiple calls.
-   Track: success rate, tool call count, token usage, and errors.
-   Let agents iterate on error feedback to refine prompts and parameters.
