# vtcode-mcp

Model Context Protocol (MCP) client, connection pooling, and tool discovery
for VT Code. Extracted from `vtcode-core` to isolate the MCP subsystem into an
independently compilable crate.

## Modules

| Module | Purpose |
|---|---|
| `client` | MCP client lifecycle and provider management |
| `connection_pool` | Connection pooling for MCP providers |
| `enhanced_config` | Enhanced MCP configuration with validation |
| `errors` | MCP-specific error types |
| `provider` | MCP provider connection and interaction |
| `rmcp_client` | Low-level rmcp protocol adapter |
| `rmcp_transport` | HTTP and stdio transport layers |
| `schema` | JSON Schema validation for tool inputs |
| `tool_discovery` | Dynamic tool discovery from MCP providers |
| `tool_discovery_cache` | Caching for discovered tools |
| `traits` | `McpToolExecutor` and `McpElicitationHandler` traits |
| `types` | MCP protocol types (tools, prompts, resources) |
| `utils` | Timezone injection, header building, schema helpers |

## Public entrypoints

- `McpClient` -- manage MCP providers and invoke tools
- `McpProvider` -- single provider connection
- `McpConnectionPool` / `PooledMcpManager` -- connection pooling
- `ToolDiscovery` / `ToolDiscoveryResult` -- dynamic tool discovery
- `McpToolExecutor` / `McpElicitationHandler` -- trait interfaces
- `validate_mcp_config` -- configuration validation

## Features

| Feature | Description |
|---|---|
| `schema` | JSON Schema validation via `schemars` |

## Usage

```rust
use vtcode_mcp::{McpClient, McpToolInfo};

let client = McpClient::new(config);
let tools = client.list_tools().await?;
```

## API reference

<https://docs.rs/vtcode-mcp>
