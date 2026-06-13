//! Small compatibility helpers that avoid pulling in heavy dependencies
//! (e.g. `chrono`) for trivial needs.

// Canonical implementation lives in vtcode-core; re-export here for
// backward compatibility with acp_tool and any other consumers.
pub use vtcode_core::tools::time_compat::current_timestamp_rfc3339;
