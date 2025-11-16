//! Command handling and execution for the session.
//!
//! This module handles all `InlineCommand` processing and execution.
//! Commands are the primary way the backend communicates state changes
//! and actions to the UI session.
//!
//! ## Command Categories
//!
//! 1. **Message Commands**
//!    - `AppendLine` - Add a new message line to transcript
//!    - `Inline` - Append content to the current line
//!    - `ReplaceLast` - Replace the last N lines
//!
//! 2. **UI State Commands**
//!    - `SetPrompt` - Update the input prompt prefix
//!    - `SetPlaceholder` - Update the input placeholder text
//!    - `SetTheme` - Change the color theme
//!    - `SetInputStatus` - Update status indicators
//!    - `SetHeaderContext` - Update header information
//!
//! 3. **Modal Commands**
//!    - `ShowModal` - Display a modal dialog
//!    - `ShowListModal` - Display a modal with selectable list
//!    - `CloseModal` - Close the active modal
//!
//! 4. **Input Commands**
//!    - `SetInput` - Set the input field content
//!    - `ClearInput` - Clear the input field
//!    - `SetInputEnabled` - Enable/disable input
//!    - `SetCursorVisible` - Show/hide cursor
//!
//! 5. **Palette Commands**
//!    - `LoadFilePalette` - Load file palette with workspace files
//!    - `SetCustomPrompts` - Load custom prompt registry
//!
//! 6. **Special Commands**
//!    - `SetQueuedInputs` - Set queued input entries
//!    - `SetPlan` - Set the task plan
//!    - `ClearScreen` - Clear all messages
//!    - `ForceRedraw` - Force screen redraw
//!    - `Shutdown` - Request application exit
//!
//! ## Architecture
//!
//! The `handle_command` function is the main dispatcher that receives
//! `InlineCommand` enum variants and delegates to appropriate handler
//! methods. Most handlers are small, focused functions that update
//! specific pieces of session state.
//!
//! After handling each command, the session is marked as dirty to
//! ensure the UI is redrawn on the next frame.

use crate::tools::TaskPlan;
use super::Session;
use super::super::types::{InlineCommand, SecurePromptConfig};
use super::modal::{ModalState, ModalListState, ModalSearchState};
use tui_popup::PopupState;

// ============================================================================
// Main Command Dispatcher
// ============================================================================

impl Session {
    /// Handle an incoming command from the backend.
    ///
    /// This is the main entry point for all command processing. It dispatches
    /// to specific handler methods based on the command type, then marks the
    /// session as dirty to trigger a redraw.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    ///
    /// # Example Command Flow
    ///
    /// ```text
    /// Backend sends InlineCommand::AppendLine
    ///   ↓
    /// handle_command() receives it
    ///   ↓
    /// Calls self.push_line()
    ///   ↓
    /// Marks session dirty
    ///   ↓
    /// UI redraws on next frame
    /// ```
    pub fn handle_command(&mut self, command: InlineCommand) {
        match command {
            InlineCommand::AppendLine { kind, segments } => {
                self.push_line(kind, segments);
            }
            InlineCommand::Inline { kind, segment } => {
                self.append_inline(kind, segment);
            }
            InlineCommand::ReplaceLast { count, kind, lines } => {
                self.replace_last(count, kind, lines);
            }
            InlineCommand::SetPrompt { prefix, style } => {
                self.prompt_state.prompt_prefix = prefix;
                self.prompt_state.prompt_style = style;
                self.ensure_prompt_style_color();
            }
            InlineCommand::SetPlaceholder { hint, style } => {
                self.prompt_state.placeholder = hint;
                self.prompt_state.placeholder_style = style;
            }
            InlineCommand::SetMessageLabels { agent, user } => {
                self.display_state.labels.agent = agent.filter(|label| !label.is_empty());
                self.display_state.labels.user = user.filter(|label| !label.is_empty());
                self.invalidate_scroll_metrics();
            }
            InlineCommand::SetHeaderContext { context } => {
                self.render_state.header_context = context;
                self.ui_state.needs_redraw = true;
            }
            InlineCommand::SetInputStatus { left, right } => {
                self.prompt_state.input_status_left = left;
                self.prompt_state.input_status_right = right;
                self.ui_state.needs_redraw = true;
            }
            InlineCommand::SetTheme { theme } => {
                self.display_state.theme = theme;
                self.ensure_prompt_style_color();
                self.invalidate_transcript_cache();
            }
            InlineCommand::SetQueuedInputs { entries } => {
                self.set_queued_inputs_entries(entries);
                self.mark_dirty();
            }
            InlineCommand::SetPlan { plan } => {
                self.set_plan(plan);
            }
            InlineCommand::SetCursorVisible(value) => {
                self.ui_state.cursor_visible = value;
            }
            InlineCommand::SetInputEnabled(value) => {
                self.ui_state.input_enabled = value;
                self.update_slash_suggestions();
            }
            InlineCommand::SetInput(content) => {
                self.input_manager.set_content(content);
                self.scroll_manager.set_offset(0);
                self.update_slash_suggestions();
            }
            InlineCommand::ClearInput => {
                self.clear_input();
            }
            InlineCommand::ForceRedraw => {
                self.mark_dirty();
            }
            InlineCommand::ShowModal {
                title,
                lines,
                secure_prompt,
            } => {
                self.show_modal(title, lines, secure_prompt);
            }
            InlineCommand::ShowListModal {
                title,
                lines,
                items,
                selected,
                search,
            } => {
                self.show_list_modal(title, lines, items, selected, search);
            }
            InlineCommand::CloseModal => {
                self.close_modal();
            }
            InlineCommand::SetCustomPrompts { registry } => {
                self.set_custom_prompts(registry);
            }
            InlineCommand::LoadFilePalette { files, workspace } => {
                self.load_file_palette(files, workspace);
            }
            InlineCommand::ClearScreen => {
                self.clear_screen();
            }
            InlineCommand::Shutdown => {
                self.request_exit();
            }
        }
        self.mark_dirty();
    }
}

// ============================================================================
// Plan Management
// ============================================================================

impl Session {
    /// Set the task plan for the session.
    ///
    /// This updates the current task plan and marks the session dirty.
    /// The task plan is typically displayed in a timeline pane or overlay.
    ///
    /// # Arguments
    /// * `plan` - The new task plan to display
    fn set_plan(&mut self, plan: TaskPlan) {
        self.render_state.plan = plan;
        self.mark_dirty();
    }
}

// ============================================================================
// Modal Management
// ============================================================================

impl Session {
    /// Show a modal dialog with text content.
    ///
    /// This creates and displays a modal dialog that overlays the main UI.
    /// The modal can optionally include a secure prompt for password entry.
    ///
    /// When a modal is shown:
    /// - Input is disabled (unless secure_prompt is Some)
    /// - Cursor is hidden
    /// - Previous input/cursor state is saved for restoration
    ///
    /// # Arguments
    /// * `title` - The modal title text
    /// * `lines` - Content lines to display in the modal
    /// * `secure_prompt` - Optional secure password prompt configuration
    fn show_modal(
        &mut self,
        title: String,
        lines: Vec<String>,
        secure_prompt: Option<SecurePromptConfig>,
    ) {
        let state = ModalState {
            title,
            lines,
            list: None,
            secure_prompt,
            popup_state: PopupState::default(),
            restore_input: self.ui_state.input_enabled,
            restore_cursor: self.ui_state.cursor_visible,
            search: None,
        };
        if state.secure_prompt.is_none() {
            self.ui_state.input_enabled = false;
        }
        self.ui_state.cursor_visible = false;
        self.render_state.modal = Some(state);
        self.mark_dirty();
    }

    /// Show a modal dialog with a selectable list.
    ///
    /// This creates and displays a modal with both text content and a list
    /// of selectable items. Users can navigate the list with arrow keys and
    /// optionally search/filter the items.
    ///
    /// # Arguments
    /// * `title` - The modal title text
    /// * `lines` - Content lines to display above the list
    /// * `items` - List items to display
    /// * `selected` - Initially selected item (by ID or index)
    /// * `search` - Optional search/filter configuration
    fn show_list_modal(
        &mut self,
        title: String,
        lines: Vec<String>,
        items: Vec<crate::tui::types::InlineListItem>,
        selected: Option<crate::tui::types::InlineListSelection>,
        search: Option<crate::tui::types::InlineListSearchConfig>,
    ) {
        let mut list_state = ModalListState::new(items, selected);
        let search_state = search.map(ModalSearchState::from);
        if let Some(search) = &search_state {
            list_state.apply_search(&search.query);
        }
        let state = ModalState {
            title,
            lines,
            list: Some(list_state),
            secure_prompt: None,
            popup_state: PopupState::default(),
            restore_input: self.ui_state.input_enabled,
            restore_cursor: self.ui_state.cursor_visible,
            search: search_state,
        };
        self.ui_state.input_enabled = false;
        self.ui_state.cursor_visible = false;
        self.render_state.modal = Some(state);
        self.mark_dirty();
    }

    /// Close the currently open modal dialog.
    ///
    /// This removes the modal from the UI and restores the previous input
    /// and cursor state. It also forces a full screen clear and transcript
    /// cache invalidation to ensure the modal is completely removed without
    /// visual artifacts.
    pub(super) fn close_modal(&mut self) {
        if let Some(state) = self.render_state.modal.take() {
            self.ui_state.input_enabled = state.restore_input;
            self.ui_state.cursor_visible = state.restore_cursor;
            // Force full screen clear on next render to remove modal artifacts
            self.ui_state.needs_full_clear = true;
            // Force transcript cache invalidation to ensure full redraw
            self.render_state.transcript_cache = None;
            self.mark_dirty();
        }
    }
}

// ============================================================================
// Screen Management
// ============================================================================

impl Session {
    /// Clear all messages from the screen.
    ///
    /// This removes all message lines from the transcript, resets scroll position,
    /// invalidates caches, and forces a full screen clear on the next render.
    ///
    /// This is typically used when starting a new conversation or when the user
    /// explicitly requests to clear the screen.
    fn clear_screen(&mut self) {
        self.display_state.lines.clear();
        self.scroll_manager.set_offset(0);
        self.invalidate_transcript_cache();
        self.invalidate_scroll_metrics();
        self.ui_state.needs_full_clear = true;
        self.mark_dirty();
    }

    /// Mark the session as needing a redraw.
    ///
    /// This sets the `needs_redraw` flag to true, which signals the render
    /// loop that the UI should be redrawn on the next frame.
    ///
    /// This is called after virtually every state change to ensure the UI
    /// stays in sync with the session state.
    pub fn mark_dirty(&mut self) {
        self.ui_state.needs_redraw = true;
    }
}
