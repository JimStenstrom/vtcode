//! Session management for the TUI
//!
//! This module provides the main `Session` type that orchestrates all TUI state and rendering.
//! The Session acts as a thin coordination layer, delegating most functionality to specialized
//! modules organized by concern.
//!
//! # Architecture
//!
//! The Session is organized into several key areas:
//!
//! ## State Management (`state.rs`)
//!
//! State is grouped into logical sub-structures rather than flat fields:
//! - **DisplayState** - Message lines, theme, labels
//! - **PromptState** - Prompt prefix, placeholder, status indicators
//! - **UIState** - Navigation, viewport metrics, redraw flags
//! - **PaletteState** - Slash/file/prompt palettes
//! - **RenderState** - Caches, overlays, modals, task plan
//!
//! ## Core Functionality
//!
//! - **`core.rs`** - Session lifecycle (new, exit, redraw)
//! - **`events/`** - Event handling (keyboard, mouse, paste, resize)
//! - **`rendering/`** - Render coordination (transcript, input, palettes)
//! - **`commands.rs`** - Command processing from backend
//! - **`messages/`** - Message line management (append, replace, reflow)
//!
//! ## Specialized Components
//!
//! - **`input_manager.rs`** - Text input, cursor, history
//! - **`scroll.rs`** - Scroll state and viewport management
//! - **`file_palette.rs`** - File browser for @ references
//! - **`prompt_palette.rs`** - Custom prompt browser for /prompt:
//! - **`slash_palette.rs`** - Slash command suggestions
//! - **`modal.rs`** - Modal dialog system
//! - **`header.rs`** - Session header rendering
//! - **`navigation.rs`** - Timeline/plan navigation pane
//! - **`transcript.rs`** - Transcript reflow and caching
//! - **`slash.rs`** - Slash palette interaction logic
//! - **`palettes/`** - Palette interaction helpers
//!
//! # Public API
//!
//! The Session exposes a minimal public interface:
//!
//! - `new()` - Create a new session
//! - `handle_event()` - Process user input events
//! - `render()` - Render the TUI to a frame
//! - `should_exit()` / `request_exit()` - Exit management
//! - `take_redraw()` - Check if redraw is needed
//!
//! All other functionality is internal and delegated to specialized modules.

use anstyle::RgbColor;
use crossterm::event::Event as CrosstermEvent;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use super::style::{measure_text_width, ratatui_color_from_ansi, ratatui_style_from_inline};
use super::types::InlineEvent;

// ============================================================================
// Module Declarations
// ============================================================================

// Core modules
mod core;
mod state;

// Event handling
mod events;

// Rendering
mod rendering;

// Command and message handling
mod commands;
mod messages;

// Palette and interaction modules
mod palettes;
mod slash;

// Component modules
mod file_palette;
mod file_tree;
mod header;
mod input_manager;
mod message;
mod modal;
mod navigation;
mod palette_renderer;
mod prompt_palette;
mod queue;
mod scroll;
mod slash_palette;
mod transcript;

// Re-exports for internal use
use self::input_manager::InputManager;
use self::scroll::ScrollManager;
use self::state::{DisplayState, PaletteState, PromptState, RenderState, UIState};

// Constants
pub(super) const USER_PREFIX: &str = "";
pub(super) const PLACEHOLDER_COLOR: RgbColor = RgbColor(0x88, 0x88, 0x88);
const PROMPT_COMMAND_NAME: &str = "prompt";
const LEGACY_PROMPT_COMMAND_NAME: &str = "prompts";
const PROMPT_INVOKE_PREFIX: &str = "prompt:";
const LEGACY_PROMPT_INVOKE_PREFIX: &str = "prompts:";
const PROMPT_COMMAND_PREFIX: &str = "/prompt:";

// ============================================================================
// Session Structure
// ============================================================================

/// Main TUI session state and coordinator.
///
/// The Session orchestrates all TUI functionality through a collection of managers
/// and organized state structures. It provides a minimal public API while delegating
/// most operations to specialized modules.
///
/// # State Organization
///
/// Rather than having dozens of flat fields, the Session groups related state into
/// logical sub-structures defined in `state.rs`:
///
/// - **Managers** - High-level subsystems (input, scroll)
/// - **display_state** - Display content and theme
/// - **prompt_state** - Prompt appearance and status
/// - **ui_state** - UI flags and viewport metrics
/// - **palette_state** - Interactive palette states
/// - **render_state** - Rendering caches and overlays
///
/// This organization makes the relationships between state fields clearer and
/// enables better encapsulation of related functionality.
///
/// # Usage
///
/// ```no_run
/// use ratatui::Terminal;
/// use crossterm::event;
/// use vtcode_ui::tui::{Session, InlineTheme};
///
/// let mut session = Session::new(
///     InlineTheme::default(),
///     Some("Type a message...".to_string()),
///     24,  // view_rows
///     false,  // show_timeline_pane
/// );
///
/// loop {
///     if session.should_exit() {
///         break;
///     }
///
///     // Handle events
///     if event::poll(Duration::from_millis(100))? {
///         let event = event::read()?;
///         session.handle_event(event, &events_tx, None);
///     }
///
///     // Render if needed
///     if session.take_redraw() {
///         terminal.draw(|frame| session.render(frame))?;
///     }
/// }
/// ```
pub struct Session {
    // --- Managers ---
    /// Manages user input, cursor position, and command history
    pub(super) input_manager: InputManager,
    /// Manages scroll state and viewport metrics
    pub(super) scroll_manager: ScrollManager,

    // --- State Components ---
    /// Display state for messages and theme
    pub(super) display_state: DisplayState,
    /// Prompt and input status display state
    pub(super) prompt_state: PromptState,
    /// UI state and viewport metrics
    pub(super) ui_state: UIState,
    /// Palette state for command and file browsers
    pub(super) palette_state: PaletteState,
    /// Render state and caching
    pub(super) render_state: RenderState,
}

// ============================================================================
// Public API
// ============================================================================

// Note: Most public methods are implemented in specialized modules:
// - new(), should_exit(), request_exit(), take_redraw() -> core.rs
// - handle_event() -> events/mod.rs (delegates to this module)
// - render() -> rendering/mod.rs (delegates to this module)
// - handle_command() -> commands.rs

impl Session {
    /// Handle all types of events for the session.
    ///
    /// This is the main event dispatcher that processes user input and system events.
    /// It delegates to the `events` module for specialized handling of different event types.
    ///
    /// Supported event types:
    /// - **Key events** - Keyboard input, shortcuts, navigation
    /// - **Mouse events** - Scroll wheel, clicks
    /// - **Paste events** - Text insertion from clipboard
    /// - **Resize events** - Terminal dimension changes
    ///
    /// # Arguments
    ///
    /// * `event` - The crossterm event to process
    /// * `events` - Channel to send inline events to backend
    /// * `callback` - Optional callback to invoke with emitted events
    ///
    /// # Event Priority
    ///
    /// The event handling respects this priority order:
    /// 1. Modal dialogs (if open)
    /// 2. File palette (if active)
    /// 3. Prompt palette (if active)
    /// 4. Slash palette navigation (if available)
    /// 5. Regular input handling
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crossterm::event;
    /// use tokio::sync::mpsc;
    /// # use vtcode_ui::tui::{Session, InlineTheme};
    ///
    /// # let mut session = Session::new(InlineTheme::default(), None, 20, false);
    /// let (events_tx, events_rx) = mpsc::unbounded_channel();
    ///
    /// loop {
    ///     if let Ok(evt) = event::read() {
    ///         session.handle_event(evt, &events_tx, None);
    ///     }
    /// }
    /// ```
    pub fn handle_event(
        &mut self,
        event: CrosstermEvent,
        events: &UnboundedSender<InlineEvent>,
        callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
    ) {
        // Delegate to events module for all event handling
        events::handle_event(self, event, events, callback);
    }

    /// Render the session TUI to a frame.
    ///
    /// This is the main render coordinator that orchestrates the layout and rendering
    /// of all TUI components. It delegates to the `rendering` module for the actual
    /// rendering logic.
    ///
    /// The rendering system handles:
    /// - Dynamic layout calculation based on viewport size
    /// - Header, transcript, input area, and navigation pane
    /// - Overlays (palettes, modals, queue display)
    /// - Responsive behavior for different terminal sizes
    /// - Caching and optimization for smooth scrolling
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    ///
    /// # Layout
    ///
    /// The screen is divided into these areas (top to bottom):
    /// 1. **Header** - Session info, model, metadata (1-3 lines)
    /// 2. **Main area** - Split into:
    ///    - **Transcript** - Conversation history (scrollable)
    ///    - **Navigation** - Timeline/plan pane (if enabled)
    /// 3. **Input** - User input area (1-3 lines + optional status)
    ///
    /// Overlays (palettes, modals) are rendered on top of these base components.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ratatui::Terminal;
    /// # use vtcode_ui::tui::{Session, InlineTheme};
    /// # let mut session = Session::new(InlineTheme::default(), None, 20, false);
    /// # let mut terminal: Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> = todo!();
    /// terminal.draw(|frame| {
    ///     session.render(frame);
    /// })?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn render(&mut self, frame: &mut Frame) {
        // Delegate to rendering module for all rendering logic
        rendering::render(self, frame);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let theme = InlineTheme::default();
        let session = Session::new(theme, Some("Test".to_string()), 20, false);

        assert!(!session.should_exit());
        assert_eq!(session.ui_state.view_rows, 20);
        assert!(!session.ui_state.show_timeline_pane);
    }

    #[test]
    fn test_exit_flow() {
        let theme = InlineTheme::default();
        let mut session = Session::new(theme, None, 20, false);

        assert!(!session.should_exit());
        session.request_exit();
        assert!(session.should_exit());
    }

    #[test]
    fn test_redraw_flag() {
        let theme = InlineTheme::default();
        let mut session = Session::new(theme, None, 20, false);

        // New session should need a redraw
        assert!(session.take_redraw());
        // Second call should return false (flag consumed)
        assert!(!session.take_redraw());

        // Request another redraw
        session.ui_state.request_redraw();
        assert!(session.take_redraw());
        assert!(!session.take_redraw());
    }
}
