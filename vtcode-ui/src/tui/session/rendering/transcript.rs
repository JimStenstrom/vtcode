//! Transcript rendering logic for the session TUI
//!
//! This module handles the rendering of the conversation transcript, including:
//! - Main transcript rendering and layout
//! - Scroll management and viewport calculations
//! - Reflow caching for efficient text wrapping
//! - Metrics tracking for layout and performance

use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use vtcode_config::constants::ui;

use super::super::{Session, transcript::TranscriptReflowCache};

impl Session {
    /// Renders the conversation transcript in the given area.
    ///
    /// This is the main entry point for transcript rendering. It handles:
    /// - Setting up the rendering area and block borders
    /// - Calculating viewport dimensions and scroll positions
    /// - Collecting visible lines from the reflow cache
    /// - Overlaying queued input messages
    /// - Rendering the final paragraph widget
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render into
    /// * `area` - The rectangular area to render the transcript in
    pub(in crate::tui::session) fn render_transcript(&mut self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Create the border block
        let block = Block::default()
            .borders(Borders::NONE)
            .border_type(BorderType::Rounded)
            .style(self.default_style())
            .border_style(self.border_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Update transcript dimensions
        self.apply_transcript_rows(inner.height);

        let content_width = inner.width;
        if content_width == 0 {
            return;
        }
        self.apply_transcript_width(content_width);

        // Calculate scroll position
        let viewport_rows = inner.height as usize;
        let padding = usize::from(ui::INLINE_TRANSCRIPT_BOTTOM_PADDING);
        let effective_padding = padding.min(viewport_rows.saturating_sub(1));
        let total_rows = self.total_transcript_rows(content_width) + effective_padding;
        let (top_offset, _clamped_total_rows) =
            self.prepare_transcript_scroll(total_rows, viewport_rows);
        let vertical_offset = top_offset.min(self.scroll_manager.max_offset());
        self.transcript_view_top = vertical_offset;

        // Collect visible lines
        let visible_start = vertical_offset;
        let scroll_area = Rect::new(inner.x, inner.y, content_width, inner.height);
        let mut visible_lines =
            self.collect_transcript_window(content_width, visible_start, viewport_rows);

        // Fill remaining space with empty lines
        let fill_count = viewport_rows.saturating_sub(visible_lines.len());
        if fill_count > 0 {
            let target_len = visible_lines.len() + fill_count;
            visible_lines.resize_with(target_len, Line::default);
        }

        // Overlay queued inputs if any
        self.overlay_queue_lines(&mut visible_lines, content_width);

        // Render the final paragraph
        let paragraph = Paragraph::new(visible_lines)
            .style(self.default_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(Clear, scroll_area);
        frame.render_widget(paragraph, scroll_area);
    }

    // ========================================================================
    // Layout and Metrics
    // ========================================================================

    /// Updates the transcript row count and invalidates scroll metrics if changed.
    ///
    /// This ensures the transcript viewport is at least 1 row tall and triggers
    /// scroll recalculation when dimensions change.
    ///
    /// # Arguments
    ///
    /// * `rows` - The new row count for the transcript viewport
    pub(super) fn apply_transcript_rows(&mut self, rows: u16) {
        let resolved = rows.max(1);
        if self.transcript_rows != resolved {
            self.transcript_rows = resolved;
            self.invalidate_scroll_metrics();
        }
    }

    /// Updates the transcript width and invalidates scroll metrics if changed.
    ///
    /// Width changes require reflow of all text content, so this invalidates
    /// the scroll metrics to trigger recalculation.
    ///
    /// # Arguments
    ///
    /// * `width` - The new width for the transcript content area
    pub(super) fn apply_transcript_width(&mut self, width: u16) {
        if self.transcript_width != width {
            self.transcript_width = width;
            self.invalidate_scroll_metrics();
        }
    }

    /// Returns the number of rows reserved for the header section.
    ///
    /// This is the maximum of the current header rows and the minimum header height.
    pub(super) fn header_reserved_rows(&self) -> u16 {
        self.header_rows.max(ui::INLINE_HEADER_HEIGHT)
    }

    /// Returns the total number of rows reserved for header and input sections.
    ///
    /// This is used to calculate how much space is available for the transcript.
    pub(super) fn input_reserved_rows(&self) -> u16 {
        self.header_reserved_rows() + self.input_height
    }

    /// Recalculates the transcript row count based on available viewport space.
    ///
    /// This accounts for header, input, and border rows to determine how much
    /// vertical space is available for the transcript display.
    pub(in crate::tui::session) fn recalculate_transcript_rows(&mut self) {
        let reserved = self.input_reserved_rows().saturating_add(2); // account for transcript block borders
        let available = self.view_rows.saturating_sub(reserved).max(1);
        self.apply_transcript_rows(available);
    }

    // ========================================================================
    // Scroll Management
    // ========================================================================

    /// Returns the current viewport height in rows.
    ///
    /// This is used for scroll calculations and ensures a minimum of 1 row.
    pub(in crate::tui::session) fn viewport_height(&self) -> usize {
        self.transcript_rows.max(1) as usize
    }

    /// Returns the current maximum scroll offset, ensuring metrics are up to date.
    ///
    /// This will recalculate scroll metrics if they've been invalidated.
    pub(in crate::tui::session) fn current_max_scroll_offset(&mut self) -> usize {
        self.ensure_scroll_metrics();
        self.scroll_manager.max_offset()
    }

    /// Ensures the current scroll offset is within valid bounds.
    ///
    /// If the scroll offset exceeds the maximum allowed, it will be clamped
    /// to the maximum offset.
    pub(in crate::tui::session) fn enforce_scroll_bounds(&mut self) {
        let max_offset = self.current_max_scroll_offset();
        if self.scroll_manager.offset() > max_offset {
            self.scroll_manager.set_offset(max_offset);
        }
    }

    /// Invalidates scroll metrics, forcing recalculation on next access.
    ///
    /// This also invalidates the transcript cache since layout may have changed.
    pub(in crate::tui::session) fn invalidate_scroll_metrics(&mut self) {
        self.scroll_manager.invalidate_metrics();
        self.invalidate_transcript_cache();
    }

    /// Ensures scroll metrics are up to date, recalculating if necessary.
    ///
    /// This will recompute the total scrollable rows based on current
    /// transcript content and viewport dimensions.
    pub(super) fn ensure_scroll_metrics(&mut self) {
        if self.scroll_manager.metrics_valid() {
            return;
        }

        let viewport_rows = self.viewport_height();
        if self.transcript_width == 0 || viewport_rows == 0 {
            let total_rows = self.lines.len().saturating_sub(viewport_rows.max(1));
            self.scroll_manager.set_total_rows(total_rows);
            return;
        }

        let padding = usize::from(ui::INLINE_TRANSCRIPT_BOTTOM_PADDING);
        let effective_padding = padding.min(viewport_rows.saturating_sub(1));
        let total_rows = self.total_transcript_rows(self.transcript_width) + effective_padding;
        self.scroll_manager.set_total_rows(total_rows);
    }

    /// Prepares scroll state for transcript rendering.
    ///
    /// This calculates the top offset based on current scroll position and
    /// total content height. Returns the calculated offsets for rendering.
    ///
    /// # Arguments
    ///
    /// * `total_rows` - Total number of rows in the transcript content
    /// * `viewport_rows` - Number of visible rows in the viewport
    ///
    /// # Returns
    ///
    /// A tuple of (top_offset, clamped_total_rows) for rendering calculations
    pub(super) fn prepare_transcript_scroll(
        &mut self,
        total_rows: usize,
        viewport_rows: usize,
    ) -> (usize, usize) {
        let viewport = viewport_rows.max(1);
        let clamped_total = total_rows.max(1);
        self.scroll_manager.set_total_rows(clamped_total);
        self.scroll_manager.set_viewport_rows(viewport as u16);
        let max_offset = self.scroll_manager.max_offset();

        if self.scroll_manager.offset() > max_offset {
            self.scroll_manager.set_offset(max_offset);
        }

        let top_offset = max_offset.saturating_sub(self.scroll_manager.offset());
        (top_offset, clamped_total)
    }

    // ========================================================================
    // Reflow and Caching
    // ========================================================================

    /// Invalidates the transcript reflow cache.
    ///
    /// This forces all message lines to be reflowed on the next render,
    /// typically called when content or layout changes.
    pub(in crate::tui::session) fn invalidate_transcript_cache(&mut self) {
        self.transcript_cache = None;
    }

    /// Ensures the reflow cache is up to date for the given width.
    ///
    /// This manages the transcript reflow cache, updating it when:
    /// - The width changes
    /// - Messages are added or modified (tracked by revision numbers)
    /// - The cache doesn't exist yet
    ///
    /// The cache stores pre-wrapped lines for each message to avoid
    /// re-wrapping on every render. It uses revision tracking to detect
    /// when individual messages need to be reflowed.
    ///
    /// # Arguments
    ///
    /// * `width` - The target width for text wrapping
    ///
    /// # Returns
    ///
    /// A mutable reference to the updated cache
    pub(super) fn ensure_reflow_cache(&mut self, width: u16) -> &mut TranscriptReflowCache {
        let mut cache = self
            .transcript_cache
            .take()
            .unwrap_or_else(|| TranscriptReflowCache::new(width));

        // Update width if needed and handle width changes
        if cache.width != width {
            cache.set_width(width);
        }

        // Resize message cache to match current line count
        while cache.messages.len() > self.lines.len() {
            cache.messages.pop();
        }
        while cache.messages.len() < self.lines.len() {
            cache.messages.push(Default::default());
        }

        // Process any dirty messages (those that need reflow)
        let mut first_dirty = self.lines.len(); // Start with all clean

        // Find the first message that needs reflow
        for (index, line) in self.lines.iter().enumerate() {
            if cache.needs_reflow(index, line.revision) {
                first_dirty = index;
                break;
            }
        }

        // If no messages need reflow, just return existing cache
        if first_dirty == self.lines.len() {
            // Still need to ensure row offsets are correct
            cache.update_row_offsets();
            self.transcript_cache = Some(cache);
            return self.transcript_cache.as_mut().unwrap();
        }

        // Update all messages from the first dirty one onwards
        for index in first_dirty..self.lines.len() {
            if index < self.lines.len() {
                let line = &self.lines[index];
                if cache.needs_reflow(index, line.revision) {
                    let new_lines = self.reflow_message_lines(index, width);
                    cache.update_message(index, line.revision, new_lines);
                }
            }
        }

        // Update row offsets and total row count
        cache.update_row_offsets();
        self.transcript_cache = Some(cache);
        self.transcript_cache.as_mut().unwrap()
    }

    /// Returns the total number of rows in the transcript after reflow.
    ///
    /// This accounts for line wrapping at the given width. The result is cached
    /// and only recalculated when the cache is invalidated.
    ///
    /// # Arguments
    ///
    /// * `width` - The width to calculate row count for
    ///
    /// # Returns
    ///
    /// The total number of rows needed to display all transcript content
    pub(super) fn total_transcript_rows(&mut self, width: u16) -> usize {
        if self.lines.is_empty() {
            return 0;
        }
        let cache = self.ensure_reflow_cache(width);
        cache.total_rows()
    }

    /// Collects a window of visible transcript lines for rendering.
    ///
    /// This retrieves lines from the reflow cache for the specified viewport
    /// window, avoiding the need to reflow content on every render.
    ///
    /// # Arguments
    ///
    /// * `width` - The width for text wrapping
    /// * `start_row` - The first row to include (0-indexed)
    /// * `max_rows` - Maximum number of rows to return
    ///
    /// # Returns
    ///
    /// A vector of lines to display in the viewport
    pub(super) fn collect_transcript_window(
        &mut self,
        width: u16,
        start_row: usize,
        max_rows: usize,
    ) -> Vec<Line<'static>> {
        if max_rows == 0 {
            return Vec::new();
        }
        let cache = self.ensure_reflow_cache(width);

        // Use the optimized method from the TranscriptReflowCache
        cache.get_visible_range(start_row, max_rows)
    }

    /// Reflows all transcript lines for testing purposes.
    ///
    /// This is only used in tests to verify the reflow logic. In production,
    /// use `collect_transcript_window` for efficient partial rendering.
    ///
    /// # Arguments
    ///
    /// * `width` - The width to reflow text to
    ///
    /// # Returns
    ///
    /// All reflowed lines in the transcript
    #[cfg(test)]
    pub(super) fn reflow_transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        if width == 0 {
            let mut lines: Vec<Line<'static>> = Vec::new();
            for index in 0..self.lines.len() {
                lines.extend(self.reflow_message_lines(index, 0));
            }
            if lines.is_empty() {
                lines.push(Line::default());
            }
            return lines;
        }

        let mut wrapped_lines = Vec::new();
        for index in 0..self.lines.len() {
            wrapped_lines.extend(self.reflow_message_lines(index, width));
        }

        if wrapped_lines.is_empty() {
            wrapped_lines.push(Line::default());
        }

        wrapped_lines
    }
}

// Tests are in the main session.rs test module since they require
// access to Session's internal state and test infrastructure
