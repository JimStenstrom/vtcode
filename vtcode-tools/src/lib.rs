//! ACP tool integration for inter-agent communication.
//!
//! This crate provides ACP-specific tools that depend on both vtcode-core
//! and vtcode-acp. All other tool infrastructure has been merged into
//! vtcode-core.

pub mod acp_tool;
pub use acp_tool::{AcpDiscoveryTool, AcpHealthTool, AcpTool};

pub mod compat;

#[cfg(test)]
mod tests {
    use super::compat::current_timestamp_rfc3339;

    #[test]
    fn compat_re_export_produces_valid_rfc3339() {
        let ts = current_timestamp_rfc3339();
        // RFC 3339 timestamps contain 'T' separating date and time
        assert!(ts.contains('T'), "expected RFC 3339 timestamp, got: {ts}");
    }
}
