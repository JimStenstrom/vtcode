///! Session State Management
///!
///! Organizes session state into logical groups to reduce the complexity
///! of the monolithic Session struct.
///!
///! All state structs use private fields with accessor methods to:
///! - Enforce invariants
///! - Provide validation
///! - Enable future changes without breaking API
///! - Make cache invalidation explicit

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
    theme: InlineTheme,
    /// Message labels configuration
    labels: MessageLabels,
    /// Message lines to display
    lines: Vec<MessageLine>,
    /// Line revision counter for cache invalidation
    line_revision_counter: u64,
    /// Whether we're inside a tool code fence
    in_tool_code_fence: bool,
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

    // Theme accessors
    pub fn theme(&self) -> &InlineTheme {
        &self.theme
    }

    pub fn theme_mut(&mut self) -> &mut InlineTheme {
        &mut self.theme
    }

    pub fn set_theme(&mut self, theme: InlineTheme) {
        self.theme = theme;
    }

    // Labels accessors
    pub fn labels(&self) -> &MessageLabels {
        &self.labels
    }

    pub fn labels_mut(&mut self) -> &mut MessageLabels {
        &mut self.labels
    }

    pub fn set_labels(&mut self, labels: MessageLabels) {
        self.labels = labels;
    }

    // Lines accessors (read-only slice, mutable vec for internal use)
    pub fn lines(&self) -> &[MessageLine] {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut Vec<MessageLine> {
        &mut self.lines
    }

    // Revision counter (encapsulated - can only increment)
    pub fn current_revision(&self) -> u64 {
        self.line_revision_counter
    }

    pub fn next_revision(&mut self) -> u64 {
        self.line_revision_counter = self.line_revision_counter.wrapping_add(1);
        self.line_revision_counter
    }

    // Tool code fence state
    pub fn is_in_tool_code_fence(&self) -> bool {
        self.in_tool_code_fence
    }

    pub fn set_in_tool_code_fence(&mut self, value: bool) {
        self.in_tool_code_fence = value;
    }
}

/// Manages prompt and input display configuration
pub struct PromptState {
    /// Prefix text displayed before user input
    prefix: String,
    /// Style for the prompt prefix
    style: InlineTextStyle,
    /// Optional placeholder text when input is empty
    placeholder: Option<String>,
    /// Style for placeholder text
    placeholder_style: Option<InlineTextStyle>,
    /// Optional status text shown on the left of input
    status_left: Option<String>,
    /// Optional status text shown on the right of input
    status_right: Option<String>,
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

    // Prefix accessors
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }

    // Style accessors
    pub fn style(&self) -> &InlineTextStyle {
        &self.style
    }

    pub fn set_style(&mut self, style: InlineTextStyle) {
        self.style = style;
    }

    // Placeholder accessors
    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn set_placeholder(&mut self, placeholder: Option<String>) {
        self.placeholder = placeholder;
    }

    pub fn placeholder_style(&self) -> Option<&InlineTextStyle> {
        self.placeholder_style.as_ref()
    }

    pub fn set_placeholder_style(&mut self, style: Option<InlineTextStyle>) {
        self.placeholder_style = style;
    }

    // Status accessors with validation
    pub fn status_left(&self) -> Option<&str> {
        self.status_left.as_deref()
    }

    pub fn set_status_left(&mut self, status: Option<String>) {
        self.status_left = status;
    }

    pub fn status_right(&self) -> Option<&str> {
        self.status_right.as_deref()
    }

    pub fn set_status_right(&mut self, status: Option<String>) {
        self.status_right = status;
    }

    // Convenience methods
    pub fn has_left_status(&self) -> bool {
        self.status_left
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn has_right_status(&self) -> bool {
        self.status_right
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn has_any_status(&self) -> bool {
        self.has_left_status() || self.has_right_status()
    }

    /// Legacy method for compatibility
    pub fn has_status(&self) -> bool {
        self.has_any_status()
    }
}

/// Manages UI state flags and dimensions
pub struct UIState {
    /// Whether input is currently enabled
    input_enabled: bool,
    /// Whether cursor should be visible
    cursor_visible: bool,
    /// Whether a full redraw is needed
    needs_redraw: bool,
    /// Whether a complete clear is needed
    needs_full_clear: bool,
    /// Whether the session should exit
    should_exit: bool,
    /// Total rows available for the view
    view_rows: u16,
    /// Height of the input area in rows
    input_height: u16,
    /// Number of rows for the transcript area
    transcript_rows: u16,
    /// Width of the transcript area
    transcript_width: u16,
    /// Top line of the transcript view
    transcript_view_top: usize,
    /// Whether to show the timeline pane
    show_timeline_pane: bool,
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

    // Boolean flags accessors
    pub fn is_input_enabled(&self) -> bool {
        self.input_enabled
    }

    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn clear_redraw_flag(&mut self) {
        self.needs_redraw = false;
    }

    pub fn take_redraw(&mut self) -> bool {
        let needs = self.needs_redraw;
        self.needs_redraw = false;
        needs
    }

    pub fn needs_full_clear(&self) -> bool {
        self.needs_full_clear
    }

    pub fn set_needs_full_clear(&mut self, needs: bool) {
        self.needs_full_clear = needs;
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    // Dimension accessors
    pub fn view_rows(&self) -> u16 {
        self.view_rows
    }

    pub fn set_view_rows(&mut self, rows: u16) {
        self.view_rows = rows;
    }

    pub fn input_height(&self) -> u16 {
        self.input_height
    }

    pub fn set_input_height(&mut self, height: u16) {
        self.input_height = height;
    }

    pub fn transcript_rows(&self) -> u16 {
        self.transcript_rows
    }

    pub fn set_transcript_rows(&mut self, rows: u16) {
        self.transcript_rows = rows;
    }

    pub fn transcript_width(&self) -> u16 {
        self.transcript_width
    }

    pub fn set_transcript_width(&mut self, width: u16) {
        self.transcript_width = width;
    }

    pub fn transcript_view_top(&self) -> usize {
        self.transcript_view_top
    }

    pub fn set_transcript_view_top(&mut self, top: usize) {
        self.transcript_view_top = top;
    }

    pub fn show_timeline_pane(&self) -> bool {
        self.show_timeline_pane
    }

    pub fn set_show_timeline_pane(&mut self, show: bool) {
        self.show_timeline_pane = show;
    }
}

/// Manages palette-related state (file browser, prompt browser, slash commands)
pub struct PaletteState {
    /// Slash command palette
    slash_palette: SlashPalette,
    /// File browser palette (optional, lazy-loaded)
    file_palette: Option<FilePalette>,
    /// Whether file palette is currently active
    file_palette_active: bool,
    /// Whether to trigger file browser on next update
    deferred_file_browser_trigger: bool,
    /// Prompt browser palette (optional, lazy-loaded)
    prompt_palette: Option<PromptPalette>,
    /// Whether prompt palette is currently active
    prompt_palette_active: bool,
    /// Whether to trigger prompt browser on next update
    deferred_prompt_browser_trigger: bool,
    /// Registry of custom prompts
    custom_prompts: Option<CustomPromptRegistry>,
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

    // Slash palette accessors
    pub fn slash_palette(&self) -> &SlashPalette {
        &self.slash_palette
    }

    pub fn slash_palette_mut(&mut self) -> &mut SlashPalette {
        &mut self.slash_palette
    }

    // File palette accessors
    pub fn file_palette(&self) -> Option<&FilePalette> {
        self.file_palette.as_ref()
    }

    pub fn file_palette_mut(&mut self) -> Option<&mut FilePalette> {
        self.file_palette.as_mut()
    }

    pub fn set_file_palette(&mut self, palette: Option<FilePalette>) {
        self.file_palette = palette;
    }

    pub fn is_file_palette_active(&self) -> bool {
        self.file_palette_active
    }

    pub fn set_file_palette_active(&mut self, active: bool) {
        self.file_palette_active = active;
    }

    pub fn has_deferred_file_browser_trigger(&self) -> bool {
        self.deferred_file_browser_trigger
    }

    pub fn set_deferred_file_browser_trigger(&mut self, trigger: bool) {
        self.deferred_file_browser_trigger = trigger;
    }

    // Prompt palette accessors
    pub fn prompt_palette(&self) -> Option<&PromptPalette> {
        self.prompt_palette.as_ref()
    }

    pub fn prompt_palette_mut(&mut self) -> Option<&mut PromptPalette> {
        self.prompt_palette.as_mut()
    }

    pub fn set_prompt_palette(&mut self, palette: Option<PromptPalette>) {
        self.prompt_palette = palette;
    }

    pub fn is_prompt_palette_active(&self) -> bool {
        self.prompt_palette_active
    }

    pub fn set_prompt_palette_active(&mut self, active: bool) {
        self.prompt_palette_active = active;
    }

    pub fn has_deferred_prompt_browser_trigger(&self) -> bool {
        self.deferred_prompt_browser_trigger
    }

    pub fn set_deferred_prompt_browser_trigger(&mut self, trigger: bool) {
        self.deferred_prompt_browser_trigger = trigger;
    }

    // Custom prompts accessors
    pub fn custom_prompts(&self) -> Option<&CustomPromptRegistry> {
        self.custom_prompts.as_ref()
    }

    pub fn custom_prompts_mut(&mut self) -> Option<&mut CustomPromptRegistry> {
        self.custom_prompts.as_mut()
    }

    pub fn set_custom_prompts(&mut self, prompts: Option<CustomPromptRegistry>) {
        self.custom_prompts = prompts;
    }
}

/// Manages rendering caches and overlays
pub struct RenderState {
    /// Header context for rendering
    header_context: InlineHeaderContext,
    /// Number of rows used by the header
    header_rows: u16,
    /// Cached transcript reflow data
    transcript_cache: Option<TranscriptReflowCache>,
    /// Queued inputs to display
    queued_inputs: Vec<String>,
    /// Cached queue overlay rendering
    queue_overlay_cache: Option<QueueOverlay>,
    /// Version number for queue overlay cache
    queue_overlay_version: u64,
    /// Current modal state (if any)
    modal: Option<ModalState>,
    /// Current task plan
    plan: TaskPlan,
    /// Navigation list state
    navigation_state: ListState,
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

    // Header accessors
    pub fn header_context(&self) -> &InlineHeaderContext {
        &self.header_context
    }

    pub fn header_context_mut(&mut self) -> &mut InlineHeaderContext {
        &mut self.header_context
    }

    pub fn set_header_context(&mut self, context: InlineHeaderContext) {
        self.header_context = context;
    }

    pub fn header_rows(&self) -> u16 {
        self.header_rows
    }

    pub fn set_header_rows(&mut self, rows: u16) {
        self.header_rows = rows;
    }

    // Transcript cache accessors
    pub fn transcript_cache(&self) -> Option<&TranscriptReflowCache> {
        self.transcript_cache.as_ref()
    }

    pub fn transcript_cache_mut(&mut self) -> &mut Option<TranscriptReflowCache> {
        &mut self.transcript_cache
    }

    pub fn take_transcript_cache(&mut self) -> Option<TranscriptReflowCache> {
        self.transcript_cache.take()
    }

    pub fn set_transcript_cache(&mut self, cache: Option<TranscriptReflowCache>) {
        self.transcript_cache = cache;
    }

    // Queued inputs accessors
    pub fn queued_inputs(&self) -> &[String] {
        &self.queued_inputs
    }

    pub fn queued_inputs_mut(&mut self) -> &mut Vec<String> {
        &mut self.queued_inputs
    }

    // Queue overlay cache accessors
    pub fn queue_overlay_cache(&self) -> Option<&QueueOverlay> {
        self.queue_overlay_cache.as_ref()
    }

    pub fn set_queue_overlay_cache(&mut self, cache: Option<QueueOverlay>) {
        self.queue_overlay_cache = cache;
    }

    pub fn queue_overlay_version(&self) -> u64 {
        self.queue_overlay_version
    }

    pub fn set_queue_overlay_version(&mut self, version: u64) {
        self.queue_overlay_version = version;
    }

    // Modal accessors
    pub fn modal(&self) -> Option<&ModalState> {
        self.modal.as_ref()
    }

    pub fn modal_mut(&mut self) -> Option<&mut ModalState> {
        self.modal.as_mut()
    }

    pub fn set_modal(&mut self, modal: Option<ModalState>) {
        self.modal = modal;
    }

    pub fn take_modal(&mut self) -> Option<ModalState> {
        self.modal.take()
    }

    // Plan accessors
    pub fn plan(&self) -> &TaskPlan {
        &self.plan
    }

    pub fn plan_mut(&mut self) -> &mut TaskPlan {
        &mut self.plan
    }

    pub fn set_plan(&mut self, plan: TaskPlan) {
        self.plan = plan;
    }

    // Navigation state accessors
    pub fn navigation_state(&self) -> &ListState {
        &self.navigation_state
    }

    pub fn navigation_state_mut(&mut self) -> &mut ListState {
        &mut self.navigation_state
    }
}
