//! Modal rendering functionality

use super::secure_prompt::render_secure_prompt;
use super::state::{ModalListState, ModalSearchState};
use crate::tui::types::SecurePromptConfig;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::mem;
use unicode_width::UnicodeWidthStr;
use vtcode_config::constants::ui;

/// Render styles for modal elements
pub struct ModalRenderStyles {
    pub border: Style,
    pub highlight: Style,
    pub badge: Style,
    pub header: Style,
    pub selectable: Style,
    pub detail: Style,
    pub search_match: Style,
    pub title: Style,
    pub divider: Style,
    pub instruction_border: Style,
    pub instruction_title: Style,
    pub instruction_bullet: Style,
    pub instruction_body: Style,
    pub hint: Style,
}

/// Modal section types for layout
#[derive(Clone, Copy)]
pub enum ModalSection {
    Search,
    Instructions,
    Prompt,
    List,
}

/// Context for rendering modal body
pub struct ModalBodyContext<'a, 'b> {
    pub instructions: &'a [String],
    pub list: Option<&'b mut ModalListState>,
    pub styles: &'a ModalRenderStyles,
    pub secure_prompt: Option<&'a SecurePromptConfig>,
    pub search: Option<&'a ModalSearchState>,
    pub input: &'a str,
    pub cursor: usize,
}

/// Render modal body with all sections
pub fn render_modal_body(frame: &mut Frame<'_>, area: Rect, context: ModalBodyContext<'_, '_>) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut sections = Vec::new();
    let has_instructions = context
        .instructions
        .iter()
        .any(|line| !line.trim().is_empty());
    if context.search.is_some() {
        sections.push(ModalSection::Search);
    }
    if has_instructions {
        sections.push(ModalSection::Instructions);
    }
    if context.secure_prompt.is_some() {
        sections.push(ModalSection::Prompt);
    }
    if context.list.is_some() {
        sections.push(ModalSection::List);
    }

    if sections.is_empty() {
        return;
    }

    let mut constraints = Vec::new();
    for section in &sections {
        match section {
            ModalSection::Search => constraints.push(Constraint::Length(3.min(area.height))),
            ModalSection::Instructions => {
                let visible_rows = context.instructions.len().max(1) as u16;
                let height = visible_rows.saturating_add(2);
                constraints.push(Constraint::Length(height.min(area.height)));
            }
            ModalSection::Prompt => constraints.push(Constraint::Length(3.min(area.height))),
            ModalSection::List => constraints.push(Constraint::Min(3)),
        }
    }

    let chunks = Layout::vertical(constraints).split(area);
    let mut list_state = context.list;

    for (section, chunk) in sections.into_iter().zip(chunks.iter()) {
        match section {
            ModalSection::Instructions => {
                if chunk.height > 0 && has_instructions {
                    render_modal_instructions(frame, *chunk, context.instructions, context.styles);
                }
            }
            ModalSection::Prompt => {
                if let Some(config) = context.secure_prompt {
                    render_secure_prompt(frame, *chunk, config, context.input, context.cursor);
                }
            }
            ModalSection::Search => {
                if let Some(config) = context.search {
                    render_modal_search(frame, *chunk, config, context.styles);
                }
            }
            ModalSection::List => {
                if let Some(list_state) = list_state.as_deref_mut() {
                    render_modal_list(frame, *chunk, list_state, context.styles);
                }
            }
        }
    }
}

/// Render modal instructions section
fn render_modal_instructions(
    frame: &mut Frame<'_>,
    area: Rect,
    instructions: &[String],
    styles: &ModalRenderStyles,
) {
    fn wrap_instruction_lines(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current = String::new();

        for word in text.split_whitespace() {
            let word_width = UnicodeWidthStr::width(word);
            if current.is_empty() {
                current.push_str(word);
                continue;
            }

            let current_width = UnicodeWidthStr::width(current.as_str());
            let candidate_width = current_width.saturating_add(1).saturating_add(word_width);
            if candidate_width > width {
                lines.push(current);
                current = word.to_string();
            } else {
                current.push(' ');
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }

        if lines.is_empty() {
            vec![text.to_string()]
        } else {
            lines
        }
    }

    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut items = Vec::new();
    let mut first_content_rendered = false;
    let content_width = area.width.saturating_sub(4) as usize;
    let bullet_prefix = format!("{} ", ui::MODAL_INSTRUCTIONS_BULLET);
    let bullet_indent = " ".repeat(UnicodeWidthStr::width(bullet_prefix.as_str()));

    for line in instructions {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            items.push(ListItem::new(Line::default()));
            continue;
        }

        let wrapped = wrap_instruction_lines(trimmed, content_width);
        if wrapped.is_empty() {
            items.push(ListItem::new(Line::default()));
            continue;
        }

        if !first_content_rendered {
            let mut lines = Vec::new();
            for (index, segment) in wrapped.into_iter().enumerate() {
                let style = if index == 0 {
                    styles.header
                } else {
                    styles.instruction_body
                };
                lines.push(Line::from(Span::styled(segment, style)));
            }
            items.push(ListItem::new(lines));
            first_content_rendered = true;
        } else {
            let mut lines = Vec::new();
            for (index, segment) in wrapped.into_iter().enumerate() {
                if index == 0 {
                    lines.push(Line::from(vec![
                        Span::styled(bullet_prefix.clone(), styles.instruction_bullet),
                        Span::styled(segment, styles.instruction_body),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(bullet_indent.clone(), styles.instruction_bullet),
                        Span::styled(segment, styles.instruction_body),
                    ]));
                }
            }
            items.push(ListItem::new(lines));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::default()));
    }

    let block = Block::default()
        .title(Span::styled(
            ui::MODAL_INSTRUCTIONS_TITLE.to_string(),
            styles.instruction_title,
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.instruction_border);

    let widget = List::new(items)
        .block(block)
        .style(styles.instruction_body)
        .highlight_symbol("")
        .repeat_highlight_symbol(false);

    frame.render_widget(widget, area);
}

/// Render modal search box
fn render_modal_search(
    frame: &mut Frame<'_>,
    area: Rect,
    search: &ModalSearchState,
    styles: &ModalRenderStyles,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut spans = Vec::new();
    if search.query.is_empty() {
        if let Some(placeholder) = &search.placeholder {
            spans.push(Span::styled(placeholder.clone(), styles.detail));
        }
    } else {
        spans.push(Span::styled(search.query.clone(), styles.selectable));
    }
    spans.push(Span::styled("▌".to_string(), styles.highlight));

    let block = Block::default()
        .title(Span::styled(search.label.clone(), styles.header))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border);

    let paragraph = Paragraph::new(Line::from(spans))
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

/// Render modal list widget
pub fn render_modal_list(
    frame: &mut Frame<'_>,
    area: Rect,
    list: &mut ModalListState,
    styles: &ModalRenderStyles,
) {
    if list.visible_indices.is_empty() {
        list.list_state.select(None);
        *list.list_state.offset_mut() = 0;
        let message = Paragraph::new(Line::from(Span::styled(
            ui::MODAL_LIST_NO_RESULTS_MESSAGE.to_string(),
            styles.detail,
        )))
        .block(modal_list_block(list, styles))
        .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
        return;
    }

    let viewport_rows = area.height.saturating_sub(2);
    list.set_viewport_rows(viewport_rows);
    list.ensure_visible(viewport_rows);
    let items = modal_list_items(list, styles);
    let widget = List::new(items)
        .block(modal_list_block(list, styles))
        .highlight_style(styles.highlight)
        .highlight_symbol(ui::MODAL_LIST_HIGHLIGHT_FULL)
        .repeat_highlight_symbol(true);
    frame.render_stateful_widget(widget, area, &mut list.list_state);
}

/// Create block for modal list
fn modal_list_block(list: &ModalListState, styles: &ModalRenderStyles) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border);
    if let Some(summary) = modal_list_summary_line(list, styles) {
        block = block.title_bottom(summary);
    }
    block
}

/// Create summary line for modal list (filter status)
#[allow(clippy::const_is_empty)]
fn modal_list_summary_line(
    list: &ModalListState,
    styles: &ModalRenderStyles,
) -> Option<Line<'static>> {
    if !list.filter_active() {
        return None;
    }

    let mut spans = Vec::new();
    if let Some(query) = list.filter_query().filter(|value| !value.is_empty()) {
        spans.push(Span::styled(
            format!("{}:", ui::MODAL_LIST_SUMMARY_FILTER_LABEL),
            styles.detail,
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(query.to_string(), styles.selectable));
    }

    let matches = list.visible_selectable_count();
    let total = list.total_selectable();
    if matches == 0 {
        if !spans.is_empty() {
            spans.push(Span::styled(
                ui::MODAL_LIST_SUMMARY_SEPARATOR.to_string(),
                styles.detail,
            ));
        }
        spans.push(Span::styled(
            ui::MODAL_LIST_SUMMARY_NO_MATCHES.to_string(),
            styles.search_match,
        ));
        if !ui::MODAL_LIST_SUMMARY_RESET_HINT.is_empty() {
            spans.push(Span::styled(
                format!(
                    "{}{}",
                    ui::MODAL_LIST_SUMMARY_SEPARATOR,
                    ui::MODAL_LIST_SUMMARY_RESET_HINT
                ),
                styles.hint,
            ));
        }
    } else {
        if !spans.is_empty() {
            spans.push(Span::styled(
                ui::MODAL_LIST_SUMMARY_SEPARATOR.to_string(),
                styles.detail,
            ));
        }
        spans.push(Span::styled(
            format!(
                "{} {} {} {}",
                ui::MODAL_LIST_SUMMARY_MATCHES_LABEL,
                matches,
                ui::MODAL_LIST_SUMMARY_TOTAL_LABEL,
                total
            ),
            styles.detail,
        ));
    }

    if spans.is_empty() {
        None
    } else {
        Some(Line::from(spans))
    }
}

/// Highlight matching segments in text
pub(super) fn highlight_segments(
    text: &str,
    normal_style: Style,
    highlight_style: Style,
    terms: &[String],
) -> Vec<Span<'static>> {
    if text.is_empty() {
        return vec![Span::styled(String::new(), normal_style)];
    }

    if terms.is_empty() {
        return vec![Span::styled(text.to_string(), normal_style)];
    }

    let lower = text.to_ascii_lowercase();
    let mut char_offsets: Vec<usize> = text.char_indices().map(|(offset, _)| offset).collect();
    char_offsets.push(text.len());
    let char_count = char_offsets.len().saturating_sub(1);
    if char_count == 0 {
        return vec![Span::styled(text.to_string(), normal_style)];
    }

    let mut highlight_flags = vec![false; char_count];
    for term in terms {
        let needle = term.to_ascii_lowercase();
        if needle.is_empty() {
            continue;
        }

        let mut search_start = 0usize;
        while search_start < lower.len() {
            let Some(pos) = lower[search_start..].find(&needle) else {
                break;
            };
            let byte_start = search_start + pos;
            let byte_end = byte_start + needle.len();
            let start_index = char_offsets.partition_point(|offset| *offset < byte_start);
            let end_index = char_offsets.partition_point(|offset| *offset < byte_end);
            for index in start_index..end_index.min(char_count) {
                highlight_flags[index] = true;
            }
            search_start = byte_end;
        }
    }

    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_highlight = highlight_flags.first().copied().unwrap_or(false);
    for (idx, ch) in text.chars().enumerate() {
        let highlight = highlight_flags.get(idx).copied().unwrap_or(false);
        if idx == 0 {
            current_highlight = highlight;
        } else if highlight != current_highlight {
            let style = if current_highlight {
                highlight_style
            } else {
                normal_style
            };
            segments.push(Span::styled(mem::take(&mut current), style));
            current_highlight = highlight;
        }
        current.push(ch);
    }

    if !current.is_empty() {
        let style = if current_highlight {
            highlight_style
        } else {
            normal_style
        };
        segments.push(Span::styled(current, style));
    }

    if segments.is_empty() {
        segments.push(Span::styled(String::new(), normal_style));
    }

    segments
}

/// Create list items for modal list
pub fn modal_list_items(
    list: &ModalListState,
    styles: &ModalRenderStyles,
) -> Vec<ListItem<'static>> {
    list.visible_indices
        .iter()
        .enumerate()
        .map(|(visible_index, &index)| modal_list_item(list, visible_index, index, styles))
        .collect()
}

/// Create a single list item
fn modal_list_item(
    list: &ModalListState,
    _visible_index: usize,
    item_index: usize,
    styles: &ModalRenderStyles,
) -> ListItem<'static> {
    let item = &list.items[item_index];
    if item.is_divider {
        let divider = if item.title.is_empty() {
            ui::INLINE_BLOCK_HORIZONTAL.repeat(8)
        } else {
            item.title.clone()
        };
        return ListItem::new(vec![Line::from(Span::styled(divider, styles.divider))]);
    }

    let indent = "  ".repeat(item.indent as usize);

    let mut primary_spans = Vec::new();

    if !indent.is_empty() {
        primary_spans.push(Span::raw(indent.clone()));
    }

    if let Some(badge) = &item.badge {
        let badge_label = format!("[{}]", badge);
        primary_spans.push(Span::styled(badge_label, styles.badge));
        primary_spans.push(Span::raw(" "));
    }

    let title_style = if item.selection.is_some() {
        styles.selectable
    } else if item.is_header() {
        styles.header
    } else {
        styles.detail
    };

    let title_spans = highlight_segments(
        item.title.as_str(),
        title_style,
        styles.search_match,
        list.highlight_terms(),
    );
    primary_spans.extend(title_spans);

    let mut lines = vec![Line::from(primary_spans)];

    if let Some(subtitle) = &item.subtitle {
        let mut secondary_spans = Vec::new();
        if !indent.is_empty() {
            secondary_spans.push(Span::raw(indent.clone()));
        }
        let subtitle_spans = highlight_segments(
            subtitle,
            styles.detail,
            styles.search_match,
            list.highlight_terms(),
        );
        secondary_spans.extend(subtitle_spans);
        lines.push(Line::from(secondary_spans));
    }

    lines.push(Line::default());
    ListItem::new(lines)
}
