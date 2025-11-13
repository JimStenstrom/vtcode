//! Tool discovery example
//!
//! This example demonstrates progressive tool discovery:
//! - Searching tools by keyword with minimal context
//! - Getting detailed schema only when needed
//! - Listing tools by provider

use std::sync::Arc;
use vtcode_config::mcp::{
    McpAllowListConfig, McpClientConfig, McpProviderConfig, McpServerConfig,
    McpStdioServerConfig, McpTransportConfig,
};
use vtcode_mcp::{DetailLevel, McpClient, ToolDiscovery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Tool Discovery Example ===\n");

    // Create and initialize MCP client
    let config = create_example_config();
    let mut client = McpClient::new(config);

    println!("Initializing MCP client...");
    match client.initialize().await {
        Ok(_) => println!("✓ Initialized\n"),
        Err(e) => {
            eprintln!("✗ Failed to initialize: {}", e);
            eprintln!("Note: Install MCP server with:");
            eprintln!("  npm install -g @modelcontextprotocol/server-filesystem");
            return Ok(());
        }
    }

    // Create tool discovery service
    let discovery = ToolDiscovery::new(Arc::new(client));

    // Example 1: Search with minimal context (name only)
    println!("=== Example 1: Minimal Context Search ===");
    println!("Searching for 'file' tools (name only)...");
    match discovery
        .search_tools("file", DetailLevel::NameOnly)
        .await
    {
        Ok(results) => {
            println!("Found {} matching tools:", results.len());
            for tool in results.iter().take(5) {
                println!("  - {} (provider: {})", tool.name, tool.provider);
                println!("    Relevance: {:.2}", tool.relevance_score);
            }
        }
        Err(e) => eprintln!("Search failed: {}", e),
    }
    println!();

    // Example 2: Search with descriptions
    println!("=== Example 2: Search with Descriptions ===");
    println!("Searching for 'read' tools (with descriptions)...");
    match discovery
        .search_tools("read", DetailLevel::NameAndDescription)
        .await
    {
        Ok(results) => {
            println!("Found {} matching tools:", results.len());
            for tool in results.iter().take(3) {
                println!("  - {} ({})", tool.name, tool.provider);
                println!("    Description: {}", tool.description);
                println!("    Relevance: {:.2}", tool.relevance_score);
            }
        }
        Err(e) => eprintln!("Search failed: {}", e),
    }
    println!();

    // Example 3: Get full schema for a specific tool
    println!("=== Example 3: Get Full Tool Detail ===");
    println!("Getting full schema for 'read_file'...");
    match discovery.get_tool_detail("read_file").await {
        Ok(Some(detail)) => {
            println!("Tool: {} ({})", detail.name, detail.provider);
            println!("Description: {}", detail.description);
            if let Some(schema) = detail.input_schema {
                println!("Input Schema:");
                println!("{}", serde_json::to_string_pretty(&schema)?);
            }
        }
        Ok(None) => println!("Tool not found"),
        Err(e) => eprintln!("Failed to get detail: {}", e),
    }
    println!();

    // Example 4: List tools by provider
    println!("=== Example 4: List Tools by Provider ===");
    match discovery.list_tools_by_provider().await {
        Ok(by_provider) => {
            for (provider, tools) in by_provider {
                println!("Provider: {}", provider);
                println!("  Tools ({}): ", tools.len());
                for tool in tools.iter().take(5) {
                    println!("    - {}", tool.name);
                }
                if tools.len() > 5 {
                    println!("    ... and {} more", tools.len() - 5);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Failed to list by provider: {}", e),
    }

    println!("=== Progressive Disclosure Benefits ===");
    println!("✓ Minimal context usage when browsing tools");
    println!("✓ Full schemas loaded only when needed");
    println!("✓ Efficient for agents with limited context windows");

    Ok(())
}

fn create_example_config() -> McpClientConfig {
    McpClientConfig {
        enabled: true,
        ui: Default::default(),
        providers: vec![McpProviderConfig {
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
            startup_timeout_ms: Some(10000),
        }],
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
    }
}
