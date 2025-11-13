# vtcode-mcp

[![Crates.io](https://img.shields.io/crates/v/vtcode-mcp.svg)](https://crates.io/crates/vtcode-mcp)
[![Documentation](https://docs.rs/vtcode-mcp/badge.svg)](https://docs.rs/vtcode-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Model Context Protocol (MCP) client implementation for VTCode.

## Overview

`vtcode-mcp` is a standalone, reusable MCP client built on top of the [rmcp](https://crates.io/crates/rmcp) library. It provides a complete MCP client with VTCode-specific features like multi-provider management, security policies, and progressive tool discovery.

## Features

- **Multi-Transport Support**: Connect via stdio or HTTP transports
- **Provider Management**: Manage multiple MCP providers simultaneously
- **Security**: Tool and resource allowlisting with argument validation
- **Tool Discovery**: Progressive disclosure of tools to minimize context overhead
- **Elicitation Support**: Handle user input requests from MCP providers
- **CLI Commands**: Built-in commands for provider configuration
- **Configuration Validation**: Enhanced validation with helpful error messages

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vtcode-mcp = "0.43.6"
```

## Quick Start

### Basic Usage

```rust
use vtcode_mcp::{McpClient, McpClientConfig};
use vtcode_config::mcp::McpClientConfig as Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::default();

    // Create and initialize client
    let mut client = McpClient::new(config);
    client.initialize().await?;

    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {}", tools.len());

    // Execute a tool
    let args = serde_json::json!({
        "path": "/workspace/file.txt"
    });
    let result = client.execute_tool("read_file", args).await?;

    // Shutdown when done
    client.shutdown().await?;

    Ok(())
}
```

### Using Tool Discovery

```rust
use vtcode_mcp::{ToolDiscovery, DetailLevel};
use std::sync::Arc;

// Create tool discovery service
let discovery = ToolDiscovery::new(Arc::new(client));

// Search for tools by keyword (minimal context)
let results = discovery.search_tools("file", DetailLevel::NameOnly).await?;
for tool in results {
    println!("Found: {}", tool.name);
}

// Get full schema when needed
let detail = discovery.get_tool_detail("read_file").await?;
if let Some(tool) = detail {
    println!("Schema: {:?}", tool.input_schema);
}
```

### CLI Commands

Add or update an MCP provider:

```bash
vtcode mcp add my-provider npx -y @modelcontextprotocol/server-filesystem /workspace
```

List configured providers:

```bash
vtcode mcp list
```

Remove a provider:

```bash
vtcode mcp remove my-provider
```

## Configuration

### Provider Configuration

Providers can be configured in `vtcode.toml`:

```toml
[mcp]
enabled = true
request_timeout_seconds = 30
tool_timeout_seconds = 60

[[mcp.providers]]
name = "filesystem"
enabled = true
max_concurrent_requests = 5

[mcp.providers.transport]
type = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
```

### HTTP Provider

```toml
[[mcp.providers]]
name = "remote-api"
enabled = true

[mcp.providers.transport]
type = "http"
endpoint = "https://api.example.com/mcp"
api_key_env = "MCP_API_KEY"
protocol_version = "2024-11-05"
```

### Security Configuration

```toml
[mcp.security]
auth_enabled = false

[mcp.security.validation]
schema_validation_enabled = true
path_traversal_protection = true
max_argument_size = 1048576  # 1MB
```

### Allowlist Configuration

Control which tools and resources are accessible:

```toml
[mcp.allowlist]
mode = "allow"  # or "deny"

[[mcp.allowlist.tools]]
provider = "filesystem"
patterns = ["read_*", "list_*"]

[[mcp.allowlist.resources]]
provider = "filesystem"
patterns = ["file:///workspace/*"]
```

## Architecture

### Module Structure

```
vtcode-mcp/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── client.rs           # Core MCP client (2.5K LOC)
│   ├── cli.rs              # CLI commands (633 LOC)
│   ├── enhanced_config.rs  # Configuration validation (361 LOC)
│   └── tool_discovery.rs   # Progressive tool discovery (325 LOC)
├── tests/                  # Integration tests
└── examples/               # Usage examples
```

### Key Components

#### McpClient

The main client for managing MCP providers:

```rust
pub struct McpClient {
    // Multi-provider management
    // Tool execution
    // Resource access
    // Prompt retrieval
}
```

**Key Methods:**
- `initialize()` - Connect to all configured providers
- `execute_tool(name, args)` - Execute a tool with validation
- `list_tools()` - List all available tools
- `list_resources()` - List all available resources
- `get_prompt(name, args)` - Retrieve a rendered prompt
- `shutdown()` - Gracefully close all connections

#### ToolDiscovery

Progressive tool discovery to minimize context usage:

```rust
pub struct ToolDiscovery {
    // Search tools by keyword
    // Get tool details on demand
    // List by provider
}
```

**Detail Levels:**
- `NameOnly` - Just the tool name (minimal context)
- `NameAndDescription` - Name + description (default)
- `Full` - Complete schema including parameters

#### Enhanced Configuration

Validation and enhanced security configuration:

```rust
pub struct ValidatedMcpClientConfig {
    pub original: McpClientConfig,
    pub security: EnhancedMcpSecurityConfig,
}
```

## Testing

Run the test suite:

```bash
cargo test -p vtcode-mcp
```

Run with output:

```bash
cargo test -p vtcode-mcp -- --nocapture
```

## Examples

See the `examples/` directory for more usage examples:

- `basic_client.rs` - Simple client setup and tool execution
- `tool_discovery.rs` - Progressive tool discovery
- `cli_usage.rs` - Using CLI commands programmatically

## Security

### Argument Validation

The client validates all tool arguments before execution:

- **Size limits**: Configurable maximum argument size
- **Path traversal**: Automatic detection and prevention
- **Schema validation**: JSON schema validation for tool arguments

### Allowlisting

Fine-grained control over tool and resource access:

```rust
// Check if a tool is allowed
if !allowlist.is_tool_allowed("provider", "dangerous_tool") {
    return Err(anyhow!("Tool blocked by allowlist"));
}
```

### Timeout Protection

All operations have configurable timeouts:

- Startup timeout for provider initialization
- Tool timeout for individual tool calls
- Request timeout for resource/prompt fetches

## Performance

### Caching

The client caches:
- Tool lists per provider
- Resource lists per provider
- Prompt lists per provider
- Provider-to-tool mappings

### Concurrency

Configurable concurrent request limits per provider:

```toml
[[mcp.providers]]
name = "my-provider"
max_concurrent_requests = 10  # Adjust based on provider capacity
```

## Error Handling

The client uses structured error types:

```rust
use anyhow::Result;

match client.execute_tool("read_file", args).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) => {
        eprintln!("Error: {}", e);
        // Error includes context about which provider failed
    }
}
```

Common error scenarios:
- Provider not found
- Tool not found
- Timeout exceeded
- Validation failed
- Connection failed

## Integration with VTCode

This crate is designed to work seamlessly with VTCode's tool system:

```rust
use vtcode_mcp::McpClient;
use vtcode_core::tools::ToolRegistry;

// Register MCP tools with VTCode's tool registry
let mcp_client = Arc::new(McpClient::new(config));
registry.set_mcp_client(mcp_client.clone()).await?;
```

## Contributing

Contributions are welcome! This crate is part of the VTCode project.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/vinhnx/vtcode
cd vtcode/vtcode-mcp

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- [vtcode](https://github.com/vinhnx/vtcode) - The main VTCode project
- [rmcp](https://crates.io/crates/rmcp) - The underlying MCP client library
- [Model Context Protocol](https://modelcontextprotocol.io) - The MCP specification

## Changelog

### 0.43.6 (Phase 2)

- Initial extraction from vtcode-core
- Standalone, reusable MCP client crate
- Complete feature set with ~3.8K LOC
- Multi-provider management
- Security and validation
- Progressive tool discovery
- CLI commands

## Support

- **Documentation**: https://docs.rs/vtcode-mcp
- **Issues**: https://github.com/vinhnx/vtcode/issues
- **Discussions**: https://github.com/vinhnx/vtcode/discussions
