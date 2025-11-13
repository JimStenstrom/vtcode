//! User interface utilities and shared UI components
//!
//! **Note**: As of vtcode 0.43.6, the UI implementation has been extracted into
//! the standalone [`vtcode-ui`](https://docs.rs/vtcode-ui) crate as part of
//! Phase 2 of the architecture transformation.
//!
//! This module re-exports all types from `vtcode-ui` for backward compatibility,
//! so existing code using `vtcode_core::ui` will continue to work without changes.
//!
//! # Migration
//!
//! For new code, you can choose to either:
//! 1. Continue using `vtcode_core::ui` (recommended for vtcode integration)
//! 2. Use `vtcode_ui` directly (for standalone UI usage)
//!
//! ```rust,ignore
//! // Option 1: Through vtcode-core (unchanged)
//! use vtcode_core::ui::{render_markdown, ThemeManager};
//!
//! // Option 2: Direct vtcode-ui usage
//! use vtcode_ui::{render_markdown, ThemeManager};
//! ```
//!
//! Both approaches work identically. See [`vtcode-ui` documentation](https://docs.rs/vtcode-ui)
//! for details.
//!
//! # Re-exported Types
//!
//! All public types from `vtcode-ui` are re-exported here:
//! - Terminal UI components (TUI, session management)
//! - Theme system (themes, configuration, manager)
//! - Rendering utilities (markdown, diff, file colorization)
//! - Input and interaction components
//! - User confirmation dialogs
//!
//! See `vtcode-ui` crate documentation for detailed API reference.

pub use vtcode_ui::*;
