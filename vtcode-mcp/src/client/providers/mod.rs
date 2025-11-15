//! Provider management and lifecycle.

pub mod handler;
pub mod provider;
pub mod rmcp_client;

pub use provider::McpProvider;
pub use rmcp_client::RmcpClient;
