//! Modal search and filtering functionality

use super::state::ModalListState;
use crate::search::normalize_query;
use crate::tui::types::InlineListSelection;

impl ModalListState {
    /// Apply search filter to list items
    pub fn apply_search(&mut self, query: &str) {
        let preferred = self.current_selection();
        self.apply_search_with_preference(query, preferred);
    }

    /// Apply search filter with preferred selection
    pub fn apply_search_with_preference(
        &mut self,
        query: &str,
        preferred: Option<InlineListSelection>,
    ) {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            self.visible_indices = (0..self.items.len()).collect();
            self.filter_terms.clear();
            self.filter_query = None;
            self.select_initial(preferred);
            return;
        }

        let normalized_query = normalize_query(trimmed);
        let terms = normalized_query
            .split_whitespace()
            .filter(|term| !term.is_empty())
            .map(|term| term.to_string())
            .collect::<Vec<_>>();
        let mut indices = Vec::new();
        let mut pending_divider: Option<usize> = None;
        let mut current_header: Option<usize> = None;
        let mut header_matches = false;
        let mut header_included = false;

        for (index, item) in self.items.iter().enumerate() {
            if item.is_divider {
                pending_divider = Some(index);
                current_header = None;
                header_matches = false;
                header_included = false;
                continue;
            }

            if item.is_header() {
                current_header = Some(index);
                header_matches = item.matches(&normalized_query);
                header_included = false;
                if header_matches {
                    if let Some(divider_index) = pending_divider.take() {
                        indices.push(divider_index);
                    }
                    indices.push(index);
                    header_included = true;
                }
                continue;
            }

            let item_matches = item.matches(&normalized_query);
            let include_item = header_matches || item_matches;
            if include_item {
                if let Some(divider_index) = pending_divider.take() {
                    indices.push(divider_index);
                }
                if let Some(header_index) = current_header {
                    if !header_included {
                        indices.push(header_index);
                        header_included = true;
                    }
                }
                indices.push(index);
            }
        }
        self.visible_indices = indices;
        self.filter_terms = terms;
        self.filter_query = Some(trimmed.to_string());
        self.select_initial(preferred);
    }
}
