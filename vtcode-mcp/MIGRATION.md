# Migration Guide: vtcode-core/mcp → vtcode-mcp

This guide helps you migrate from using the internal MCP module in vtcode-core to the standalone vtcode-mcp crate.

## Overview

As of Phase 2 of the architecture transformation, the MCP subsystem has been extracted from `vtcode-core` into a standalone `vtcode-mcp` crate. This provides better modularity and allows the MCP client to be used independently.

## For vtcode-core Users (No Changes Required)

**Good News**: If you're using MCP through `vtcode-core`, **no changes are required**!

The `vtcode-core` crate maintains full backward compatibility through re-exports:

```rust
// This still works exactly the same
use vtcode_core::mcp::{McpClient, McpToolExecutor};
```

Behind the scenes, `vtcode_core::mcp` now re-exports from `vtcode_mcp`, but the API is identical.

## For Direct vtcode-mcp Users

If you want to use the standalone `vtcode-mcp` crate directly (not through vtcode-core):

### Before (vtcode-core)

```rust
// Old: Using internal module
use vtcode_core::mcp::{McpClient, McpClientConfig};

let client = McpClient::new(config);
```

### After (vtcode-mcp)

```rust
// New: Using standalone crate
use vtcode_mcp::{McpClient, McpClientConfig};
use vtcode_config::mcp::McpClientConfig;

let client = McpClient::new(config);
```

### Configuration

Configuration types remain in `vtcode-config`:

```rust
use vtcode_config::mcp::{
    McpClientConfig,
    McpProviderConfig,
    McpTransportConfig,
    McpStdioServerConfig,
    McpHttpServerConfig,
};
```

## Import Path Changes

### Module Structure (Before)

```rust
vtcode_core::mcp
├── McpClient
├── McpToolExecutor
├── McpToolInfo
├── cli
├── enhanced_config
└── tool_discovery
```

### Module Structure (After)

```rust
vtcode_mcp
├── client
│   ├── McpClient
│   ├── McpToolExecutor
│   └── McpToolInfo
├── cli
│   ├── McpCommands
│   └── handle_mcp_command
├── enhanced_config
│   ├── ValidatedMcpClientConfig
│   └── EnhancedMcpSecurityConfig
└── tool_discovery
    ├── ToolDiscovery
    └── DetailLevel
```

All types are re-exported from the root for convenience:

```rust
use vtcode_mcp::{
    McpClient,           // from client
    ToolDiscovery,       // from tool_discovery
    ValidatedMcpClientConfig, // from enhanced_config
};
```

## Dependency Changes

### Cargo.toml

If you're building a project that uses MCP directly:

**Before:**
```toml
[dependencies]
vtcode-core = "0.43.6"
```

**After (if you only need MCP):**
```toml
[dependencies]
vtcode-mcp = "0.43.6"
vtcode-config = "0.43.6"  # For configuration types
```

**After (if you still need vtcode-core):**
```toml
[dependencies]
vtcode-core = "0.43.6"  # Already includes vtcode-mcp
```

## API Compatibility

### ✅ No Breaking Changes

All public APIs remain exactly the same:

- `McpClient::new(config)` - unchanged
- `client.initialize().await` - unchanged
- `client.execute_tool(name, args).await` - unchanged
- `client.list_tools().await` - unchanged
- CLI commands - unchanged
- Configuration structure - unchanged

### Function Signatures

No function signatures have changed. All methods have the same:
- Parameters
- Return types
- Error types
- Async/sync behavior

## Feature Flags

The `vtcode-mcp` crate supports optional features:

```toml
[dependencies]
vtcode-mcp = { version = "0.43.6", features = ["schema"] }
```

**Available features:**
- `schema`: Enable JSON schema generation for configuration types
- `tool-traits`: Enable integration with vtcode-tool-traits (future)

## Testing Your Migration

### 1. Compile Check

```bash
cargo check
```

If you're using `vtcode-core`, this should succeed without changes.

### 2. Test Existing Code

```bash
cargo test
```

All existing tests should pass without modification.

### 3. Runtime Verification

Your existing MCP configuration and provider setup will work identically:

```toml
# vtcode.toml - no changes needed
[mcp]
enabled = true

[[mcp.providers]]
name = "filesystem"
# ... rest of config unchanged
```

## Benefits of the New Structure

### For Library Users

✅ **Lighter Dependencies**: Use only MCP without full vtcode-core
✅ **Better Modularity**: Clear separation of concerns
✅ **Improved Documentation**: Dedicated MCP documentation
✅ **Independent Versioning**: MCP can evolve independently

### For Contributors

✅ **Easier Testing**: Test MCP in isolation
✅ **Clearer Boundaries**: Well-defined module interfaces
✅ **Reduced Compilation**: Faster iteration on MCP changes
✅ **Better Organization**: Logical code structure

## Troubleshooting

### Issue: "Cannot find vtcode_mcp in dependencies"

**Solution**: If you're using `vtcode-core`, ensure you're on version 0.43.6 or later:

```toml
vtcode-core = "0.43.6"
```

### Issue: "Type mismatch on McpClientConfig"

**Solution**: Configuration types are still in `vtcode-config`:

```rust
use vtcode_config::mcp::McpClientConfig;
use vtcode_mcp::McpClient;
```

### Issue: "Trait McpToolExecutor not in scope"

**Solution**: Import from vtcode-mcp:

```rust
use vtcode_mcp::McpToolExecutor;
```

## Advanced: Using Without vtcode-core

If you want to use `vtcode-mcp` in a standalone project:

```rust
use vtcode_mcp::{McpClient, McpToolExecutor};
use vtcode_config::mcp::{
    McpClientConfig, McpProviderConfig, McpStdioServerConfig, McpTransportConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build configuration
    let config = McpClientConfig {
        enabled: true,
        providers: vec![
            McpProviderConfig {
                name: "my-provider".to_string(),
                transport: McpTransportConfig::Stdio(McpStdioServerConfig {
                    command: "npx".to_string(),
                    args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
                    working_directory: None,
                }),
                enabled: true,
                max_concurrent_requests: 5,
                // ... other fields
            }
        ],
        // ... other configuration
    };

    // Use the client
    let mut client = McpClient::new(config);
    client.initialize().await?;

    Ok(())
}
```

## Questions?

- **Documentation**: https://docs.rs/vtcode-mcp
- **Examples**: See `examples/` directory in vtcode-mcp
- **Issues**: https://github.com/vinhnx/vtcode/issues

## Timeline

- **Phase 1**: Foundation types extracted (vtcode-llm-types, vtcode-tool-traits)
- **Phase 2** (Current): MCP extraction complete ← **You are here**
- **Phase 3**: LLM provider modularization
- **Phase 4**: Tool modularization
- **Phase 5**: Final polish
