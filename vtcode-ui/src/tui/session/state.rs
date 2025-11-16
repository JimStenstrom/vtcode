//! Session state management
//!
//! This module organizes the Session's state into logical sub-structures,
//! making it easier to understand and maintain the relationships between
//! different aspects of the TUI state.

use ratatui::widgets::ListState;

use crate::prompts::CustomPromptRegistry;
use crate::tools::TaskPlan;

use super::file_palette::FilePalette;
use super::message::{MessageLabels, MessageLine};
use super::modal::ModalState;
use super::prompt_palette::PromptPalette;
use super::queue::QueueOverlay;
use super::slash_palette::SlashPalette;
use super::transcript::TranscriptReflowCache;
use crate::tui::types::{InlineHeaderContext, InlineTextStyle, InlineTheme};

/// Display state for messages and theme
///
/// Manages the core display content including message lines, theme settings,
/// and label management.
pub(super) struct DisplayState {
    /// Message lines to display in the transcript
    pub(super) lines: Vec<MessageLine>,
    /// Theme for inline rendering
    pub(super) theme: InlineTheme,
    /// Labels for categorizing messages
    pub(super) labels: MessageLabels,
    /// Whether we're currently inside a tool code fence
    pub(super) in_tool_code_fence: bool,
}

impl DisplayState {
    pub(super) fn new() -> Self {
        Self {
            lines: Vec::new(),
            theme: InlineTheme::default(),
            labels: MessageLabels::default(),
            in_tool_code_fence: false,
        }
    }
}

/// Prompt and input status display state
///
/// Manages the appearance and content of the prompt area including prefix,
/// styles, placeholder text, and status indicators.
pub(super) struct PromptState {
    /// Prefix shown before the prompt (e.g., "You: ")
    pub(super) prompt_prefix: String,
    /// Style for the prompt prefix
    pub(super) prompt_style: InlineTextStyle,
    /// Placeholder text when input is empty
    pub(super) placeholder: Option<String>,
    /// Style for placeholder text
    pub(super) placeholder_style: Option<InlineTextStyle>,
    /// Status text shown on the left of the input area
    pub(super) input_status_left: Option<String>,
    /// Status text shown on the right of the input area
    pub(super) input_status_right: Option<String>,
}

impl PromptState {
    pub(super) fn new() -> Self {
        Self {
            prompt_prefix: String::new(),
            prompt_style: InlineTextStyle::default(),
            placeholder: None,
            placeholder_style: None,
            input_status_left: None,
            input_status_right: None,
        }
    }

    /// Set the placeholder text and style
    pub(super) fn set_placeholder(&mut self, text: Option<String>, style: Option<InlineTextStyle>) {
        self.placeholder = text;
        self.placeholder_style = style;
    }

    /// Clear all status indicators
    pub(super) fn clear_status(&mut self) {
        self.input_status_left = None;
        self.input_status_right = None;
    }

    /// Check if any status is currently set
    pub(super) fn has_status(&self) -> bool {
        self.input_status_left
            .as_ref()
            .or(self.input_status_right.as_ref())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }
}

/// UI state and viewport metrics
///
/// Manages the state of UI interactions, visibility flags, and viewport dimensions.
pub(super) struct UIState {
    /// State for navigating lists (e.g., slash commands)
    pub(super) navigation_state: ListState,
    /// Whether input is currently enabled
    pub(super) input_enabled: bool,
    /// Whether the cursor should be visible
    pub(super) cursor_visible: bool,
    /// Whether the UI needs to be redrawn
    pub(super) needs_redraw: bool,
    /// Whether a full terminal clear is needed
    pub(super) needs_full_clear: bool,
    /// Whether the session should exit
    pub(super) should_exit: bool,
    /// Total rows available for the view
    pub(super) view_rows: u16,
    /// Height of the input area in rows
    pub(super) input_height: u16,
    /// Height of the transcript area in rows
    pub(super) transcript_rows: u16,
    /// Width of the transcript area in columns
    pub(super) transcript_width: u16,
    /// Top line index of the transcript viewport
    pub(super) transcript_view_top: usize,
    /// Whether to show the timeline pane
    pub(super) show_timeline_pane: bool,
}

impl UIState {
    pub(super) fn new() -> Self {
        Self {
            navigation_state: ListState::default(),
            input_enabled: true,
            cursor_visible: true,
            needs_redraw: false,
            needs_full_clear: false,
            should_exit: false,
            view_rows: 0,
            input_height: 0,
            transcript_rows: 0,
            transcript_width: 0,
            transcript_view_top: 0,
            show_timeline_pane: false,
        }
    }

    /// Request a UI redraw on the next render cycle
    pub(super) fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Request a full terminal clear and redraw
    pub(super) fn request_full_clear(&mut self) {
        self.needs_full_clear = true;
        self.needs_redraw = true;
    }

    /// Request the session to exit
    pub(super) fn request_exit(&mut self) {
        self.should_exit = true;
    }

    /// Check if a redraw is needed and consume the flag
    pub(super) fn consume_redraw_flag(&mut self) -> bool {
        let needs_redraw = self.needs_redraw;
        self.needs_redraw = false;
        needs_redraw
    }

    /// Check if a full clear is needed and consume the flag
    pub(super) fn consume_full_clear_flag(&mut self) -> bool {
        let needs_clear = self.needs_full_clear;
        self.needs_full_clear = false;
        needs_clear
    }

    /// Enable or disable user input
    pub(super) fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
        self.cursor_visible = enabled;
    }
}

/// Palette state for command and file browsers
///
/// Manages the state of various interactive palettes (slash commands, files, prompts)
/// including their active states and deferred triggers.
pub(super) struct PaletteState {
    /// Slash command palette
    pub(super) slash_palette: SlashPalette,
    /// Custom prompt registry
    pub(super) custom_prompts: Option<CustomPromptRegistry>,
    /// File browser palette
    pub(super) file_palette: Option<FilePalette>,
    /// Whether the file palette is currently active
    pub(super) file_palette_active: bool,
    /// Whether to trigger the file browser on the next cycle
    pub(super) deferred_file_browser_trigger: bool,
    /// Prompt browser palette
    pub(super) prompt_palette: Option<PromptPalette>,
    /// Whether the prompt palette is currently active
    pub(super) prompt_palette_active: bool,
    /// Whether to trigger the prompt browser on the next cycle
    pub(super) deferred_prompt_browser_trigger: bool,
}

impl PaletteState {
    pub(super) fn new(slash_palette: SlashPalette) -> Self {
        Self {
            slash_palette,
            custom_prompts: None,
            file_palette: None,
            file_palette_active: false,
            deferred_file_browser_trigger: false,
            prompt_palette: None,
            prompt_palette_active: false,
            deferred_prompt_browser_trigger: false,
        }
    }

    /// Activate the file palette
    pub(super) fn activate_file_palette(&mut self) {
        self.file_palette_active = true;
    }

    /// Deactivate the file palette
    pub(super) fn deactivate_file_palette(&mut self) {
        self.file_palette_active = false;
    }

    /// Activate the prompt palette
    pub(super) fn activate_prompt_palette(&mut self) {
        self.prompt_palette_active = true;
    }

    /// Deactivate the prompt palette
    pub(super) fn deactivate_prompt_palette(&mut self) {
        self.prompt_palette_active = false;
    }

    /// Trigger the file browser on the next cycle
    pub(super) fn defer_file_browser_trigger(&mut self) {
        self.deferred_file_browser_trigger = true;
    }

    /// Trigger the prompt browser on the next cycle
    pub(super) fn defer_prompt_browser_trigger(&mut self) {
        self.deferred_prompt_browser_trigger = true;
    }

    /// Check if any palette is currently active
    pub(super) fn is_any_palette_active(&self) -> bool {
        self.file_palette_active || self.prompt_palette_active
    }
}

/// Render state and caching
///
/// Manages rendering-related state including caches, overlays, modals,
/// and other transient rendering data.
pub(super) struct RenderState {
    /// Context for rendering the header
    pub(super) header_context: InlineHeaderContext,
    /// Number of rows the header occupies
    pub(super) header_rows: u16,
    /// Cache for reflowed transcript content
    pub(super) transcript_cache: Option<TranscriptReflowCache>,
    /// Queued inputs waiting to be processed
    pub(super) queued_inputs: Vec<String>,
    /// Cache for the queue overlay
    pub(super) queue_overlay_cache: Option<QueueOverlay>,
    /// Version counter for queue overlay cache invalidation
    pub(super) queue_overlay_version: u64,
    /// Current modal state, if any
    pub(super) modal: Option<ModalState>,
    /// Current task plan being displayed
    pub(super) plan: TaskPlan,
    /// Counter for line revisions (cache invalidation)
    pub(super) line_revision_counter: u64,
}

impl RenderState {
    pub(super) fn new() -> Self {
        Self {
            header_context: InlineHeaderContext::default(),
            header_rows: 0,
            transcript_cache: None,
            queued_inputs: Vec::new(),
            queue_overlay_cache: None,
            queue_overlay_version: 0,
            modal: None,
            plan: TaskPlan::default(),
            line_revision_counter: 0,
        }
    }

    /// Invalidate the transcript cache
    pub(super) fn invalidate_transcript_cache(&mut self) {
        self.transcript_cache = None;
    }

    /// Invalidate the queue overlay cache
    pub(super) fn invalidate_queue_overlay(&mut self) {
        self.queue_overlay_cache = None;
        self.queue_overlay_version = self.queue_overlay_version.wrapping_add(1);
    }

    /// Get the next revision number for line updates
    pub(super) fn next_revision(&mut self) -> u64 {
        self.line_revision_counter = self.line_revision_counter.wrapping_add(1);
        self.line_revision_counter
    }

    /// Set or clear the modal state
    pub(super) fn set_modal(&mut self, modal: Option<ModalState>) {
        self.modal = modal;
    }

    /// Check if a modal is currently active
    pub(super) fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    /// Add a queued input
    pub(super) fn queue_input(&mut self, input: String) {
        self.queued_inputs.push(input);
        self.invalidate_queue_overlay();
    }

    /// Remove and return the next queued input
    pub(super) fn dequeue_input(&mut self) -> Option<String> {
        if self.queued_inputs.is_empty() {
            None
        } else {
            let input = self.queued_inputs.remove(0);
            self.invalidate_queue_overlay();
            Some(input)
        }
    }

    /// Clear all queued inputs
    pub(super) fn clear_queued_inputs(&mut self) {
        self.queued_inputs.clear();
        self.invalidate_queue_overlay();
    }

    /// Check if there are queued inputs
    pub(super) fn has_queued_inputs(&self) -> bool {
        !self.queued_inputs.is_empty()
    }
}
