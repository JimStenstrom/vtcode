# Archived Documentation

This directory contains historical documentation from VT Code development sessions. These documents record implementation details, completion summaries, and session notes that are no longer actively maintained but preserved for historical reference.

## Archive Organization

### Implementation Sessions
- **`implementation_history.md`** - Consolidated history of major implementation sessions including ACP integration, tool configuration, and system improvements

### Loop Detection Evolution
- **`loop_detection_sessions.md`** - Complete history of loop hang detection feature development, improvements, and refactoring sessions

### Permission System Development
- **`permission_sessions.md`** - Evolution of the permission system including implementation status, improvements, and integration work

### Security Implementation Reports
- **`security_reports/`** - Historical security verification reports, fix documentation, and implementation completion records

### Tool Implementation History
- **`tool_sessions.md`** - Tool system cleanups, policy implementation, and configuration completion notes

### Styling System Evolution
- **`styling_sessions.md`** - Styling refactor completion reports and implementation status documents

### Feature Implementation Sessions (November 2025)
- **FILE_BROWSER_*.md** (7 files) - File browser feature development including fuzzy search, sorting, tree view fixes, and UX improvements
- **FILE_REFERENCE_*.md** (14 files) - File reference system implementation, optimizations, slash command integration, and performance improvements
- **Current Docs**: File browser and reference features are documented in active user guides

### Research and Analysis Sessions (November 2025)
- **CODEX_*.md** (5 files) - Codex implementation analysis, improvement opportunities, and issue reviews
- **codex_issue_*.md** - Codex issue implementation progress and reviews
- **prompt_caching.md** - Prompt caching research
- **Current Docs**: Active features integrated into main documentation and provider guides

### Refactoring Sessions (November 2025)
- **MIGRATION_STRATEGY.md** - Migration strategy documentation
- **tui_session_*.md** (4 files) - TUI session refactoring patterns, technical details, and next steps
- **Current Docs**: TUI architecture documented in development guides

### VS Code Extension Development (November 2025)
- **VSCODE_EXTENSION_*.md** (7 files) - VS Code extension improvements, migration roadmap, code examples, and review summaries
- **ANALYSIS_COMPLETE.md**, **INDEX.md** - Extension analysis and documentation index
- **PHASE_*_SUMMARY.md** (3 files) - Phase completion and implementation summaries
- **README_VSCODE_REVIEW.md**, **TODAY_SESSION_COMPLETION.md** - Session completion reports

### Bug Fixes (November 2025)
- **mcp_broken_pipe_fix.md** - MCP broken pipe bug fix documentation
- **Current Docs**: MCP integration is documented in docs/mcp/

## Why Archive?

These documents were created during active development to track progress and decisions. While they contain valuable historical context, they are no longer part of the active documentation set. Key information from these documents has been:

1. **Integrated into active documentation** - Current reference guides incorporate the final state
2. **Summarized in CHANGELOG** - Release notes capture the user-facing changes
3. **Referenced in ARCHITECTURE** - Design decisions are documented in architecture docs

## Using Archived Documentation

When reviewing archived docs:

- **For current features**: Refer to the main documentation in `/docs/` instead
- **For historical context**: These archives show how features evolved
- **For implementation details**: See commit history and git blame for line-by-line changes
- **For API changes**: Check CHANGELOG and migration guides

## Archive Policy

Documents are archived when they are:
- Session-specific completion summaries
- Implementation status reports that are no longer current
- Redundant with consolidated documentation
- Historical snapshots superseded by newer documentation

All archived documents are preserved in git history and can be retrieved if needed.

---

**Last Updated:** November 2025
**VT Code Version:** 0.43.6
