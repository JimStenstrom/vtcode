use crossterm::event::{KeyCode, KeyEvent};
use vtcode_config::constants::ui;

use super::{
    LEGACY_PROMPT_COMMAND_NAME, PROMPT_COMMAND_NAME, PROMPT_COMMAND_PREFIX, Session,
    slash_palette::{SlashPaletteUpdate, command_prefix, command_range},
};

impl Session {


    pub(super) fn handle_slash_palette_change(&mut self) {
        self.recalculate_transcript_rows();
        self.enforce_scroll_bounds();
        self.mark_dirty();
    }

    pub(super) fn clear_slash_suggestions(&mut self) {
        if self.slash_palette.clear() {
            self.handle_slash_palette_change();
        }
    }

    pub(super) fn update_slash_suggestions(&mut self) {
        if !self.input_enabled {
            self.clear_slash_suggestions();
            return;
        }

        let Some(prefix) =
            command_prefix(self.input_manager.content(), self.input_manager.cursor())
        else {
            self.clear_slash_suggestions();
            return;
        };

        // Update slash palette with custom prompts if available
        if let Some(ref custom_prompts) = self.custom_prompts {
            self.slash_palette
                .set_custom_prompts(custom_prompts.clone());
        }

        match self
            .slash_palette
            .update(Some(&prefix), ui::SLASH_SUGGESTION_LIMIT)
        {
            SlashPaletteUpdate::NoChange => {}
            SlashPaletteUpdate::Cleared | SlashPaletteUpdate::Changed { .. } => {
                self.handle_slash_palette_change();
            }
        }
    }

    pub(super) fn slash_navigation_available(&self) -> bool {
        self.input_enabled
            && !self.slash_palette.is_empty()
            && self.modal.is_none()
            && !self.file_palette_active
            && !self.prompt_palette_active
    }

    pub(super) fn move_slash_selection_up(&mut self) -> bool {
        let changed = self.slash_palette.move_up();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn move_slash_selection_down(&mut self) -> bool {
        let changed = self.slash_palette.move_down();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn select_first_slash_suggestion(&mut self) -> bool {
        let changed = self.slash_palette.select_first();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn select_last_slash_suggestion(&mut self) -> bool {
        let changed = self.slash_palette.select_last();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn page_up_slash_suggestion(&mut self) -> bool {
        let changed = self.slash_palette.page_up();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn page_down_slash_suggestion(&mut self) -> bool {
        let changed = self.slash_palette.page_down();
        self.handle_slash_selection_change(changed)
    }

    pub(super) fn handle_slash_selection_change(&mut self, changed: bool) -> bool {
        if changed {
            self.preview_selected_slash_suggestion();
            self.recalculate_transcript_rows();
            self.enforce_scroll_bounds();
            self.mark_dirty();
            true
        } else {
            false
        }
    }

    fn preview_selected_slash_suggestion(&mut self) {
        let Some(command) = self.slash_palette.selected_command() else {
            return;
        };
        let Some(range) = command_range(self.input_manager.content(), self.input_manager.cursor())
        else {
            return;
        };

        let current_input = self.input_manager.content().to_string();
        let prefix = &current_input[..range.start];
        let suffix = &current_input[range.end..];

        let mut new_input = String::new();
        new_input.push_str(prefix);
        new_input.push('/');
        new_input.push_str(command.name);
        let cursor_position = new_input.len();

        if !suffix.is_empty() {
            if !suffix.chars().next().map_or(false, char::is_whitespace) {
                new_input.push(' ');
            }
            new_input.push_str(suffix);
        }

        self.input_manager.set_content(new_input.clone());
        self.input_manager
            .set_cursor(cursor_position.min(new_input.len()));
        self.mark_dirty();
    }

    pub(super) fn apply_selected_slash_suggestion(&mut self) -> bool {
        if let Some(custom_prompt) = self.slash_palette.selected_custom_prompt() {
            let input_content = self.input_manager.content();
            let cursor_pos = self.input_manager.cursor();
            let Some(range) = command_range(input_content, cursor_pos) else {
                return false;
            };

            let mut new_input = String::from(PROMPT_COMMAND_PREFIX);
            new_input.push_str(&custom_prompt.name);

            let suffix = &input_content[range.end..];
            if !suffix.is_empty() {
                if !suffix.chars().next().map_or(false, char::is_whitespace) {
                    new_input.push(' ');
                }
                new_input.push_str(suffix);
            } else {
                new_input.push(' ');
            }

            let cursor_position = new_input.len();

            self.input_manager.set_content(new_input);
            self.input_manager.set_cursor(cursor_position);
            self.clear_slash_suggestions();
            self.mark_dirty();
            return true;
        }

        let Some(command) = self.slash_palette.selected_command() else {
            return false;
        };

        let command_name = command.name.to_string();

        let input_content = self.input_manager.content();
        let cursor_pos = self.input_manager.cursor();
        let Some(range) = command_range(input_content, cursor_pos) else {
            return false;
        };

        let suffix = input_content[range.end..].to_string();
        let mut new_input = format!("/{}", command_name);

        let cursor_position = if suffix.is_empty() {
            new_input.push(' ');
            new_input.len()
        } else {
            if !suffix.chars().next().map_or(false, char::is_whitespace) {
                new_input.push(' ');
            }
            let position = new_input.len();
            new_input.push_str(&suffix);
            position
        };

        self.input_manager.set_content(new_input);
        self.input_manager.set_cursor(cursor_position);

        if command_name == "files" {
            self.clear_slash_suggestions();
            self.mark_dirty();
            self.deferred_file_browser_trigger = true;
        } else if command_name == PROMPT_COMMAND_NAME || command_name == LEGACY_PROMPT_COMMAND_NAME
        {
            self.clear_slash_suggestions();
            self.mark_dirty();
            self.deferred_prompt_browser_trigger = true;
        } else {
            self.clear_slash_suggestions();
            self.mark_dirty();
        }

        true
    }

    pub(super) fn try_handle_slash_navigation(
        &mut self,
        key: &KeyEvent,
        has_control: bool,
        has_alt: bool,
        has_command: bool,
    ) -> bool {
        if !self.slash_navigation_available() || has_control || has_alt {
            return false;
        }

        let handled = match key.code {
            KeyCode::Up => {
                if has_command {
                    self.select_first_slash_suggestion()
                } else {
                    self.move_slash_selection_up()
                }
            }
            KeyCode::Down => {
                if has_command {
                    self.select_last_slash_suggestion()
                } else {
                    self.move_slash_selection_down()
                }
            }
            KeyCode::PageUp => self.page_up_slash_suggestion(),
            KeyCode::PageDown => self.page_down_slash_suggestion(),
            KeyCode::Tab => self.move_slash_selection_down(),
            KeyCode::BackTab => self.move_slash_selection_up(),
            KeyCode::Enter => self.apply_selected_slash_suggestion(),
            _ => return false,
        };

        if handled {
            self.mark_dirty();
        }

        handled
    }




}
