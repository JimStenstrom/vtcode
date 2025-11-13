//! Basic MCP client usage example
//!
//! This example demonstrates:
//! - Creating an MCP client
//! - Initializing with providers
//! - Listing available tools
//! - Getting client status
//! - Proper shutdown

use vtcode_config::mcp::{
    McpAllowListConfig, McpClientConfig, McpProviderConfig, McpServerConfig,
    McpStdioServerConfig, McpTransportConfig,
};
use vtcode_mcp::McpClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("=== VTCode MCP Client Example ===\n");

    // Create a basic configuration
    let config = McpClientConfig {
        enabled: true,
        ui: Default::default(),
        providers: vec![
            // Example: filesystem provider
            McpProviderConfig {
                name: "filesystem".to_string(),
                transport: McpTransportConfig::Stdio(McpStdioServerConfig {
                    command: "npx".to_string(),
                    args: vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/workspace".to_string(),
                    ],
                    working_directory: None,
                }),
                env: Default::default(),
                enabled: true,
                max_concurrent_requests: 5,
                startup_timeout_ms: Some(10000), // 10 seconds
            },
        ],
        server: McpServerConfig {
            enabled: false,
            bind_address: "127.0.0.1".to_string(),
            port: 3000,
            transport: vtcode_config::mcp::McpServerTransport::Sse,
            name: "example".to_string(),
            version: "1.0.0".to_string(),
            exposed_tools: vec![],
        },
        allowlist: McpAllowListConfig::default(),
        max_concurrent_connections: 10,
        request_timeout_seconds: 30,
        retry_attempts: 3,
        startup_timeout_seconds: Some(60),
        tool_timeout_seconds: Some(300),
        experimental_use_rmcp_client: true,
        security: Default::default(),
    };

    // Create the MCP client
    println!("Creating MCP client...");
    let mut client = McpClient::new(config);

    // Get initial status
    let status = client.get_status();
    println!("Initial status:");
    println!("  Enabled: {}", status.enabled);
    println!("  Providers configured: {}", status.provider_count);
    println!();

    // Initialize (this connects to all providers)
    println!("Initializing MCP client...");
    match client.initialize().await {
        Ok(_) => println!("✓ MCP client initialized successfully"),
        Err(e) => {
            eprintln!("✗ Failed to initialize: {}", e);
            eprintln!("Note: This is expected if you don't have the MCP server installed.");
            eprintln!("Install it with: npm install -g @modelcontextprotocol/server-filesystem");
            return Ok(());
        }
    }
    println!();

    // Get updated status
    let status = client.get_status();
    println!("Status after initialization:");
    println!("  Active connections: {}", status.active_connections);
    println!("  Configured providers: {:?}", status.configured_providers);
    println!();

    // List available tools
    println!("Listing available tools...");
    match client.list_tools().await {
        Ok(tools) => {
            println!("Found {} tools:", tools.len());
            for (i, tool) in tools.iter().enumerate().take(5) {
                println!("  {}. {} ({})", i + 1, tool.name, tool.provider);
                if !tool.description.is_empty() {
                    println!("     {}", tool.description);
                }
            }
            if tools.len() > 5 {
                println!("  ... and {} more", tools.len() - 5);
            }
        }
        Err(e) => eprintln!("Failed to list tools: {}", e),
    }
    println!();

    // List available resources
    println!("Listing available resources...");
    match client.list_resources().await {
        Ok(resources) => {
            println!("Found {} resources:", resources.len());
            for (i, resource) in resources.iter().enumerate().take(5) {
                println!("  {}. {} ({})", i + 1, resource.name, resource.provider);
                println!("     URI: {}", resource.uri);
            }
            if resources.len() > 5 {
                println!("  ... and {} more", resources.len() - 5);
            }
        }
        Err(e) => eprintln!("Failed to list resources: {}", e),
    }
    println!();

    // Example: Execute a tool (if available)
    println!("Example tool execution:");
    println!("  (This would execute a tool with validated arguments)");
    println!("  let result = client.execute_tool(\"list_files\", args).await?;");
    println!();

    // Shutdown
    println!("Shutting down MCP client...");
    client.shutdown().await?;
    println!("✓ Shutdown complete");

    Ok(())
}
