//! Modal state types and configuration

use crate::tui::types::{
    InlineEvent, InlineListSearchConfig, InlineListSelection, SecurePromptConfig,
};
use ratatui::widgets::ListState;
use tui_popup::PopupState;

/// Main modal state
#[derive(Clone)]
pub struct ModalState {
    pub title: String,
    pub lines: Vec<String>,
    pub list: Option<ModalListState>,
    pub secure_prompt: Option<SecurePromptConfig>,
    #[allow(dead_code)]
    pub popup_state: PopupState,
    pub restore_input: bool,
    pub restore_cursor: bool,
    pub search: Option<ModalSearchState>,
}

/// Key modifiers for modal keyboard handling
#[derive(Debug, Clone, Copy, Default)]
pub struct ModalKeyModifiers {
    pub control: bool,
    pub alt: bool,
    pub command: bool,
}

/// Result of handling keyboard events in modal lists
#[derive(Debug, Clone)]
pub enum ModalListKeyResult {
    NotHandled,
    HandledNoRedraw,
    Redraw,
    Submit(InlineEvent),
    Cancel(InlineEvent),
}

/// List modal state
#[derive(Clone)]
pub struct ModalListState {
    pub items: Vec<ModalListItem>,
    pub visible_indices: Vec<usize>,
    pub list_state: ListState,
    pub total_selectable: usize,
    pub filter_terms: Vec<String>,
    pub filter_query: Option<String>,
    pub viewport_rows: Option<u16>,
}

/// Individual item in a modal list
#[derive(Clone)]
pub struct ModalListItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
    pub indent: u8,
    pub selection: Option<InlineListSelection>,
    pub search_value: Option<String>,
    pub is_divider: bool,
}

/// Search state for list filtering
#[derive(Clone)]
pub struct ModalSearchState {
    pub label: String,
    pub placeholder: Option<String>,
    pub query: String,
}

impl From<InlineListSearchConfig> for ModalSearchState {
    fn from(config: InlineListSearchConfig) -> Self {
        Self {
            label: config.label,
            placeholder: config.placeholder,
            query: String::new(),
        }
    }
}

impl ModalSearchState {
    /// Insert text into search query, filtering out newlines
    pub fn insert(&mut self, value: &str) {
        for ch in value.chars() {
            if matches!(ch, '\n' | '\r') {
                continue;
            }
            self.query.push(ch);
        }
    }

    /// Add a single character to search query
    pub fn push_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    /// Remove last character from search query
    pub fn backspace(&mut self) -> bool {
        if self.query.pop().is_some() {
            return true;
        }
        false
    }

    /// Clear search query
    pub fn clear(&mut self) -> bool {
        if self.query.is_empty() {
            return false;
        }
        self.query.clear();
        true
    }
}

impl ModalListItem {
    /// Check if this item is a header (not selectable, not a divider)
    pub fn is_header(&self) -> bool {
        self.selection.is_none() && !self.is_divider
    }

    /// Check if this item matches a search query
    pub fn matches(&self, query: &str) -> bool {
        use crate::search::fuzzy_match;

        if query.is_empty() {
            return true;
        }
        let Some(value) = self.search_value.as_ref() else {
            return false;
        };
        fuzzy_match(query, value)
    }
}
