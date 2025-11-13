# VTCode Tools Documentation

This directory contains specifications and documentation for all tools available to VTCode agents.

## Overview

VTCode provides a comprehensive set of tools that enable AI agents to interact with code, files, the terminal, and external resources. The tool system is designed around modularity, safety, and policy-based execution.

## Tool Documentation

### Core Tool Specifications

**[TOOL_SPECS.md](./TOOL_SPECS.md)** - Comprehensive tool specifications including:

**File Operations:**
- `grep_file` - Unified code search with multiple modes
- `list_files` - File discovery and navigation
- `read_file` - Read file contents
- `write_file` - Write content to files
- `edit_file` - Replace specific text in files

**Execution:**
- `run_terminal_cmd` - Execute shell commands with PTY support

**Advanced Operations:**
- `apply_patch` - Apply unified diff patches
- `web_fetch` - Fetch content from URLs (see security configuration)
- `tree_sitter` - Syntax-aware code analysis

**Additional Tools:**
- Git operations, planning tools, and MCP-provided tools

### Security Configuration

**[web_fetch_security.md](./web_fetch_security.md)** - Security configuration for the web_fetch tool:
- Restricted mode (default)
- Open mode
- Custom URL patterns
- Security best practices

## Related Documentation

### User Guides
- **[Git Commands Quick Reference](../user-guide/GIT_QUICK_REFERENCE.md)** - Git command usage
- **[Git Command Execution Policy](../user-guide/GIT_COMMAND_EXECUTION.md)** - Git security tiers

### System Guides
- **[Prompt Caching Guide](../guides/prompt_caching_guide.md)** - Optimize prompt caching for tools
- **[Configuration Reference](../config.md)** - Tool policies and configuration

### Developer Documentation
- **[Tool Development Guide](../development/tool-development.md)** - Implement custom tools
- **[Justification System](../development/JUSTIFICATION_SYSTEM.md)** - Agent approval system architecture
- **[Async Architecture](../development/async-architecture.md)** - Tool async/await patterns

## Tool System Architecture

### Registry Pattern
All tools are registered in a central `ToolRegistry` that handles:
- Tool discovery and invocation
- Parameter validation
- Policy enforcement
- Timeout management
- Result caching

### Policy System
Tools respect configurable policies defined in `vtcode.toml`:

```toml
[tools]
default_policy = "prompt"  # prompt | allow | deny
max_tool_loops = 100

[tools.policies]
run_terminal_cmd = "prompt"
read_file = "allow"
write_file = "prompt"
```

See **[Tool Configuration](../config/TOOLS_CONFIG.md)** for details.

### Safety Features
- **Path Validation**: All file operations check workspace boundaries
- **Command Policies**: Allow/deny lists for shell commands
- **Execution Limits**: Timeouts and resource controls
- **Audit Logging**: Complete trail of tool executions

## Tool Development

To implement custom tools:

1. Define a tool struct implementing the `Tool` trait
2. Register with `ToolRegistry`
3. Configure policies in `vtcode.toml`
4. Test with sample payloads

See the **[Tool Development Guide](../development/tool-development.md)** for step-by-step instructions.

## Tool Policies

Tools can be configured with different permission levels:
- **allow** - Execute without prompting
- **prompt** - Request user approval
- **deny** - Block execution

Policy configuration is per-tool and can be scoped to specific workspaces or globally.

## MCP Integration

VTCode supports Model Context Protocol (MCP) servers that can provide additional tools:
- Database access
- API integrations
- Cloud service operations
- Custom business logic

See **[MCP Integration](../mcp/)** for configuration.

---

**Last Updated**: November 2025
**VTCode Version**: 0.43.6
