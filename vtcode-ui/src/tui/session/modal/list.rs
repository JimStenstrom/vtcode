//! List modal navigation and selection

use super::state::{ModalListItem, ModalListState};
use crate::tui::types::{InlineListItem, InlineListSelection};
use ratatui::widgets::ListState;
use vtcode_config::constants::ui;

/// Helper function to check if an item represents a divider
#[allow(clippy::const_is_empty)]
pub fn is_divider_title(item: &InlineListItem) -> bool {
    if item.selection.is_some() {
        return false;
    }
    if item.indent != 0 {
        return false;
    }
    if item.subtitle.is_some() || item.badge.is_some() {
        return false;
    }
    let symbol = ui::INLINE_USER_MESSAGE_DIVIDER_SYMBOL;
    if symbol.is_empty() {
        return false;
    }
    item.title
        .chars()
        .all(|ch| symbol.chars().any(|needle| needle == ch))
}

impl ModalListState {
    /// Create a new list state from inline list items
    pub fn new(items: Vec<InlineListItem>, selected: Option<InlineListSelection>) -> Self {
        let converted: Vec<ModalListItem> = items
            .into_iter()
            .map(|item| {
                let is_divider = is_divider_title(&item);
                let search_value = item
                    .search_value
                    .as_ref()
                    .map(|value| value.to_ascii_lowercase());
                ModalListItem {
                    title: item.title,
                    subtitle: item.subtitle,
                    badge: item.badge,
                    indent: item.indent,
                    selection: item.selection,
                    search_value,
                    is_divider,
                }
            })
            .collect();
        let total_selectable = converted
            .iter()
            .filter(|item| item.selection.is_some())
            .count();
        let mut modal_state = Self {
            visible_indices: (0..converted.len()).collect(),
            items: converted,
            list_state: ListState::default(),
            total_selectable,
            filter_terms: Vec::new(),
            filter_query: None,
            viewport_rows: None,
        };
        modal_state.select_initial(selected);
        modal_state
    }

    /// Get the currently selected item's selection value
    pub fn current_selection(&self) -> Option<InlineListSelection> {
        self.list_state
            .selected()
            .and_then(|index| self.visible_indices.get(index))
            .and_then(|&item_index| self.items.get(item_index))
            .and_then(|item| item.selection.clone())
    }

    /// Move selection to previous item
    pub fn select_previous(&mut self) {
        if self.visible_indices.is_empty() {
            return;
        }
        let Some(mut index) = self.list_state.selected() else {
            if let Some(last) = self.last_selectable_index() {
                self.list_state.select(Some(last));
            }
            return;
        };

        while index > 0 {
            index -= 1;
            let item_index = self.visible_indices[index];
            if self.items[item_index].selection.is_some() {
                self.list_state.select(Some(index));
                return;
            }
        }

        if let Some(first) = self.first_selectable_index() {
            self.list_state.select(Some(first));
        } else {
            self.list_state.select(None);
        }
    }

    /// Move selection to next item
    pub fn select_next(&mut self) {
        if self.visible_indices.is_empty() {
            return;
        }
        let mut index = self.list_state.selected().unwrap_or(usize::MAX);
        if index == usize::MAX {
            if let Some(first) = self.first_selectable_index() {
                self.list_state.select(Some(first));
            }
            return;
        }
        while index + 1 < self.visible_indices.len() {
            index += 1;
            let item_index = self.visible_indices[index];
            if self.items[item_index].selection.is_some() {
                self.list_state.select(Some(index));
                break;
            }
        }
    }

    /// Move selection to first selectable item
    pub fn select_first(&mut self) {
        if let Some(first) = self.first_selectable_index() {
            self.list_state.select(Some(first));
        } else {
            self.list_state.select(None);
        }
        if let Some(rows) = self.viewport_rows {
            self.ensure_visible(rows);
        }
    }

    /// Move selection to last selectable item
    pub fn select_last(&mut self) {
        if let Some(last) = self.last_selectable_index() {
            self.list_state.select(Some(last));
        } else {
            self.list_state.select(None);
        }
        if let Some(rows) = self.viewport_rows {
            self.ensure_visible(rows);
        }
    }

    /// Move selection up by one page
    pub fn page_up(&mut self) {
        let step = self.page_step();
        if step == 0 {
            self.select_previous();
            return;
        }
        for _ in 0..step {
            let before = self.list_state.selected();
            self.select_previous();
            if self.list_state.selected() == before {
                break;
            }
        }
    }

    /// Move selection down by one page
    pub fn page_down(&mut self) {
        let step = self.page_step();
        if step == 0 {
            self.select_next();
            return;
        }
        for _ in 0..step {
            let before = self.list_state.selected();
            self.select_next();
            if self.list_state.selected() == before {
                break;
            }
        }
    }

    /// Set viewport rows for pagination
    pub fn set_viewport_rows(&mut self, rows: u16) {
        self.viewport_rows = Some(rows);
    }

    /// Ensure selected item is visible within viewport
    pub(crate) fn ensure_visible(&mut self, viewport: u16) {
        let Some(selected) = self.list_state.selected() else {
            return;
        };
        if viewport == 0 {
            return;
        }
        let visible = viewport as usize;
        let offset = self.list_state.offset();
        if selected < offset {
            *self.list_state.offset_mut() = selected;
        } else if selected >= offset + visible {
            *self.list_state.offset_mut() = selected + 1 - visible;
        }
    }

    /// Select initial item, preferring the specified selection
    pub(crate) fn select_initial(&mut self, preferred: Option<InlineListSelection>) {
        let mut selection_index = preferred.and_then(|needle| {
            self.visible_indices
                .iter()
                .position(|&idx| self.items[idx].selection.as_ref() == Some(&needle))
        });

        if selection_index.is_none() {
            selection_index = self.first_selectable_index();
        }

        self.list_state.select(selection_index);
        *self.list_state.offset_mut() = 0;
    }

    /// Get index of first selectable item
    pub(crate) fn first_selectable_index(&self) -> Option<usize> {
        self.visible_indices
            .iter()
            .position(|&idx| self.items[idx].selection.is_some())
    }

    /// Get index of last selectable item
    pub(crate) fn last_selectable_index(&self) -> Option<usize> {
        self.visible_indices
            .iter()
            .rposition(|&idx| self.items[idx].selection.is_some())
    }

    /// Check if filter is active
    pub(crate) fn filter_active(&self) -> bool {
        self.filter_query
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    }

    /// Get current filter query
    pub(crate) fn filter_query(&self) -> Option<&str> {
        self.filter_query.as_deref()
    }

    /// Get highlight terms for search
    pub(crate) fn highlight_terms(&self) -> &[String] {
        &self.filter_terms
    }

    /// Count visible selectable items
    pub(crate) fn visible_selectable_count(&self) -> usize {
        self.visible_indices
            .iter()
            .filter(|&&idx| self.items[idx].selection.is_some())
            .count()
    }

    /// Get total selectable items
    pub(crate) fn total_selectable(&self) -> usize {
        self.total_selectable
    }

    /// Calculate page step size based on viewport
    fn page_step(&self) -> usize {
        let rows = self.viewport_rows.unwrap_or(0).max(1);
        usize::from(rows)
    }
}
