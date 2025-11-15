# Tools Configuration

This document describes the tools-related configuration in `vtcode.toml`.

**For the main configuration reference including tool policies**, see [VTCode Configuration](../config.md#security-and-approvals).

**For tool development documentation**, see [Tool Development Guide](../development/tool-development.md).

## Configuration Options

### max_tool_loops

Maximum number of inner tool-call loops per user turn. Prevents infinite tool-calling cycles in interactive chat.

- **Configuration**: `[tools].max_tool_loops` in `vtcode.toml`
- **Code default**: defined in `vtcode-core/src/config/core/tools.rs`
- **Default value**: `100`
- **Purpose**: Protects against infinite tool-calling loops

### default_policy

Default policy for tool execution approval.

- **Configuration**: `[tools].default_policy` in `vtcode.toml`
- **Options**: `"prompt"`, `"allow"`, `"deny"`
- **Default value**: `"prompt"`
- **Purpose**: Controls whether tools require human approval before execution

## Example Configuration

```toml
[tools]
default_policy = "prompt"
max_tool_loops = 100
```

## Tool Policies

Individual tool policies can be configured under `[tools.policies]` to override the default policy for specific tools. See the main [configuration guide](../config.md#toolspolicies) for details.

## Rendering

Tool outputs are rendered with ANSI styles in the chat interface. Tools should return plain text for proper formatting.
