//! Keyboard event handling for the session.
//!
//! This module contains all keyboard input processing logic, including:
//! - Main key event handler with modifier key detection
//! - Input manipulation (insert, delete, move cursor)
//! - History navigation (previous/next commands)
//! - Word and sentence-based editing operations
//! - Modal, palette, and slash palette priority handling
//!
//! The keyboard handler respects the following priority order:
//! 1. Modal dialogs (if open)
//! 2. File palette (if active)
//! 3. Prompt palette (if active)
//! 4. Slash palette navigation (if available)
//! 5. Regular input handling

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;
use vtcode_config::constants::ui;

use crate::tui::types::InlineEvent;
use super::super::Session;
use super::super::file_palette::extract_file_reference;
use super::super::modal::ModalKeyModifiers;
use super::super::modal::ModalListKeyResult;

/// Handle keyboard events for the session.
///
/// This is the main entry point for processing keyboard input. It handles:
/// - Modifier key detection (Control, Alt, Command/Meta, Shift)
/// - Modal dialogs and palettes with priority
/// - Special key combinations (Ctrl+C, Ctrl+D, Esc, etc.)
/// - Text input and editing operations
/// - History navigation
/// - Scrolling
///
/// # Arguments
/// * `session` - The session state to modify
/// * `key` - The keyboard event to process
///
/// # Returns
/// An optional `InlineEvent` if the key press should trigger an action
pub fn handle_key_event(session: &mut Session, key: KeyEvent) -> Option<InlineEvent> {
    let modifiers = key.modifiers;
    let has_control = modifiers.contains(KeyModifiers::CONTROL);
    let has_shift = modifiers.contains(KeyModifiers::SHIFT);
    let raw_alt = modifiers.contains(KeyModifiers::ALT);
    let raw_meta = modifiers.contains(KeyModifiers::META);
    let has_super = modifiers.contains(KeyModifiers::SUPER);
    let has_alt = raw_alt || (!has_super && raw_meta);
    let has_command = has_super || (raw_meta && !has_alt);

    // Handle modal dialogs first (highest priority)
    if let Some(modal) = session.render_state.modal.as_mut() {
        let result = modal.handle_list_key_event(
            &key,
            ModalKeyModifiers {
                control: has_control,
                alt: has_alt,
                command: has_command,
            },
        );

        match result {
            ModalListKeyResult::Redraw => {
                session.mark_dirty();
                return None;
            }
            ModalListKeyResult::HandledNoRedraw => {
                return None;
            }
            ModalListKeyResult::Submit(event) | ModalListKeyResult::Cancel(event) => {
                session.close_modal();
                return Some(event);
            }
            ModalListKeyResult::NotHandled => {}
        }
    }

    // Handle file palette (second priority)
    if session.handle_file_palette_key(&key) {
        return None;
    }

    // Handle prompt palette (third priority)
    if session.handle_prompt_palette_key(&key) {
        return None;
    }

    // Handle slash palette navigation (fourth priority)
    if session.try_handle_slash_navigation(&key, has_control, has_alt, has_command) {
        return None;
    }

    // Handle regular key events
    match key.code {
        // Interrupt (Ctrl+C or raw ^C character)
        KeyCode::Char('c') | KeyCode::Char('C') if has_control => {
            session.mark_dirty();
            Some(InlineEvent::Interrupt)
        }
        KeyCode::Char(c) if c == '\u{0003}' => {
            session.mark_dirty();
            Some(InlineEvent::Interrupt)
        }

        // Exit (Ctrl+D)
        KeyCode::Char('d') if has_control => {
            session.mark_dirty();
            Some(InlineEvent::Exit)
        }

        // Escape key
        KeyCode::Esc => {
            if session.render_state.modal.is_some() {
                session.close_modal();
                None
            } else {
                // Handle double escape to clear input
                let is_double_escape = session.input_manager.check_escape_double_tap();

                if is_double_escape && !session.input_manager.content().is_empty() {
                    // Double escape detected - clear the input
                    session.clear_input();
                    session.mark_dirty();
                    None // Don't send an event, just clear the input
                } else {
                    // Single escape - send cancel event
                    session.mark_dirty();
                    Some(InlineEvent::Cancel)
                }
            }
        }

        // Page Up/Down for scrolling
        KeyCode::PageUp => {
            super::mouse::scroll_page_up(session);
            session.mark_dirty();
            Some(InlineEvent::ScrollPageUp)
        }
        KeyCode::PageDown => {
            super::mouse::scroll_page_down(session);
            session.mark_dirty();
            Some(InlineEvent::ScrollPageDown)
        }

        // Up arrow - history or scroll
        KeyCode::Up => {
            let history_requested = if session.ui_state.input_enabled && (has_alt || has_command) {
                true
            } else if session.ui_state.input_enabled {
                session.current_max_scroll_offset() == 0
            } else {
                false
            };

            if history_requested && navigate_history_previous(session) {
                return None;
            }

            // Only scroll transcript if not navigating history
            super::mouse::scroll_line_up(session);
            session.mark_dirty();
            Some(InlineEvent::ScrollLineUp)
        }

        // Down arrow - history or scroll
        KeyCode::Down => {
            let history_requested = if session.ui_state.input_enabled && (has_alt || has_command) {
                true
            } else if session.ui_state.input_enabled {
                session.current_max_scroll_offset() == 0
            } else {
                false
            };

            if history_requested && navigate_history_next(session) {
                return None;
            }

            // Only scroll transcript if not navigating history
            super::mouse::scroll_line_down(session);
            session.mark_dirty();
            Some(InlineEvent::ScrollLineDown)
        }

        // Enter - submit input or insert newline with Shift
        KeyCode::Enter => {
            if !session.ui_state.input_enabled {
                return None;
            }

            // Handle file palette selection
            if session.palette_state.file_palette_active {
                if let Some(palette) = session.palette_state.file_palette.as_ref() {
                    if let Some(entry) = palette.get_selected() {
                        let file_path = entry.path.clone();
                        insert_file_reference(session, &file_path);
                        session.close_file_palette();
                        session.mark_dirty();
                        return Some(InlineEvent::FileSelected(file_path));
                    }
                }
                return None;
            }

            // Shift+Enter inserts a newline
            if has_shift && !has_control && !has_command {
                insert_char(session, '\n');
                session.mark_dirty();
                return None;
            }

            // Submit input
            let submitted = session.input_manager.content().to_string();
            session.input_manager.clear();
            session.scroll_manager.set_offset(0);
            session.update_slash_suggestions();

            // Don't submit empty input
            if submitted.trim().is_empty() {
                session.mark_dirty();
                return None;
            }

            remember_submitted_input(session, &submitted);
            session.mark_dirty();

            if has_control || has_command {
                Some(InlineEvent::QueueSubmit(submitted))
            } else {
                Some(InlineEvent::Submit(submitted))
            }
        }

        // Backspace - delete character, word, or sentence
        KeyCode::Backspace => {
            if session.ui_state.input_enabled {
                if has_alt {
                    // Alt+Backspace (Option+Backspace on Mac) - delete word backwards
                    delete_word_backward(session);
                } else if has_command {
                    // Command+Backspace (Mac) - delete sentence backwards
                    delete_sentence_backward(session);
                } else {
                    // Standard Backspace - backward delete of single character
                    delete_char(session);
                }
                session.check_file_reference_trigger();
                session.check_prompt_reference_trigger();
                session.mark_dirty();
            }
            None
        }

        // Delete - forward delete character, word, or sentence
        KeyCode::Delete => {
            if session.ui_state.input_enabled {
                if has_alt {
                    // Alt+Delete (Option+Delete on Mac) - delete word backwards
                    delete_word_backward(session);
                } else if has_command {
                    // Command+Delete (Mac) - delete sentence backwards
                    delete_sentence_backward(session);
                } else {
                    // Standard Delete - forward delete
                    delete_char_forward(session);
                }
                session.check_file_reference_trigger();
                session.check_prompt_reference_trigger();
                session.mark_dirty();
            }
            None
        }

        // Left arrow - move cursor
        KeyCode::Left => {
            if session.ui_state.input_enabled {
                if has_command {
                    move_to_start(session);
                } else if has_alt {
                    move_left_word(session);
                } else {
                    move_left(session);
                }
                session.mark_dirty();
            }
            None
        }

        // Right arrow - move cursor
        KeyCode::Right => {
            if session.ui_state.input_enabled {
                if has_command {
                    move_to_end(session);
                } else if has_alt {
                    move_right_word(session);
                } else {
                    move_right(session);
                }
                session.mark_dirty();
            }
            None
        }

        // Home - move to start
        KeyCode::Home => {
            if session.ui_state.input_enabled {
                move_to_start(session);
                session.mark_dirty();
            }
            None
        }

        // End - move to end
        KeyCode::End => {
            if session.ui_state.input_enabled {
                move_to_end(session);
                session.mark_dirty();
            }
            None
        }

        // Ctrl+T - toggle timeline pane
        KeyCode::Char('t') | KeyCode::Char('T') if has_control => {
            session.ui_state.show_timeline_pane = !session.ui_state.show_timeline_pane;
            session.mark_dirty();
            None
        }

        // Character input
        KeyCode::Char(ch) => {
            if !session.ui_state.input_enabled {
                return None;
            }

            // Command key shortcuts
            if has_command {
                match ch {
                    'a' | 'A' => {
                        move_to_start(session);
                        session.mark_dirty();
                        return None;
                    }
                    'e' | 'E' => {
                        move_to_end(session);
                        session.mark_dirty();
                        return None;
                    }
                    _ => {
                        return None;
                    }
                }
            }

            // Alt key shortcuts
            if has_alt {
                match ch {
                    'b' | 'B' => {
                        move_left_word(session);
                        session.mark_dirty();
                    }
                    'f' | 'F' => {
                        move_right_word(session);
                        session.mark_dirty();
                    }
                    _ => {}
                }
                return None;
            }

            // Regular character input (no Control modifier)
            if !has_control {
                insert_char(session, ch);
                session.check_file_reference_trigger();
                session.check_prompt_reference_trigger();
                session.mark_dirty();
            }
            None
        }

        _ => None,
    }
}

// ============================================================================
// Input Manipulation Helpers
// ============================================================================

/// Insert a single character at the cursor position.
fn insert_char(session: &mut Session, ch: char) {
    if ch == '\u{7f}' {
        return;
    }
    if ch == '\n' && !can_insert_newline(session) {
        return;
    }
    session.input_manager.insert_char(ch);
    session.update_slash_suggestions();
}

/// Insert a file reference at the current cursor position.
///
/// Replaces the current file reference pattern with the full path.
fn insert_file_reference(session: &mut Session, file_path: &str) {
    if let Some((start, end, _)) =
        extract_file_reference(session.input_manager.content(), session.input_manager.cursor())
    {
        let replacement = format!("@{}", file_path);
        let content = session.input_manager.content().to_string();
        let mut new_content = String::new();
        new_content.push_str(&content[..start]);
        new_content.push_str(&replacement);
        new_content.push_str(&content[end..]);
        session.input_manager.set_content(new_content);
        session.input_manager.set_cursor(start + replacement.len());
        session.input_manager.insert_char(' ');
    }
}

/// Calculate remaining newline capacity based on max lines.
fn remaining_newline_capacity(session: &Session) -> usize {
    ui::INLINE_INPUT_MAX_LINES
        .saturating_sub(1)
        .saturating_sub(session.input_manager.content().matches('\n').count())
}

/// Check if a newline can be inserted.
fn can_insert_newline(session: &Session) -> bool {
    remaining_newline_capacity(session) > 0
}

/// Delete the character before the cursor (backspace).
fn delete_char(session: &mut Session) {
    session.input_manager.backspace();
    session.update_slash_suggestions();
}

/// Delete the character after the cursor (forward delete).
fn delete_char_forward(session: &mut Session) {
    session.input_manager.delete();
    session.update_slash_suggestions();
}

/// Delete from the cursor backward to the start of the current word.
///
/// This operation skips trailing whitespace, then deletes non-whitespace
/// characters until it hits whitespace or the start of the input.
fn delete_word_backward(session: &mut Session) {
    if session.input_manager.cursor() == 0 {
        return;
    }

    // Find the start of the current word by moving backward (same logic as move_left_word)
    let graphemes: Vec<(usize, &str)> = session.input_manager.content()
        [..session.input_manager.cursor()]
        .grapheme_indices(true)
        .collect();

    if graphemes.is_empty() {
        return;
    }

    let mut index = graphemes.len();

    // Skip any trailing whitespace
    while index > 0 {
        let (_, grapheme) = graphemes[index - 1];
        if grapheme.chars().all(char::is_whitespace) {
            index -= 1;
        } else {
            break;
        }
    }

    // Move backwards until we find whitespace (start of the word)
    while index > 0 {
        let (_, grapheme) = graphemes[index - 1];
        if grapheme.chars().all(char::is_whitespace) {
            break;
        }
        index -= 1;
    }

    // Calculate the position to delete from
    let delete_start = if index < graphemes.len() {
        graphemes[index].0
    } else {
        0
    };

    // Delete from delete_start to cursor
    if delete_start < session.input_manager.cursor() {
        let content = session.input_manager.content().to_string();
        let mut new_content = String::new();
        new_content.push_str(&content[..delete_start]);
        new_content.push_str(&content[session.input_manager.cursor()..]);
        session.input_manager.set_content(new_content);
        session.input_manager.set_cursor(delete_start);
        session.update_slash_suggestions();
    }
}

/// Delete from the cursor backward to the start of the current sentence.
///
/// A sentence is defined as text ending with `.`, `!`, or `?` followed by
/// whitespace, or separated by newlines.
fn delete_sentence_backward(session: &mut Session) {
    if session.input_manager.cursor() == 0 {
        return;
    }

    let input_before_cursor = &session.input_manager.content()[..session.input_manager.cursor()];
    let chars: Vec<(usize, char)> = input_before_cursor.char_indices().collect();

    if chars.is_empty() {
        return;
    }

    // Look backwards from cursor for the most recent sentence ending followed by whitespace
    // A sentence typically ends with ., !, ? followed by space, tab, newline or end of input
    let mut delete_start = 0;

    // Search backwards to find the most recent sentence boundary
    for i in (0..chars.len()).rev() {
        let (pos, ch) = chars[i];

        if matches!(ch, '.' | '!' | '?') {
            // Check if this punctuation is followed by whitespace or we're at the end
            // Since we're looking at input before cursor, we check the original full input
            if pos + ch.len_utf8() < session.input_manager.content().len() {
                // Check the character after the punctuation in the full input string
                let after_punct = &session.input_manager.content()
                    [pos + ch.len_utf8()..session.input_manager.cursor()];
                if !after_punct.is_empty() {
                    let next_char = after_punct.chars().next().unwrap();
                    if next_char.is_whitespace() {
                        // Found sentence ending punctuation followed by whitespace
                        delete_start = pos + ch.len_utf8();
                        break;
                    }
                } else {
                    // At the end of the text being considered (before cursor)
                    // This might be a sentence boundary if there's whitespace after cursor
                    delete_start = pos + ch.len_utf8();
                    break;
                }
            } else {
                // At the end of the entire input string
                delete_start = pos + ch.len_utf8();
                break;
            }
        } else if matches!(ch, '\n' | '\r') {
            // Newlines can also separate sentences
            delete_start = pos + ch.len_utf8();
            break;
        }
    }

    // Delete from delete_start to cursor
    if delete_start < session.input_manager.cursor() {
        let content = session.input_manager.content().to_string();
        let mut new_content = String::new();
        new_content.push_str(&content[..delete_start]);
        new_content.push_str(&content[session.input_manager.cursor()..]);
        session.input_manager.set_content(new_content);
        session.input_manager.set_cursor(delete_start);
        session.update_slash_suggestions();
    }
}

// ============================================================================
// History Navigation
// ============================================================================

/// Save the submitted input to history.
fn remember_submitted_input(session: &mut Session, submitted: &str) {
    session.input_manager.add_to_history(submitted.to_string());
}

/// Navigate to the previous entry in command history.
///
/// Returns true if navigation occurred, false otherwise.
fn navigate_history_previous(session: &mut Session) -> bool {
    if let Some(entry) = session.input_manager.go_to_previous_history() {
        session.input_manager.set_content(entry);
        session.mark_dirty();
        true
    } else {
        false
    }
}

/// Navigate to the next entry in command history.
///
/// Returns true if navigation occurred, false otherwise.
fn navigate_history_next(session: &mut Session) -> bool {
    if let Some(entry) = session.input_manager.go_to_next_history() {
        session.input_manager.set_content(entry);
        session.mark_dirty();
        true
    } else {
        false
    }
}

// ============================================================================
// Cursor Movement
// ============================================================================

/// Move cursor one position to the left.
fn move_left(session: &mut Session) {
    session.input_manager.move_cursor_left();
    session.update_slash_suggestions();
}

/// Move cursor one position to the right.
fn move_right(session: &mut Session) {
    session.input_manager.move_cursor_right();
    session.update_slash_suggestions();
}

/// Move cursor to the start of the previous word.
///
/// Skips trailing whitespace, then moves to the start of the word.
fn move_left_word(session: &mut Session) {
    if session.input_manager.cursor() == 0 {
        return;
    }

    let graphemes: Vec<(usize, &str)> = session.input_manager.content()
        [..session.input_manager.cursor()]
        .grapheme_indices(true)
        .collect();

    if graphemes.is_empty() {
        session.input_manager.set_cursor(0);
        return;
    }

    let mut index = graphemes.len();

    // Skip trailing whitespace
    while index > 0 {
        let (_, grapheme) = graphemes[index - 1];
        if grapheme.chars().all(char::is_whitespace) {
            index -= 1;
        } else {
            break;
        }
    }

    // Move to start of word
    while index > 0 {
        let (_, grapheme) = graphemes[index - 1];
        if grapheme.chars().all(char::is_whitespace) {
            break;
        }
        index -= 1;
    }

    if index < graphemes.len() {
        session.input_manager.set_cursor(graphemes[index].0);
    } else {
        session.input_manager.set_cursor(0);
    }
}

/// Move cursor to the start of the next word.
///
/// Skips leading whitespace, then moves to the start of the next word.
fn move_right_word(session: &mut Session) {
    if session.input_manager.cursor() >= session.input_manager.content().len() {
        return;
    }

    let graphemes: Vec<(usize, &str)> = session.input_manager.content()
        [session.input_manager.cursor()..]
        .grapheme_indices(true)
        .collect();

    if graphemes.is_empty() {
        session.input_manager
            .set_cursor(session.input_manager.content().len());
        return;
    }

    let mut index = 0;
    let mut skipped_whitespace = false;

    // Skip leading whitespace
    while index < graphemes.len() {
        let (_, grapheme) = graphemes[index];
        if grapheme.chars().all(char::is_whitespace) {
            index += 1;
            skipped_whitespace = true;
        } else {
            break;
        }
    }

    if index >= graphemes.len() {
        session.input_manager
            .set_cursor(session.input_manager.content().len());
        return;
    }

    if skipped_whitespace {
        session.input_manager
            .set_cursor(session.input_manager.cursor() + graphemes[index].0);
        return;
    }

    // Move to end of current word
    while index < graphemes.len() {
        let (_, grapheme) = graphemes[index];
        if grapheme.chars().all(char::is_whitespace) {
            break;
        }
        index += 1;
    }

    if index < graphemes.len() {
        session.input_manager
            .set_cursor(session.input_manager.cursor() + graphemes[index].0);
    } else {
        session.input_manager
            .set_cursor(session.input_manager.content().len());
    }
}

/// Move cursor to the start of the input.
fn move_to_start(session: &mut Session) {
    session.input_manager.move_cursor_to_start();
}

/// Move cursor to the end of the input.
fn move_to_end(session: &mut Session) {
    session.input_manager.move_cursor_to_end();
}
