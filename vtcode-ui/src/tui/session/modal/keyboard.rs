//! Modal keyboard event handling

use super::state::{ModalKeyModifiers, ModalListKeyResult, ModalState};
use crate::tui::types::InlineEvent;
use crossterm::event::{KeyCode, KeyEvent};

impl ModalState {
    /// Handle keyboard event for list modals
    pub fn handle_list_key_event(
        &mut self,
        key: &KeyEvent,
        modifiers: ModalKeyModifiers,
    ) -> ModalListKeyResult {
        let Some(list) = self.list.as_mut() else {
            return ModalListKeyResult::NotHandled;
        };

        if let Some(search) = self.search.as_mut() {
            match key.code {
                KeyCode::Char(ch) if !modifiers.control && !modifiers.alt && !modifiers.command => {
                    search.push_char(ch);
                    list.apply_search(&search.query);
                    return ModalListKeyResult::Redraw;
                }
                KeyCode::Backspace => {
                    if search.backspace() {
                        list.apply_search(&search.query);
                        return ModalListKeyResult::Redraw;
                    }
                    return ModalListKeyResult::HandledNoRedraw;
                }
                KeyCode::Delete => {
                    if search.clear() {
                        list.apply_search(&search.query);
                        return ModalListKeyResult::Redraw;
                    }
                    return ModalListKeyResult::HandledNoRedraw;
                }
                KeyCode::Esc => {
                    if search.clear() {
                        list.apply_search(&search.query);
                        return ModalListKeyResult::Redraw;
                    }
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Up => {
                if modifiers.command {
                    list.select_first();
                } else {
                    list.select_previous();
                }
                ModalListKeyResult::Redraw
            }
            KeyCode::Down => {
                if modifiers.command {
                    list.select_last();
                } else {
                    list.select_next();
                }
                ModalListKeyResult::Redraw
            }
            KeyCode::PageUp => {
                list.page_up();
                ModalListKeyResult::Redraw
            }
            KeyCode::PageDown => {
                list.page_down();
                ModalListKeyResult::Redraw
            }
            KeyCode::Home => {
                list.select_first();
                ModalListKeyResult::Redraw
            }
            KeyCode::End => {
                list.select_last();
                ModalListKeyResult::Redraw
            }
            KeyCode::Tab => {
                list.select_next();
                ModalListKeyResult::Redraw
            }
            KeyCode::BackTab => {
                list.select_previous();
                ModalListKeyResult::Redraw
            }
            KeyCode::Right => {
                list.select_next();
                ModalListKeyResult::Redraw
            }
            KeyCode::Enter => {
                if let Some(selection) = list.current_selection() {
                    ModalListKeyResult::Submit(InlineEvent::ListModalSubmit(selection))
                } else {
                    ModalListKeyResult::HandledNoRedraw
                }
            }
            KeyCode::Esc => ModalListKeyResult::Cancel(InlineEvent::ListModalCancel),
            KeyCode::Char(ch) if modifiers.control || modifiers.alt => match ch {
                'n' | 'N' | 'j' | 'J' => {
                    list.select_next();
                    ModalListKeyResult::Redraw
                }
                'p' | 'P' | 'k' | 'K' => {
                    list.select_previous();
                    ModalListKeyResult::Redraw
                }
                _ => ModalListKeyResult::NotHandled,
            },
            _ => ModalListKeyResult::NotHandled,
        }
    }
}
