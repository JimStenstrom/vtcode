# Changelog

All notable changes to VTCode Zed Extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-09

### Added

**Core Features:**
- CLI Integration with full VTCode command execution and error handling
- Command Palette integration with 5 primary commands (Ask, Analyze, Chat, Status, About Selection)
- Output channel with thread-safe message management and formatting
- Configuration system with TOML parsing and sensible defaults

**Editor Integration:**
- EditorContext for code selection and workspace context
- Diagnostics tracking with error/warning/info levels and quick fixes
- StatusIndicator for CLI availability in status bar
- EditorState for thread-safe state management

**Configuration Management:**
- Comprehensive configuration validation with rule checking
- Detailed error reporting with actionable suggestions
- Warning system for non-critical issues
- Per-section validation (AI, workspace, security)

**Context Awareness:**
- Workspace structure analysis with directory traversal and file discovery
- File content extraction with size limits
- Syntax-aware selection context
- Open buffers tracking and state management
- Project hierarchy analysis with language metrics

**Error Handling & Recovery:**
- Comprehensive error variants for all failure scenarios
- Automatic retry logic with exponential backoff
- Professional error messages with actionable suggestions
- Thread-safe error state management

**Performance Optimization:**
- Multi-level caching (workspace, files, command-level)
- Intelligent LRU and TTL-based cache eviction
- Memory bounds with 100MB maximum cache size
- Zero-allocation fast path for cache hits

### Quality Metrics
- 107 unit tests (100% passing)
- 0 clippy warnings
- ~3,705 lines of code across 11 modules
- Build time: <2 seconds (incremental)
- Test execution: <100ms total

## [0.2.0] - 2025-11-09

### Added
- Initial VTCode CLI integration
- Basic command palette commands
- Output channel for VTCode responses
- Configuration file support
