//! Palette and modal rendering logic for the session TUI
//!
//! This module handles the rendering of all overlay UIs that appear on top of the main session:
//! - File palette - File browser for inserting file references
//! - Prompt palette - Custom prompt browser for /prompt: command
//! - Slash palette - Command suggestions for / commands
//! - Modal dialogs - General-purpose modal overlays
//!
//! Each palette provides:
//! - List-based navigation with keyboard shortcuts
//! - Search/filter capabilities
//! - Loading states while data is being fetched
//! - Pagination for large result sets
//! - Styled rendering consistent with the theme

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use vtcode_config::constants::ui;

use super::super::{
    Session,
    measure_text_width,
    modal::{
        ModalBodyContext, ModalListLayout, ModalRenderStyles,
        compute_modal_area, modal_content_width, render_modal_body,
    },
    file_palette::FilePalette,
    prompt_palette::PromptPalette,
    slash_palette,
    ratatui_color_from_ansi,
    ratatui_style_from_inline,
    PROMPT_COMMAND_NAME,
};
use crate::tui::types::InlineTextStyle;

impl Session {
    /// Renders all palettes and modals in the correct order.
    ///
    /// This is the main coordinator function that renders overlay UIs on top of the viewport.
    /// Palettes and modals are rendered in a specific order to ensure proper layering:
    /// 1. Modal (if active) - highest priority, blocks other palettes
    /// 2. Slash palette - command suggestions
    /// 3. File palette - file browser
    /// 4. Prompt palette - custom prompts
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The full viewport area for positioning overlays
    pub(in crate::tui::session) fn render_palettes_and_modals(
        &mut self,
        frame: &mut Frame<'_>,
        viewport: Rect,
    ) {
        // Render in order from bottom to top (last rendered is on top)
        self.render_file_palette(frame, viewport);
        self.render_prompt_palette(frame, viewport);
        self.render_slash_palette(frame, viewport);
        self.render_modal(frame, viewport);
    }

    /// Renders the file browser palette.
    ///
    /// The file palette allows users to browse and select files from the workspace.
    /// It appears when typing `@` in the input and shows a filtered list of files.
    ///
    /// Features:
    /// - Fuzzy filtering based on user input
    /// - Directory navigation with visual indicators
    /// - LS_COLORS support for file type styling
    /// - Pagination for large file lists
    /// - Loading state while files are being indexed
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The viewport area for positioning the palette
    fn render_file_palette(&mut self, frame: &mut Frame<'_>, viewport: Rect) {
        if !self.file_palette_active {
            return;
        }

        let Some(palette) = self.file_palette.as_ref() else {
            return;
        };

        if viewport.height == 0 || viewport.width == 0 || self.modal.is_some() {
            return;
        }

        // Show loading state if no files loaded yet
        if !palette.has_files() {
            self.render_file_palette_loading(frame, viewport);
            return;
        }

        let items = palette.current_page_items();
        if items.is_empty() && palette.filter_query().is_empty() {
            return;
        }

        let mut width_hint = 40u16;
        for (_, entry, _) in &items {
            width_hint = width_hint.max(measure_text_width(&entry.display_name) + 4);
        }

        let instructions = self.file_palette_instructions(palette);
        let has_continuation = palette.has_more_items();
        let modal_height = items.len()
            + instructions.len()
            + 2  // borders
            + if has_continuation { 1 } else { 0 }; // continuation indicator
        let area = compute_modal_area(viewport, width_hint, modal_height, 0, 0, true);

        frame.render_widget(Clear, area);
        let title = format!(
            "File Browser (Page {}/{})",
            palette.current_page_number(),
            palette.total_pages()
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let layout = ModalListLayout::new(inner, instructions.len());
        if let Some(text_area) = layout.text_area {
            let paragraph = Paragraph::new(instructions).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, text_area);
        }

        let mut list_items: Vec<ListItem> = items
            .iter()
            .map(|(_, entry, is_selected)| {
                let base_style = if *is_selected {
                    self.modal_list_highlight_style()
                } else {
                    self.default_style()
                };

                // Get system file color from LS_COLORS if available
                let mut style = if let Some(file_palette) = self.file_palette.as_ref() {
                    if let Some(file_style) = file_palette.style_for_entry(entry) {
                        // Convert anstyle::Style to ratatui::Style using anstyle_utils
                        let anstyle_converted =
                            crate::utils::anstyle_utils::ansi_style_to_ratatui_style(file_style);

                        let mut ratatui_style = base_style;
                        if let Some(fg) = anstyle_converted.fg {
                            ratatui_style = ratatui_style.fg(fg);
                        }
                        if let Some(bg) = anstyle_converted.bg {
                            ratatui_style = ratatui_style.bg(bg);
                        }
                        ratatui_style = ratatui_style.add_modifier(anstyle_converted.add_modifier);

                        ratatui_style
                    } else {
                        base_style
                    }
                } else {
                    base_style
                };

                // Add visual distinction for directories (this can enhance or override LS_COLORS)
                if entry.is_dir {
                    style = style.add_modifier(Modifier::BOLD);
                }

                // Add icon prefix
                let prefix = if entry.is_dir {
                    "↳  " // Folder indicator
                } else {
                    "  · " // Indent files
                };

                let display_text = format!("{}{}", prefix, entry.display_name);

                ListItem::new(Line::from(Span::styled(display_text, style)))
            })
            .collect();

        // Add continuation indicator if there are more items
        if palette.has_more_items() {
            let continuation_text = format!(
                "  ... ({} more items)",
                palette.total_items() - (palette.current_page_number() * 20)
            );
            let continuation_style = self
                .default_style()
                .add_modifier(Modifier::DIM | Modifier::ITALIC);
            list_items.push(ListItem::new(Line::from(Span::styled(
                continuation_text,
                continuation_style,
            ))));
        }

        let list = List::new(list_items).style(self.default_style());
        frame.render_widget(list, layout.list_area);
    }

    /// Renders the loading state for the file palette.
    ///
    /// Displays a simple message while files are being indexed.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The viewport area for positioning
    fn render_file_palette_loading(&self, frame: &mut Frame<'_>, viewport: Rect) {
        let width_hint = 40u16;
        let modal_height = 3;
        let area = compute_modal_area(viewport, width_hint, modal_height, 0, 0, true);

        frame.render_widget(Clear, area);
        let block = Block::default()
            .title("File Browser")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height > 0 && inner.width > 0 {
            let loading_text = vec![Line::from(Span::styled(
                "Loading workspace files...".to_string(),
                self.default_style().add_modifier(Modifier::DIM),
            ))];
            let paragraph = Paragraph::new(loading_text).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    }

    /// Generates instruction lines for the file palette.
    ///
    /// Shows navigation help and current filter status.
    ///
    /// # Arguments
    ///
    /// * `palette` - The file palette state
    ///
    /// # Returns
    ///
    /// A vector of styled instruction lines
    fn file_palette_instructions(&self, palette: &FilePalette) -> Vec<Line<'static>> {
        let mut lines = vec![];

        if palette.is_empty() {
            lines.push(Line::from(Span::styled(
                "No files found matching filter".to_string(),
                self.default_style().add_modifier(Modifier::DIM),
            )));
        } else {
            let total = palette.total_items();
            let count_text = if total == 1 {
                "1 file".to_string()
            } else {
                format!("{} files", total)
            };

            let nav_text = "↑↓ Navigate · PgUp/PgDn Page · Tab/Enter Select";

            lines.push(Line::from(vec![Span::styled(
                format!("{} · Esc Close", nav_text),
                self.default_style(),
            )]));

            lines.push(Line::from(vec![
                Span::styled(
                    format!("Showing {}", count_text),
                    self.default_style().add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    if !palette.filter_query().is_empty() {
                        format!(" matching '{}'", palette.filter_query())
                    } else {
                        String::new()
                    },
                    self.accent_style(),
                ),
            ]));
        }

        lines
    }

    /// Renders the custom prompt palette.
    ///
    /// The prompt palette allows users to browse and select custom prompts.
    /// It appears when typing `/prompt:` in the input.
    ///
    /// Features:
    /// - Fuzzy filtering based on prompt names
    /// - Pagination for large prompt lists
    /// - Loading state while prompts are being loaded
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The viewport area for positioning the palette
    fn render_prompt_palette(&mut self, frame: &mut Frame<'_>, viewport: Rect) {
        if !self.prompt_palette_active {
            return;
        }

        let Some(palette) = self.prompt_palette.as_ref() else {
            return;
        };

        if viewport.height == 0 || viewport.width == 0 || self.modal.is_some() {
            return;
        }

        // Show loading state if no prompts loaded yet
        if !palette.has_prompts() {
            self.render_prompt_palette_loading(frame, viewport);
            return;
        }

        let items = palette.current_page_items();
        if items.is_empty() && palette.filter_query().is_empty() {
            return;
        }

        let mut width_hint = 40u16;
        for (_, entry, _) in &items {
            width_hint = width_hint.max(measure_text_width(&entry.name) + 4);
        }

        let instructions = self.prompt_palette_instructions(palette);
        let has_continuation = palette.has_more_items();
        let modal_height = items.len()
            + instructions.len()
            + 2  // borders
            + if has_continuation { 1 } else { 0 }; // continuation indicator
        let area = compute_modal_area(viewport, width_hint, modal_height, 0, 0, true);

        frame.render_widget(Clear, area);
        let title = format!(
            "Custom Prompts (Page {}/{})",
            palette.current_page_number(),
            palette.total_pages()
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let layout = ModalListLayout::new(inner, instructions.len());
        if let Some(text_area) = layout.text_area {
            let paragraph = Paragraph::new(instructions).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, text_area);
        }

        let mut list_items: Vec<ListItem> = items
            .iter()
            .map(|(_, entry, is_selected)| {
                let base_style = if *is_selected {
                    self.modal_list_highlight_style()
                } else {
                    self.default_style()
                };

                // Format: "  · prompt-name"
                let display_text = format!("  · {}", entry.name);

                ListItem::new(Line::from(Span::styled(display_text, base_style)))
            })
            .collect();

        // Add continuation indicator if there are more items
        if palette.has_more_items() {
            let continuation_text = format!(
                "  ... ({} more items)",
                palette.total_items() - (palette.current_page_number() * 20)
            );
            let continuation_style = self
                .default_style()
                .add_modifier(Modifier::DIM | Modifier::ITALIC);
            list_items.push(ListItem::new(Line::from(Span::styled(
                continuation_text,
                continuation_style,
            ))));
        }

        let list = List::new(list_items).style(self.default_style());
        frame.render_widget(list, layout.list_area);
    }

    /// Renders the loading state for the prompt palette.
    ///
    /// Displays a simple message while prompts are being loaded.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The viewport area for positioning
    fn render_prompt_palette_loading(&self, frame: &mut Frame<'_>, viewport: Rect) {
        let width_hint = 40u16;
        let modal_height = 3;
        let area = compute_modal_area(viewport, width_hint, modal_height, 0, 0, true);

        frame.render_widget(Clear, area);
        let block = Block::default()
            .title("Custom Prompts")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height > 0 && inner.width > 0 {
            let loading_text = vec![Line::from(Span::styled(
                "Loading custom prompts...".to_string(),
                self.default_style().add_modifier(Modifier::DIM),
            ))];
            let paragraph = Paragraph::new(loading_text).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    }

    /// Generates instruction lines for the prompt palette.
    ///
    /// Shows navigation help and current filter status.
    ///
    /// # Arguments
    ///
    /// * `palette` - The prompt palette state
    ///
    /// # Returns
    ///
    /// A vector of styled instruction lines
    fn prompt_palette_instructions(&self, palette: &PromptPalette) -> Vec<Line<'static>> {
        let mut lines = vec![];

        if palette.is_empty() {
            lines.push(Line::from(Span::styled(
                "No prompts found matching filter".to_string(),
                self.default_style().add_modifier(Modifier::DIM),
            )));
        } else {
            let total = palette.total_items();
            let count_text = if total == 1 {
                "1 prompt".to_string()
            } else {
                format!("{} prompts", total)
            };

            lines.push(Line::from(vec![Span::styled(
                "↑↓ Navigate · Enter/Tab Select · Esc Close",
                self.default_style(),
            )]));

            lines.push(Line::from(vec![
                Span::styled(
                    format!("Showing {}", count_text),
                    self.default_style().add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    if !palette.filter_query().is_empty() {
                        format!(" matching '{}'", palette.filter_query())
                    } else {
                        String::new()
                    },
                    self.accent_style(),
                ),
            ]));
        }

        lines
    }

    /// Renders the slash command palette.
    ///
    /// The slash palette shows available slash commands when the user types `/`.
    /// It provides fuzzy filtering and shows both built-in commands and custom prompts.
    ///
    /// Features:
    /// - Automatic filtering based on typed command
    /// - Keyboard navigation with up/down arrows
    /// - Preview of selected command in input
    /// - Scrolling for long command lists
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The viewport area for positioning the palette
    pub(in crate::tui::session) fn render_slash_palette(&mut self, frame: &mut Frame<'_>, viewport: Rect) {
        if viewport.height == 0 || viewport.width == 0 || self.modal.is_some() {
            self.slash_palette.clear_visible_rows();
            return;
        }
        let suggestions = self.slash_palette.suggestions();
        if suggestions.is_empty() {
            self.slash_palette.clear_visible_rows();
            return;
        }

        let mut width_hint = measure_text_width(ui::SLASH_PALETTE_HINT_PRIMARY);
        width_hint = width_hint.max(measure_text_width(ui::SLASH_PALETTE_HINT_SECONDARY));
        for suggestion in suggestions.iter().take(ui::SLASH_SUGGESTION_LIMIT) {
            let label = match suggestion {
                slash_palette::SlashPaletteSuggestion::Static(cmd) => {
                    if !cmd.description.is_empty() {
                        format!("/{} {}", cmd.name, cmd.description)
                    } else {
                        format!("/{}", cmd.name)
                    }
                }
                slash_palette::SlashPaletteSuggestion::Custom(prompt) => {
                    // For custom prompts, format as /prompt:name (legacy alias /prompts:name)
                    let prompt_cmd = format!("{}:{}", PROMPT_COMMAND_NAME, prompt.name);
                    let description = prompt.description.as_str();
                    if !description.is_empty() {
                        format!("/{} {}", prompt_cmd, description)
                    } else {
                        format!("/{}", prompt_cmd)
                    }
                }
            };
            width_hint = width_hint.max(measure_text_width(&label));
        }

        let instructions = self.slash_palette_instructions();
        let area = compute_modal_area(viewport, width_hint, instructions.len(), 0, 0, true);

        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(self.suggestion_block_title())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.height == 0 || inner.width == 0 {
            self.slash_palette.clear_visible_rows();
            return;
        }

        let layout = ModalListLayout::new(inner, instructions.len());
        if let Some(text_area) = layout.text_area {
            let paragraph = Paragraph::new(instructions).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, text_area);
        }

        self.slash_palette
            .set_visible_rows(layout.list_area.height as usize);

        // Get all list items (scrollable via ListState)
        let list_items = self.slash_list_items();

        let list = List::new(list_items)
            .style(self.default_style())
            .highlight_style(self.slash_highlight_style());

        frame.render_stateful_widget(list, layout.list_area, self.slash_palette.list_state_mut());
    }

    /// Generates instruction lines for the slash palette.
    ///
    /// Shows navigation help for the command palette.
    ///
    /// # Returns
    ///
    /// A vector of styled instruction lines
    fn slash_palette_instructions(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                ui::SLASH_PALETTE_HINT_PRIMARY.to_string(),
                self.default_style(),
            )),
            Line::from(Span::styled(
                ui::SLASH_PALETTE_HINT_SECONDARY.to_string(),
                self.default_style().add_modifier(Modifier::DIM),
            )),
        ]
    }

    /// Creates list items for slash command suggestions.
    ///
    /// Formats each suggestion with command name and description.
    ///
    /// # Returns
    ///
    /// A vector of list items ready for rendering
    fn slash_list_items(&self) -> Vec<ListItem<'static>> {
        self.slash_palette
            .suggestions()
            .iter()
            .map(|suggestion| match suggestion {
                slash_palette::SlashPaletteSuggestion::Static(command) => {
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("/{}", command.name), self.slash_name_style()),
                        Span::raw(" "),
                        Span::styled(
                            command.description.to_string(),
                            self.slash_description_style(),
                        ),
                    ]))
                }
                slash_palette::SlashPaletteSuggestion::Custom(prompt) => {
                    let display_name = format!("/{}:{}", PROMPT_COMMAND_NAME, prompt.name);
                    let description = prompt.description.clone();
                    ListItem::new(Line::from(vec![
                        Span::styled(display_name, self.slash_name_style()),
                        Span::raw(" "),
                        Span::styled(description, self.slash_description_style()),
                    ]))
                }
            })
            .collect()
    }

    /// Gets the highlight style for slash palette selection.
    ///
    /// # Returns
    ///
    /// The style to use for the selected command
    fn slash_highlight_style(&self) -> Style {
        let mut style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
        if let Some(primary) = self.theme.primary.or(self.theme.secondary) {
            style = style.fg(ratatui_color_from_ansi(primary));
        }
        style
    }

    /// Gets the style for slash command names.
    ///
    /// # Returns
    ///
    /// The style to use for command names
    fn slash_name_style(&self) -> Style {
        let style = InlineTextStyle::default()
            .bold()
            .with_color(self.theme.primary.or(self.theme.foreground));
        ratatui_style_from_inline(&style, self.theme.foreground)
    }

    /// Gets the style for slash command descriptions.
    ///
    /// # Returns
    ///
    /// The style to use for command descriptions
    fn slash_description_style(&self) -> Style {
        self.default_style().add_modifier(Modifier::DIM)
    }

    /// Renders a modal dialog overlay.
    ///
    /// Modals are general-purpose overlays that can display:
    /// - Informational messages
    /// - Lists with selection
    /// - Search interfaces
    /// - Secure prompts (password input)
    ///
    /// The modal is centered on the viewport and blocks interaction with other UI elements.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `viewport` - The full viewport area for positioning
    fn render_modal(&mut self, frame: &mut Frame<'_>, viewport: Rect) {
        if viewport.width == 0 || viewport.height == 0 {
            return;
        }

        let styles = self.modal_render_styles();
        let Some(modal) = self.modal.as_mut() else {
            return;
        };

        let width_hint = modal_content_width(
            &modal.lines,
            modal.list.as_ref(),
            modal.secure_prompt.as_ref(),
            modal.search.as_ref(),
        );
        let prompt_lines = modal.secure_prompt.is_some().then_some(2).unwrap_or(0);
        let search_lines = modal.search.as_ref().map(|_| 3).unwrap_or(0);
        let area = compute_modal_area(
            viewport,
            width_hint,
            modal.lines.len(),
            prompt_lines,
            search_lines,
            modal.list.is_some(),
        );

        let block = Block::default()
            .title(Line::styled(modal.title.clone(), styles.title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(styles.border);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        if area.width <= 2 || area.height <= 2 {
            return;
        }

        let inner = Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        render_modal_body(
            frame,
            inner,
            ModalBodyContext {
                instructions: &modal.lines,
                list: modal.list.as_mut(),
                styles: &styles,
                secure_prompt: modal.secure_prompt.as_ref(),
                search: modal.search.as_ref(),
                input: self.input_manager.content(),
                cursor: self.input_manager.cursor(),
            },
        );
    }

    /// Creates the style configuration for modal rendering.
    ///
    /// This centralizes all modal styling to ensure consistency across different modal types.
    ///
    /// # Returns
    ///
    /// A `ModalRenderStyles` struct with all configured styles
    fn modal_render_styles(&self) -> ModalRenderStyles {
        ModalRenderStyles {
            border: self.border_style(),
            highlight: self.modal_list_highlight_style(),
            badge: self.section_title_style().add_modifier(Modifier::DIM),
            header: self.section_title_style(),
            selectable: self.default_style().add_modifier(Modifier::BOLD),
            detail: self.default_style().add_modifier(Modifier::DIM),
            search_match: self
                .accent_style()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            title: Style::default().add_modifier(Modifier::BOLD),
            divider: self
                .default_style()
                .add_modifier(Modifier::DIM | Modifier::ITALIC),
            instruction_border: self.border_style(),
            instruction_title: self.section_title_style(),
            instruction_bullet: self.accent_style().add_modifier(Modifier::BOLD),
            instruction_body: self.default_style(),
            hint: self
                .default_style()
                .add_modifier(Modifier::DIM | Modifier::ITALIC),
        }
    }

    /// Gets the highlight style for modal list items.
    ///
    /// This style is used for the currently selected item in modal lists.
    ///
    /// # Returns
    ///
    /// The highlight style with reversed colors and bold text
    fn modal_list_highlight_style(&self) -> Style {
        let mut style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
        if let Some(primary) = self.theme.primary.or(self.theme.foreground) {
            style = style.fg(ratatui_color_from_ansi(primary));
        }
        style
    }
}
