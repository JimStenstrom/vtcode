//! Core Session lifecycle and initialization
//!
//! This module contains the fundamental Session methods for:
//! - Construction and initialization
//! - Lifecycle management (exit, redraw)
//! - Utility helpers (revision counters, input height)

use vtcode_config::constants::ui;

use crate::tui::types::InlineTheme;

use super::input_manager::InputManager;
use super::scroll::ScrollManager;
use super::slash_palette::SlashPalette;
use super::state::{DisplayState, PaletteState, PromptState, RenderState, UIState};
use super::Session;

/// Prefix shown before user input
const USER_PREFIX: &str = "";

impl Session {
    /// Create a new Session instance
    ///
    /// Initializes all state components with appropriate default values based on
    /// the provided theme, placeholder text, viewport size, and timeline pane visibility.
    ///
    /// # Arguments
    ///
    /// * `theme` - The color theme for rendering
    /// * `placeholder` - Optional placeholder text shown when input is empty
    /// * `view_rows` - Total number of rows available in the viewport
    /// * `show_timeline_pane` - Whether to show the timeline pane
    ///
    /// # Returns
    ///
    /// A fully initialized Session ready for rendering and user interaction
    pub fn new(
        theme: InlineTheme,
        placeholder: Option<String>,
        view_rows: u16,
        show_timeline_pane: bool,
    ) -> Self {
        let resolved_rows = view_rows.max(2);
        let initial_header_rows = ui::INLINE_HEADER_HEIGHT;
        // Input block height: 1 line + 2 for borders = 3
        let initial_input_height = 3u16;
        let reserved_rows = initial_header_rows + initial_input_height;
        let initial_transcript_rows = resolved_rows.saturating_sub(reserved_rows).max(1);

        // Initialize state components
        let mut display_state = DisplayState::new();
        display_state.theme = theme;

        let mut prompt_state = PromptState::new();
        prompt_state.prompt_prefix = USER_PREFIX.to_string();
        prompt_state.placeholder = placeholder;

        let mut ui_state = UIState::new();
        ui_state.needs_redraw = true;
        ui_state.view_rows = resolved_rows;
        ui_state.input_height = initial_input_height;
        ui_state.transcript_rows = initial_transcript_rows;
        ui_state.show_timeline_pane = show_timeline_pane;

        let palette_state = PaletteState::new(SlashPalette::new());

        let mut render_state = RenderState::new();
        render_state.header_rows = initial_header_rows;

        let mut session = Self {
            // Managers
            input_manager: InputManager::new(),
            scroll_manager: ScrollManager::new(initial_transcript_rows),

            // State components
            display_state,
            prompt_state,
            ui_state,
            palette_state,
            render_state,
        };

        session.ensure_prompt_style_color();
        session
    }

    /// Check if the session should exit
    ///
    /// Returns true if an exit has been requested, indicating the event loop
    /// should terminate and the application should shut down.
    pub fn should_exit(&self) -> bool {
        self.ui_state.should_exit
    }

    /// Request the session to exit
    ///
    /// Sets the exit flag, which will be checked by the event loop to
    /// determine when to terminate the application.
    pub fn request_exit(&mut self) {
        self.ui_state.request_exit();
    }

    /// Check and consume the redraw flag
    ///
    /// Returns true if a redraw was requested and resets the flag.
    /// This should be called once per render cycle to determine if
    /// the terminal needs to be redrawn.
    ///
    /// # Returns
    ///
    /// `true` if a redraw is needed, `false` otherwise
    pub fn take_redraw(&mut self) -> bool {
        self.ui_state.consume_redraw_flag()
    }

    /// Get the next revision number for line updates
    ///
    /// Increments and returns the line revision counter, used for
    /// cache invalidation and tracking updates to message lines.
    ///
    /// # Returns
    ///
    /// The next revision number (wraps on overflow)
    pub(super) fn next_revision(&mut self) -> u64 {
        self.render_state.next_revision()
    }


    /// Ensure the prompt style has a color set
    ///
    /// If the prompt style doesn't have a color, sets it to the theme's
    /// primary color, falling back to the foreground color if no primary
    /// color is defined. This ensures the prompt is always visible.
    pub(super) fn ensure_prompt_style_color(&mut self) {
        if self.prompt_state.prompt_style.color.is_none() {
            self.prompt_state.prompt_style.color = self
                .display_state
                .theme
                .primary
                .or(self.display_state.theme.foreground);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session_initialization() {
        let theme = InlineTheme::default();
        let session = Session::new(theme.clone(), Some("Test".to_string()), 20, false);

        // Verify state initialization
        assert!(!session.should_exit());
        assert_eq!(session.prompt_state.prompt_prefix, "");
        assert_eq!(session.prompt_state.placeholder, Some("Test".to_string()));
        assert_eq!(session.ui_state.view_rows, 20);
        assert!(!session.ui_state.show_timeline_pane);
        assert!(session.ui_state.input_enabled);
        assert!(session.ui_state.cursor_visible);
    }

    #[test]
    fn test_exit_request() {
        let theme = InlineTheme::default();
        let mut session = Session::new(theme, None, 20, false);

        assert!(!session.should_exit());
        session.request_exit();
        assert!(session.should_exit());
    }

    #[test]
    fn test_take_redraw() {
        let theme = InlineTheme::default();
        let mut session = Session::new(theme, None, 20, false);

        // Initial state has redraw requested (from new())
        assert!(session.take_redraw());
        // Second call should return false (flag consumed)
        assert!(!session.take_redraw());

        // Request another redraw
        session.ui_state.request_redraw();
        assert!(session.take_redraw());
        assert!(!session.take_redraw());
    }

    #[test]
    fn test_next_revision() {
        let theme = InlineTheme::default();
        let mut session = Session::new(theme, None, 20, false);

        let rev1 = session.next_revision();
        let rev2 = session.next_revision();
        let rev3 = session.next_revision();

        assert_eq!(rev1, 1);
        assert_eq!(rev2, 2);
        assert_eq!(rev3, 3);
    }

    #[test]
    fn test_ensure_prompt_style_color() {
        let mut theme = InlineTheme::default();
        theme.primary = Some(anstyle::RgbColor(255, 0, 0));

        let session = Session::new(theme.clone(), None, 20, false);

        // Prompt style should have the primary color
        assert_eq!(session.prompt_state.prompt_style.color, theme.primary);
    }
}
