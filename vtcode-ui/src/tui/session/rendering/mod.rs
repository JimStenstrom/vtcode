//! Render coordinator for session TUI components
//!
//! This module serves as the central orchestrator for all session rendering operations.
//! It coordinates the layout and rendering of different UI components in a well-defined
//! order, ensuring consistent behavior and proper separation of concerns.
//!
//! # Architecture
//!
//! The rendering system follows a coordinator pattern:
//!
//! 1. **Main Coordinator** (`render()` function in this module)
//!    - Single entry point for all session rendering
//!    - Manages viewport dimensions and layout calculations
//!    - Handles deferred palette triggers (file/prompt browsers)
//!    - Orchestrates rendering order across all components
//!
//! 2. **Specialized Rendering Modules**
//!    - `transcript` - Conversation transcript with scrolling and reflow
//!    - `input_area` - User input with cursor and status indicators
//!    - `palettes` - File/prompt/slash palettes and modal dialogs
//!    - `header` - Session header with model info and metadata (in parent module)
//!    - `navigation` - Timeline/plan navigation pane (in parent module)
//!
//! 3. **Layout Calculation**
//!    - Dynamic header height based on content wrapping
//!    - Adaptive input area height (1-3 lines based on content)
//!    - Flexible transcript/navigation split (horizontal or vertical)
//!    - Responsive behavior for narrow terminals
//!
//! # Rendering Order
//!
//! Components are rendered in this sequence:
//! 1. Viewport setup and full clear (if modal was just closed)
//! 2. Deferred palette trigger handling
//! 3. View dimensions calculation
//! 4. Layout computation (header, transcript, navigation, input areas)
//! 5. Header rendering
//! 6. Navigation pane rendering (if enabled)
//! 7. Transcript rendering
//! 8. Input area rendering
//! 9. Palettes and modals rendering (overlay on top)
//!
//! # Module Organization
//!
//! - `transcript` - Conversation transcript rendering, scrolling, and reflow caching
//! - `input_area` - User input area rendering, cursor positioning, and status line
//! - `palettes` - Overlay UIs including file/prompt/slash palettes and modal dialogs

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::Clear,
};

use vtcode_config::constants::ui;

use super::Session;

// Specialized rendering modules
pub(super) mod input_area;
pub(super) mod palettes;
pub(super) mod transcript;

/// Main render coordinator for the session TUI.
///
/// This is the single entry point for all session rendering operations. It orchestrates
/// the layout calculation and delegates rendering to specialized modules in the correct
/// order.
///
/// # Layout Algorithm
///
/// The layout is calculated in multiple phases:
///
/// 1. **Viewport Validation** - Ensure non-zero dimensions
/// 2. **View Rows Application** - Update internal scroll metrics
/// 3. **Header Height Calculation** - Measure wrapped header content (1-3 lines)
/// 4. **Input Height Calculation** - Measure input content (1-3 lines + optional status)
/// 5. **Vertical Layout** - Split into header, main area, and input
/// 6. **Horizontal Layout** - Split main area into transcript and navigation (if enabled)
///
/// # Navigation Pane Behavior
///
/// When the timeline pane is enabled (`show_timeline_pane`):
/// - **Wide terminals** (≥ minimum width): Side-by-side horizontal layout
/// - **Narrow terminals** (< minimum width): Stacked vertical layout
///
/// # Deferred Triggers
///
/// Some palettes cannot be opened while modals are active. Deferred triggers allow
/// the user to request a palette (e.g., by typing `/` then `@`) while a modal is open.
/// The request is queued and executed after the modal dismisses.
///
/// # Arguments
///
/// * `session` - The session state containing all UI components and data
/// * `frame` - The ratatui frame to render into
///
/// # Panics
///
/// This function does not panic. It handles zero-dimension viewports gracefully.
pub fn render(session: &mut Session, frame: &mut Frame) {
    // Phase 1: Viewport setup and validation
    let viewport = frame.area();
    if viewport.height == 0 || viewport.width == 0 {
        return;
    }

    // Clear entire frame if modal was just closed to remove artifacts
    if session.ui_state.needs_full_clear {
        frame.render_widget(Clear, viewport);
        session.ui_state.needs_full_clear = false;
    }

    // Phase 2: Handle deferred palette triggers
    handle_deferred_triggers(session);

    // Phase 3: Update view dimensions and scroll metrics
    apply_view_rows(session, viewport.height);

    // Phase 4: Calculate dynamic header height
    let header_lines = session.header_lines();
    let header_height = session.header_height_from_lines(viewport.width, &header_lines);
    if header_height != session.render_state.header_rows {
        session.render_state.header_rows = header_height;
        session.recalculate_transcript_rows();
    }

    // Phase 5: Calculate dynamic input height
    let input_height = calculate_input_height(session, viewport.width);
    session.apply_input_height(input_height);

    // Phase 6: Compute vertical layout (header, main, input)
    let (header_area, main_area, input_area) =
        calculate_vertical_layout(viewport, header_height, input_height);

    // Phase 7: Compute horizontal layout (transcript, navigation)
    let (transcript_area, navigation_area) =
        calculate_horizontal_layout(session, main_area);

    // Phase 8: Render all components in order
    session.render_header(frame, header_area, &header_lines);

    if session.ui_state.show_timeline_pane {
        session.render_navigation(frame, navigation_area);
    }

    session.render_transcript(frame, transcript_area);
    session.render_input(frame, input_area);
    session.render_palettes_and_modals(frame, viewport);
}

/// Handles deferred palette triggers that were queued while modals were active.
///
/// When a user types a palette trigger (like `@` for files or `#` for prompts) while
/// a modal is open, the trigger is deferred. This function executes those deferred
/// triggers after the modal dismisses.
///
/// # Arguments
///
/// * `session` - The session state containing deferred trigger flags
fn handle_deferred_triggers(session: &mut Session) {
    // Handle deferred file browser trigger (after slash modal dismisses)
    if session.palette_state.deferred_file_browser_trigger {
        session.palette_state.deferred_file_browser_trigger = false;
        // Insert @ to trigger file browser now that slash modal is gone
        session.input_manager.insert_char('@');
        session.check_file_reference_trigger();
        session.mark_dirty(); // Ensure UI updates
    }

    // Handle deferred prompt browser trigger (after slash modal dismisses)
    if session.palette_state.deferred_prompt_browser_trigger {
        session.palette_state.deferred_prompt_browser_trigger = false;
        // Insert # to trigger prompt browser now that slash modal is gone
        session.input_manager.insert_char('#');
        session.check_prompt_reference_trigger();
        session.mark_dirty(); // Ensure UI updates
    }
}

/// Updates the session's view row count and invalidates scroll metrics if changed.
///
/// The view row count determines the viewport height for scrolling calculations.
/// When the terminal is resized, this function ensures scroll positions are
/// recalculated and bounds are enforced.
///
/// # Arguments
///
/// * `session` - The session state to update
/// * `rows` - The new viewport height in rows (minimum 2)
fn apply_view_rows(session: &mut Session, rows: u16) {
    let resolved = rows.max(2);
    if session.ui_state.view_rows != resolved {
        session.ui_state.view_rows = resolved;
        session.invalidate_scroll_metrics();
    }
    session.recalculate_transcript_rows();
    session.enforce_scroll_bounds();
}

/// Calculates the total height needed for the input area.
///
/// The input area consists of:
/// - Input block with borders (wraps based on content)
/// - Optional status line (git status, trust indicators, etc.)
///
/// The block height adapts from 1-3 lines based on input content, plus 1 line
/// for the status if present.
///
/// # Arguments
///
/// * `session` - The session state containing input content
/// * `viewport_width` - The total viewport width
///
/// # Returns
///
/// The total height in rows needed for the input area (block + status)
fn calculate_input_height(session: &Session, viewport_width: u16) -> u16 {
    let status_height = if viewport_width > 0 && session.has_input_status() {
        1
    } else {
        0
    };

    let inner_width = viewport_width.saturating_sub(2);
    let desired_lines = session.desired_input_lines(inner_width);
    let block_height = Session::input_block_height_for_lines(desired_lines);

    block_height.saturating_add(status_height)
}

/// Calculates the vertical layout splitting viewport into header, main, and input areas.
///
/// The layout uses these constraints:
/// - Header: Fixed height based on wrapped content (1-3 lines)
/// - Main area: Flexible, takes remaining space (minimum 1 row)
/// - Input area: Fixed height based on content (1-3 lines + optional status)
///
/// # Arguments
///
/// * `viewport` - The full viewport rectangle
/// * `header_height` - The calculated header height in rows
/// * `input_height` - The calculated input height in rows
///
/// # Returns
///
/// A tuple of (header_area, main_area, input_area) rectangles
fn calculate_vertical_layout(
    viewport: Rect,
    header_height: u16,
    input_height: u16,
) -> (Rect, Rect, Rect) {
    let constraints = vec![
        Constraint::Length(header_height),
        Constraint::Min(1),
        Constraint::Length(input_height),
    ];

    let segments = Layout::vertical(constraints).split(viewport);

    let header_area = segments[0];
    let main_area = segments[1];
    let input_area = segments[2];

    (header_area, main_area, input_area)
}

/// Calculates the horizontal layout splitting main area into transcript and navigation.
///
/// This function implements responsive layout behavior:
///
/// **When navigation is disabled:**
/// - Transcript takes the entire main area
/// - Navigation area is zero-sized
///
/// **When navigation is enabled (wide terminals):**
/// - Side-by-side horizontal split
/// - Navigation width: Configured percentage (default 25%), clamped to min/max
/// - Transcript width: Remaining space (guaranteed minimum width)
///
/// **When navigation is enabled (narrow terminals):**
/// - Stacked vertical split
/// - Transcript height: Configured percentage (default 70%)
/// - Navigation height: Remaining percentage (default 30%)
///
/// # Arguments
///
/// * `session` - The session state containing navigation visibility flag
/// * `main_area` - The main area rectangle to split
///
/// # Returns
///
/// A tuple of (transcript_area, navigation_area) rectangles
fn calculate_horizontal_layout(session: &Session, main_area: Rect) -> (Rect, Rect) {
    if !session.ui_state.show_timeline_pane {
        // Navigation disabled - transcript takes full area
        return (main_area, Rect::new(main_area.x, main_area.y, 0, 0));
    }

    let available_width = main_area.width;
    let horizontal_minimum = ui::INLINE_CONTENT_MIN_WIDTH + ui::INLINE_NAVIGATION_MIN_WIDTH;

    if available_width >= horizontal_minimum {
        // Wide terminal - side-by-side horizontal layout
        let nav_percent = u32::from(ui::INLINE_NAVIGATION_PERCENT);
        let mut nav_width = ((available_width as u32 * nav_percent) / 100) as u16;

        // Clamp navigation width to minimum
        nav_width = nav_width.max(ui::INLINE_NAVIGATION_MIN_WIDTH);

        // Clamp navigation width to maximum (preserve transcript minimum)
        let max_allowed = available_width.saturating_sub(ui::INLINE_CONTENT_MIN_WIDTH);
        nav_width = nav_width.min(max_allowed);

        let constraints = [
            Constraint::Min(ui::INLINE_CONTENT_MIN_WIDTH),
            Constraint::Length(nav_width),
        ];
        let chunks = Layout::horizontal(constraints).split(main_area);

        (chunks[0], chunks[1])
    } else {
        // Narrow terminal - stacked vertical layout
        let nav_percent = ui::INLINE_STACKED_NAVIGATION_PERCENT.min(99);
        let transcript_percent = (100u16).saturating_sub(nav_percent).max(1);

        let constraints = [
            Constraint::Percentage(transcript_percent),
            Constraint::Percentage(nav_percent.max(1)),
        ];
        let chunks = Layout::vertical(constraints).split(main_area);

        (chunks[0], chunks[1])
    }
}
