//! Event handling for the session.
//!
//! This module provides a centralized event handling system for the TUI session,
//! dispatching different event types to specialized handlers:
//!
//! - **Keyboard events** - Handled by the `keyboard` module for key presses, modifiers,
//!   and keyboard shortcuts
//! - **Mouse events** - Handled by the `mouse` module for scroll events and other
//!   mouse interactions
//! - **Paste events** - Handled inline for text insertion into the input buffer
//! - **Resize events** - Handled inline to update the viewport dimensions
//!
//! The main entry point is `handle_event()`, which receives crossterm events and
//! delegates them to the appropriate handlers while managing the event callback
//! and channel system.

pub mod keyboard;
pub mod mouse;

use crossterm::event::{Event as CrosstermEvent, KeyEventKind, MouseEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::types::InlineEvent;
use super::Session;

/// Handle all types of events for the session.
///
/// This is the main event dispatcher that routes different event types to their
/// specialized handlers. The function processes:
///
/// - **Key events** - Only `Press` events are processed to avoid duplicate character
///   insertion from repeat events. Delegated to `keyboard::handle_key_event()`.
/// - **Mouse events** - Currently handles scroll up/down events. Delegated to
///   `mouse::handle_mouse_event()`.
/// - **Paste events** - Inserts pasted text into the input buffer or modal search,
///   checking for file/prompt reference triggers.
/// - **Resize events** - Updates the viewport height when the terminal is resized.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `event` - The crossterm event to process
/// * `events` - Channel to send inline events
/// * `callback` - Optional callback to invoke with emitted events
///
/// # Event Priority
/// The event handling respects this priority order:
/// 1. Modal dialogs (if open)
/// 2. File palette (if active)
/// 3. Prompt palette (if active)
/// 4. Slash palette navigation (if available)
/// 5. Regular input handling
///
/// # Examples
/// ```no_run
/// use crossterm::event::{self, Event};
/// use tokio::sync::mpsc;
///
/// let (events_tx, events_rx) = mpsc::unbounded_channel();
/// let mut session = Session::new(/* ... */);
///
/// // In your event loop:
/// if let Event::Key(key) = event::read()? {
///     handle_event(&mut session, Event::Key(key), &events_tx, None);
/// }
/// ```
pub fn handle_event(
    session: &mut Session,
    event: CrosstermEvent,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) {
    match event {
        CrosstermEvent::Key(key) => {
            // Only process Press events to avoid duplicate character insertion.
            // Repeat events can cause characters to be inserted multiple times.
            if matches!(key.kind, KeyEventKind::Press) {
                if let Some(outbound) = keyboard::handle_key_event(session, key) {
                    emit_inline_event(session, &outbound, events, callback);
                }
            }
        }
        CrosstermEvent::Mouse(mouse_event) => {
            // Delegate mouse event handling to the mouse module
            mouse::handle_mouse_event(session, mouse_event, events, callback);
        }
        CrosstermEvent::Paste(content) => {
            handle_paste_event(session, content);
        }
        CrosstermEvent::Resize(_, rows) => {
            handle_resize_event(session, rows);
        }
        _ => {
            // Ignore other event types (FocusGained, FocusLost, etc.)
        }
    }
}

/// Handles paste events by inserting text into the input buffer or modal search.
///
/// When input is enabled, the pasted content is inserted at the current cursor
/// position and checks are performed for file/prompt reference triggers.
///
/// When a modal is open with a searchable list, the content is inserted into
/// the search field and the list is filtered accordingly.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `content` - The pasted text content
#[inline]
fn handle_paste_event(session: &mut Session, content: String) {
    if session.input_enabled {
        session.insert_text(&content);
        session.check_file_reference_trigger();
        session.check_prompt_reference_trigger();
        session.mark_dirty();
    } else if let Some(modal) = session.modal.as_mut() {
        if let (Some(list), Some(search)) = (modal.list.as_mut(), modal.search.as_mut()) {
            search.insert(&content);
            list.apply_search(&search.query);
            session.mark_dirty();
        }
    }
}

/// Handles resize events by updating the viewport height.
///
/// When the terminal is resized, this updates the number of rows available
/// for the transcript view and marks the session as needing a redraw.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `rows` - The new number of rows in the terminal
#[inline]
fn handle_resize_event(session: &mut Session, rows: u16) {
    session.apply_view_rows(rows);
    session.mark_dirty();
}

/// Emits an InlineEvent through the event channel and callback.
///
/// This helper consolidates the common pattern of:
/// 1. Calling the callback if present
/// 2. Sending the event through the channel
///
/// This is used by the keyboard handler to emit events. The mouse handler
/// has its own internal emit function.
///
/// # Arguments
/// * `session` - Reference to the session (for consistency, though not currently used)
/// * `event` - The InlineEvent to emit
/// * `events` - The unbounded sender for the event channel
/// * `callback` - Optional callback to invoke with the event
#[inline]
fn emit_inline_event(
    _session: &Session,
    event: &InlineEvent,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) {
    if let Some(cb) = callback {
        cb(event);
    }
    let _ = events.send(event.clone());
}
