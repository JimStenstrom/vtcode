///! Session State Management
///!
///! Organizes session state into logical groups to reduce the complexity
///! of the monolithic Session struct

use crate::tools::TaskPlan;
use crate::prompts::CustomPromptRegistry;
use super::super::types::{InlineTheme, InlineTextStyle, InlineHeaderContext};
use super::file_palette::FilePalette;
use super::prompt_palette::PromptPalette;
use super::slash_palette::SlashPalette;
use super::modal::ModalState;
use super::transcript::TranscriptReflowCache;
use super::queue::QueueOverlay;
use super::message::{MessageLabels, MessageLine};
use ratatui::widgets::ListState;

/// Manages display and presentation state
pub struct DisplayState {
    /// Theme configuration for styling
    pub theme: InlineTheme,
    /// Message labels configuration
    pub labels: MessageLabels,
    /// Message lines to display
    pub lines: Vec<MessageLine>,
    /// Line revision counter for cache invalidation
    pub line_revision_counter: u64,
    /// Whether we're inside a tool code fence
    pub in_tool_code_fence: bool,
}

impl DisplayState {
    pub fn new(theme: InlineTheme) -> Self {
        Self {
            theme,
            labels: MessageLabels::default(),
            lines: Vec::new(),
            line_revision_counter: 0,
            in_tool_code_fence: false,
        }
    }

    pub fn next_revision(&mut self) -> u64 {
        self.line_revision_counter = self.line_revision_counter.wrapping_add(1);
        self.line_revision_counter
    }
}

/// Manages prompt and input display configuration
pub struct PromptState {
    /// Prefix text displayed before user input
    pub prefix: String,
    /// Style for the prompt prefix
    pub style: InlineTextStyle,
    /// Optional placeholder text when input is empty
    pub placeholder: Option<String>,
    /// Style for placeholder text
    pub placeholder_style: Option<InlineTextStyle>,
    /// Optional status text shown on the left of input
    pub status_left: Option<String>,
    /// Optional status text shown on the right of input
    pub status_right: Option<String>,
}

impl PromptState {
    pub fn new(placeholder: Option<String>, prompt_prefix: String, prompt_style: InlineTextStyle) -> Self {
        let placeholder_style = placeholder.as_ref().map(|_| InlineTextStyle {
            color: Some(super::PLACEHOLDER_COLOR.into()),
            ..InlineTextStyle::default()
        });

        Self {
            prefix: prompt_prefix,
            style: prompt_style,
            placeholder,
            placeholder_style,
            status_left: None,
            status_right: None,
        }
    }

    pub fn has_status(&self) -> bool {
        self.status_left
            .as_ref()
            .or(self.status_right.as_ref())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }
}

/// Manages UI state flags and dimensions
pub struct UIState {
    /// Whether input is currently enabled
    pub input_enabled: bool,
    /// Whether cursor should be visible
    pub cursor_visible: bool,
    /// Whether a full redraw is needed
    pub needs_redraw: bool,
    /// Whether a complete clear is needed
    pub needs_full_clear: bool,
    /// Whether the session should exit
    pub should_exit: bool,
    /// Total rows available for the view
    pub view_rows: u16,
    /// Height of the input area in rows
    pub input_height: u16,
    /// Number of rows for the transcript area
    pub transcript_rows: u16,
    /// Width of the transcript area
    pub transcript_width: u16,
    /// Top line of the transcript view
    pub transcript_view_top: usize,
    /// Whether to show the timeline pane
    pub show_timeline_pane: bool,
}

impl UIState {
    pub fn new(view_rows: u16, input_height: u16, transcript_rows: u16, show_timeline_pane: bool) -> Self {
        Self {
            input_enabled: true,
            cursor_visible: true,
            needs_redraw: true,
            needs_full_clear: false,
            should_exit: false,
            view_rows,
            input_height,
            transcript_rows,
            transcript_width: 0,
            transcript_view_top: 0,
            show_timeline_pane,
        }
    }
}

/// Manages palette-related state (file browser, prompt browser, slash commands)
pub struct PaletteState {
    /// Slash command palette
    pub slash_palette: SlashPalette,
    /// File browser palette (optional, lazy-loaded)
    pub file_palette: Option<FilePalette>,
    /// Whether file palette is currently active
    pub file_palette_active: bool,
    /// Whether to trigger file browser on next update
    pub deferred_file_browser_trigger: bool,
    /// Prompt browser palette (optional, lazy-loaded)
    pub prompt_palette: Option<PromptPalette>,
    /// Whether prompt palette is currently active
    pub prompt_palette_active: bool,
    /// Whether to trigger prompt browser on next update
    pub deferred_prompt_browser_trigger: bool,
    /// Registry of custom prompts
    pub custom_prompts: Option<CustomPromptRegistry>,
}

impl PaletteState {
    pub fn new(slash_palette: SlashPalette) -> Self {
        Self {
            slash_palette,
            file_palette: None,
            file_palette_active: false,
            deferred_file_browser_trigger: false,
            prompt_palette: None,
            prompt_palette_active: false,
            deferred_prompt_browser_trigger: false,
            custom_prompts: None,
        }
    }
}

/// Manages rendering caches and overlays
pub struct RenderState {
    /// Header context for rendering
    pub header_context: InlineHeaderContext,
    /// Number of rows used by the header
    pub header_rows: u16,
    /// Cached transcript reflow data
    pub transcript_cache: Option<TranscriptReflowCache>,
    /// Queued inputs to display
    pub queued_inputs: Vec<String>,
    /// Cached queue overlay rendering
    pub queue_overlay_cache: Option<QueueOverlay>,
    /// Version number for queue overlay cache
    pub queue_overlay_version: u64,
    /// Current modal state (if any)
    pub modal: Option<ModalState>,
    /// Current task plan
    pub plan: TaskPlan,
    /// Navigation list state
    pub navigation_state: ListState,
}

impl RenderState {
    pub fn new(header_context: InlineHeaderContext, header_rows: u16) -> Self {
        Self {
            header_context,
            header_rows,
            transcript_cache: None,
            queued_inputs: Vec::new(),
            queue_overlay_cache: None,
            queue_overlay_version: 0,
            modal: None,
            plan: TaskPlan::default(),
            navigation_state: ListState::default(),
        }
    }
}
