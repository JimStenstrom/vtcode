//! Style and color utilities for VTCode
//!
//! This module consolidates all styling, color conversion, and ANSI handling
//! into a coherent structure:
//!
//! - `ansi` - ANSI escape code parsing, stripping, and utilities
//! - `color` - Color types, conversions, and builder patterns
//! - `bridge` - Conversions between anstyle and ratatui styles
//!
//! ## Migration from Old Structure
//!
//! Old files consolidated here:
//! - `utils/ansi.rs` → `style/ansi.rs` (ANSI utilities)
//! - `utils/ansi_parser.rs` → `style/ansi.rs` (parsing)
//! - `utils/colors.rs` → `style/color.rs` (StyledString)
//! - `utils/color_utils.rs` → `style/color.rs` (color conversions)
//! - `utils/anstyle_utils.rs` → `style/bridge.rs` (anstyle ↔ ratatui)
//! - `utils/ratatui_styles.rs` → `style/bridge.rs` (ratatui helpers)
//! - `utils/style_helpers.rs` → `style/color.rs` (style builders)
//! - `utils/cached_style_parser.rs` → `style/ansi.rs` (cached parsing)
//! - `utils/diff_styles.rs` → `style/color.rs` (diff coloring)

pub mod ansi;
pub mod bridge;
pub mod color;

// Re-export commonly used types
pub use ansi::{MessageStyle, strip_ansi_codes};
pub use bridge::{ansi_style_to_ratatui, ansi_color_to_ratatui};
pub use color::{StyledString, style};
