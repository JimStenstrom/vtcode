//! Size and boundary limit constants for file operations and content management

/// Chunking constants for large file handling
pub mod chunking {
    /// Maximum lines before triggering chunking for read_file
    pub const MAX_LINES_THRESHOLD: usize = 2_000;

    /// Number of lines to read from start of file when chunking
    pub const CHUNK_START_LINES: usize = 800;

    /// Number of lines to read from end of file when chunking
    pub const CHUNK_END_LINES: usize = 800;

    // DEPRECATED: Terminal output truncation now uses token-based limits instead of line limits
    // See: src/agent/runloop/tool_output/streams.rs (MAX_TOOL_RESPONSE_TOKENS: 25_000)
    // These constants are no longer used and can be safely removed when cleaning up

    /// Maximum content size for write_file before chunking (in bytes)
    pub const MAX_WRITE_CONTENT_SIZE: usize = 500_000; // 500KB

    /// Chunk size for write operations (in bytes)
    pub const WRITE_CHUNK_SIZE: usize = 50_000; // 50KB chunks
}

/// Diff preview controls for file operations
pub mod diff {
    /// Maximum number of bytes allowed in diff preview inputs
    pub const MAX_PREVIEW_BYTES: usize = 200_000;

    /// Number of context lines to include around changes in unified diff output
    pub const CONTEXT_RADIUS: usize = 3;

    /// Maximum number of diff lines to keep in preview output before condensation
    pub const MAX_PREVIEW_LINES: usize = 160;

    /// Number of leading diff lines to retain when condensing previews
    pub const HEAD_LINE_COUNT: usize = 96;

    /// Number of trailing diff lines to retain when condensing previews
    pub const TAIL_LINE_COUNT: usize = 32;
}

/// Project documentation limits
pub mod project_doc {
    pub const DEFAULT_MAX_BYTES: usize = 16 * 1024;
}

/// Instructions file limits
pub mod instructions {
    pub const DEFAULT_MAX_BYTES: usize = 16 * 1024;
}

// Re-export commonly used constants
pub use chunking::{
    CHUNK_END_LINES, CHUNK_START_LINES, MAX_LINES_THRESHOLD, MAX_WRITE_CONTENT_SIZE,
    WRITE_CHUNK_SIZE,
};
pub use diff::{CONTEXT_RADIUS, HEAD_LINE_COUNT, MAX_PREVIEW_BYTES, MAX_PREVIEW_LINES, TAIL_LINE_COUNT};
