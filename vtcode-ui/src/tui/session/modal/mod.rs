//! Modal dialog system for vtcode-ui
//!
//! This module provides a comprehensive modal dialog system with support for:
//! - Text modals
//! - List modals with search and filtering
//! - Secure password prompts
//! - Keyboard navigation
//! - Custom styling
//!
//! ## Module Organization
//!
//! - `state`: Modal state types and configuration
//! - `list`: List navigation and selection logic
//! - `search`: Search and filtering functionality
//! - `keyboard`: Keyboard event handling
//! - `rendering`: Modal rendering functions
//! - `layout`: Layout calculations and positioning
//! - `secure_prompt`: Secure input handling

// Module declarations
mod keyboard;
mod layout;
mod list;
mod rendering;
mod search;
mod secure_prompt;
mod state;

// Re-export public API
pub use layout::{compute_modal_area, modal_content_width, ModalListLayout};
pub use rendering::{render_modal_body, ModalBodyContext, ModalRenderStyles};
pub use state::{
    ModalKeyModifiers, ModalListKeyResult, ModalListState, ModalSearchState, ModalState,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::types::{InlineEvent, InlineListItem, InlineListSearchConfig, InlineListSelection};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::{Modifier, Style};
    use rendering::highlight_segments;
    use tui_popup::PopupState;
    use vtcode_config::constants::ui;

    fn base_item(title: &str) -> InlineListItem {
        InlineListItem {
            title: title.to_string(),
            subtitle: None,
            badge: None,
            indent: 0,
            selection: None,
            search_value: None,
        }
    }

    fn sample_list_modal() -> ModalState {
        let items = vec![
            InlineListItem {
                title: "First".to_string(),
                selection: Some(InlineListSelection::Model(0)),
                search_value: Some("general".to_string()),
                ..base_item("First")
            },
            InlineListItem {
                title: "Second".to_string(),
                selection: Some(InlineListSelection::Model(1)),
                search_value: Some("other".to_string()),
                ..base_item("Second")
            },
        ];

        let list_state = ModalListState::new(items, None);
        let search_state = ModalSearchState::from(InlineListSearchConfig {
            label: "Search".to_string(),
            placeholder: None,
        });

        let mut modal = ModalState {
            title: "Test".to_string(),
            lines: vec![],
            list: Some(list_state),
            secure_prompt: None,
            popup_state: PopupState::default(),
            restore_input: true,
            restore_cursor: true,
            search: Some(search_state),
        };

        if let Some(list) = modal.list.as_mut() {
            let query = modal
                .search
                .as_ref()
                .map(|state| state.query.clone())
                .unwrap_or_default();
            list.apply_search(&query);
        }

        modal
    }

    fn sample_list_modal_with_count(count: usize) -> ModalState {
        let items = (0..count)
            .map(|index| {
                let label = format!("Item {}", index + 1);
                InlineListItem {
                    selection: Some(InlineListSelection::Model(index)),
                    search_value: Some(label.to_ascii_lowercase()),
                    ..base_item(&label)
                }
            })
            .collect::<Vec<_>>();

        ModalState {
            title: "Test".to_string(),
            lines: vec![],
            list: Some(ModalListState::new(items, None)),
            secure_prompt: None,
            popup_state: PopupState::default(),
            restore_input: true,
            restore_cursor: true,
            search: None,
        }
    }

    #[test]
    fn apply_search_retains_related_structure() {
        let divider = InlineListItem {
            title: ui::INLINE_USER_MESSAGE_DIVIDER_SYMBOL.repeat(3),
            ..base_item("")
        };
        let header = InlineListItem {
            search_value: Some("General Models".to_string()),
            ..base_item("Models")
        };
        let matching = InlineListItem {
            indent: 1,
            selection: Some(InlineListSelection::Model(0)),
            search_value: Some("general purpose".to_string()),
            ..base_item("General Purpose")
        };
        let non_matching = InlineListItem {
            selection: Some(InlineListSelection::Model(1)),
            search_value: Some("specialized".to_string()),
            ..base_item("Specialized")
        };

        let mut state = ModalListState::new(vec![divider, header, matching, non_matching], None);

        state.apply_search("general");

        let visible_titles: Vec<String> = state
            .visible_indices
            .iter()
            .map(|&idx| state.items[idx].title.clone())
            .collect();

        let expected_divider = ui::INLINE_USER_MESSAGE_DIVIDER_SYMBOL.repeat(3);
        assert_eq!(
            visible_titles,
            vec![
                expected_divider,
                "Models".to_string(),
                "General Purpose".to_string(),
                "Specialized".to_string()
            ]
        );
        assert_eq!(state.visible_selectable_count(), 2);
        assert_eq!(state.filter_query(), Some("general"));

        state.apply_search("");
        assert_eq!(state.visible_indices.len(), state.items.len());
        assert!(state.filter_query().is_none());
    }

    #[test]
    fn highlight_segments_marks_matching_spans() {
        let segments = highlight_segments(
            "Hello",
            Style::default(),
            Style::default().add_modifier(Modifier::BOLD),
            &["el".to_string()],
        );

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].content.as_ref(), "H");
        assert_eq!(segments[0].style, Style::default());
        assert_eq!(segments[1].content.as_ref(), "el");
        assert_eq!(
            segments[1].style,
            Style::default().add_modifier(Modifier::BOLD)
        );
        assert_eq!(segments[2].content.as_ref(), "lo");
        assert_eq!(segments[2].style, Style::default());
    }

    #[test]
    fn list_modal_handles_search_typing() {
        let mut modal = sample_list_modal();
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        match result {
            ModalListKeyResult::Redraw => {}
            other => panic!("expected redraw, got {:?}", other),
        }

        let query = &modal.search.unwrap().query;
        assert_eq!(query, "g");
    }

    #[test]
    fn list_modal_submit_emits_event() {
        let mut modal = sample_list_modal();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        match result {
            ModalListKeyResult::Submit(InlineEvent::ListModalSubmit(selection)) => {
                assert_eq!(selection, InlineListSelection::Model(0));
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn list_modal_cancel_emits_event() {
        let mut modal = sample_list_modal();
        if let Some(search) = modal.search.as_mut() {
            search.query = "value".to_string();
        }

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        match result {
            ModalListKeyResult::Redraw => {}
            other => panic!("expected redraw to clear query first, got {:?}", other),
        }

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        match result {
            ModalListKeyResult::Cancel(InlineEvent::ListModalCancel) => {}
            other => panic!("expected cancel event, got {:?}", other),
        }
    }

    #[test]
    fn list_modal_tab_moves_forward() {
        let mut modal = sample_list_modal();
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        assert!(matches!(result, ModalListKeyResult::Redraw));
        let selection = modal
            .list
            .as_ref()
            .and_then(|list| list.current_selection());
        assert_eq!(selection, Some(InlineListSelection::Model(1)));
    }

    #[test]
    fn list_modal_backtab_moves_backward() {
        let mut modal = sample_list_modal();
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let _ = modal.handle_list_key_event(&down, ModalKeyModifiers::default());

        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        let result = modal.handle_list_key_event(&key, ModalKeyModifiers::default());

        assert!(matches!(result, ModalListKeyResult::Redraw));
        let selection = modal
            .list
            .as_ref()
            .and_then(|list| list.current_selection());
        assert_eq!(selection, Some(InlineListSelection::Model(0)));
    }

    #[test]
    fn list_modal_control_navigation_moves_selection() {
        let mut modal = sample_list_modal();
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let _ = modal.handle_list_key_event(&tab, ModalKeyModifiers::default());

        let ctrl_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let result = modal.handle_list_key_event(
            &ctrl_p,
            ModalKeyModifiers {
                control: true,
                alt: false,
                command: false,
            },
        );

        assert!(matches!(result, ModalListKeyResult::Redraw));
        let selection = modal
            .list
            .as_ref()
            .and_then(|list| list.current_selection());
        assert_eq!(selection, Some(InlineListSelection::Model(0)));
    }

    #[test]
    fn list_search_preserves_selection_when_item_matches() {
        let mut modal = sample_list_modal();
        let list = modal.list.as_mut().expect("list state");
        list.select_next();

        let previous = list.current_selection();
        list.apply_search("other");

        assert_eq!(list.current_selection(), previous);
    }

    #[test]
    fn list_search_resets_selection_when_item_removed() {
        let mut modal = sample_list_modal();
        let list = modal.list.as_mut().expect("list state");
        list.select_next();

        list.apply_search("general");

        assert_eq!(
            list.current_selection(),
            Some(InlineListSelection::Model(0))
        );
    }

    #[test]
    fn list_modal_page_navigation_respects_viewport() {
        let mut modal = sample_list_modal_with_count(6);
        let list = modal.list.as_mut().expect("list state");
        list.set_viewport_rows(3);

        let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&page_down, ModalKeyModifiers::default());
        assert!(matches!(result, ModalListKeyResult::Redraw));

        let selection = modal
            .list
            .as_ref()
            .and_then(|state| state.current_selection());
        assert_eq!(selection, Some(InlineListSelection::Model(3)));

        let page_up = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        let result = modal.handle_list_key_event(&page_up, ModalKeyModifiers::default());
        assert!(matches!(result, ModalListKeyResult::Redraw));

        let selection = modal
            .list
            .as_ref()
            .and_then(|state| state.current_selection());
        assert_eq!(selection, Some(InlineListSelection::Model(0)));
    }
}
