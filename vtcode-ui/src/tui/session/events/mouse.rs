//! Mouse event handling for the session.
//!
//! This module contains all mouse input processing logic, including:
//! - Mouse event handler for scroll events
//! - Scroll operations (line up/down, page up/down)
//!
//! Currently, only scroll events are handled. Other mouse events like clicks
//! and drags are ignored but can be added in the future.

use crossterm::event::{MouseEvent, MouseEventKind};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui::types::InlineEvent;
use super::super::Session;

/// Handle mouse events for the session.
///
/// This is the main entry point for processing mouse input. Currently handles:
/// - Scroll down events (mouse wheel down)
/// - Scroll up events (mouse wheel up)
///
/// Other mouse events (clicks, drags, etc.) are currently ignored.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `event` - The mouse event to process
/// * `events` - Channel to send inline events
/// * `callback` - Optional callback to invoke with emitted events
///
/// # Returns
/// Returns Some(InlineEvent) if an event should be emitted, None otherwise.
/// The event is already sent through the channel and callback within this function.
pub fn handle_mouse_event(
    session: &mut Session,
    event: MouseEvent,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) -> Option<InlineEvent> {
    match event.kind {
        MouseEventKind::ScrollDown => {
            let event = handle_scroll_down(session, events, callback);
            Some(event)
        }
        MouseEventKind::ScrollUp => {
            let event = handle_scroll_up(session, events, callback);
            Some(event)
        }
        _ => None,
    }
}

/// Handles scroll down event from mouse input.
///
/// Scrolls the transcript down by one line, marks the session as needing
/// redraw, and emits the ScrollLineDown event.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `events` - Channel to send inline events
/// * `callback` - Optional callback to invoke with the event
///
/// # Returns
/// The InlineEvent that was emitted (ScrollLineDown)
#[inline]
fn handle_scroll_down(
    session: &mut Session,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) -> InlineEvent {
    scroll_line_down(session);
    session.mark_dirty();
    let event = InlineEvent::ScrollLineDown;
    emit_inline_event(&event, events, callback);
    event
}

/// Handles scroll up event from mouse input.
///
/// Scrolls the transcript up by one line, marks the session as needing
/// redraw, and emits the ScrollLineUp event.
///
/// # Arguments
/// * `session` - Mutable reference to the session state
/// * `events` - Channel to send inline events
/// * `callback` - Optional callback to invoke with the event
///
/// # Returns
/// The InlineEvent that was emitted (ScrollLineUp)
#[inline]
fn handle_scroll_up(
    session: &mut Session,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) -> InlineEvent {
    scroll_line_up(session);
    session.mark_dirty();
    let event = InlineEvent::ScrollLineUp;
    emit_inline_event(&event, events, callback);
    event
}

/// Scrolls the transcript view up by one line.
///
/// Updates the scroll manager's offset and marks the session for full clear
/// if the scroll position changed. This ensures proper rendering after scroll.
#[inline]
fn scroll_line_up(session: &mut Session) {
    let previous = session.scroll_manager.offset();
    session.scroll_manager.scroll_up(1);
    if session.scroll_manager.offset() != previous {
        session.needs_full_clear = true;
    }
}

/// Scrolls the transcript view down by one line.
///
/// Updates the scroll manager's offset and marks the session for full clear
/// if the scroll position changed. This ensures proper rendering after scroll.
#[inline]
fn scroll_line_down(session: &mut Session) {
    let previous = session.scroll_manager.offset();
    session.scroll_manager.scroll_down(1);
    if session.scroll_manager.offset() != previous {
        session.needs_full_clear = true;
    }
}

/// Scrolls the transcript view up by one page.
///
/// A page is defined as the current viewport height. Marks the session for
/// full clear if the scroll position changed.
#[inline]
pub fn scroll_page_up(session: &mut Session) {
    let previous = session.scroll_manager.offset();
    let viewport_height = session.viewport_height().max(1);
    session.scroll_manager.scroll_up(viewport_height);
    if session.scroll_manager.offset() != previous {
        session.needs_full_clear = true;
    }
}

/// Scrolls the transcript view down by one page.
///
/// A page is defined as the current viewport height. Marks the session for
/// full clear if the scroll position changed.
#[inline]
pub fn scroll_page_down(session: &mut Session) {
    let previous = session.scroll_manager.offset();
    let viewport_height = session.viewport_height().max(1);
    session.scroll_manager.scroll_down(viewport_height);
    if session.scroll_manager.offset() != previous {
        session.needs_full_clear = true;
    }
}

/// Emits an InlineEvent through the event channel and callback.
///
/// This helper consolidates the common pattern of:
/// 1. Calling the callback if present
/// 2. Sending the event through the channel
///
/// # Arguments
/// * `event` - The InlineEvent to emit
/// * `events` - The unbounded sender for the event channel
/// * `callback` - Optional callback to invoke with the event
#[inline]
fn emit_inline_event(
    event: &InlineEvent,
    events: &UnboundedSender<InlineEvent>,
    callback: Option<&(dyn Fn(&InlineEvent) + Send + Sync + 'static)>,
) {
    if let Some(cb) = callback {
        cb(event);
    }
    let _ = events.send(event.clone());
}
