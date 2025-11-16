//! Palette coordination for file references, prompt references, and slash commands.
//!
//! This module coordinates all palette-related functionality in the session:
//!
//! ## Palette Types
//!
//! 1. **File Palette** (@-references)
//!    - Triggered by typing `@` followed by text
//!    - Shows fuzzy-matched file list from workspace
//!    - Inserts file references into input
//!
//! 2. **Prompt Palette** (/-prefix or /prompt:)
//!    - Triggered by typing `/prompt:` or legacy `/prompts:`
//!    - Shows available custom and builtin prompts
//!    - Inserts prompt references into input
//!
//! 3. **Slash Palette** (slash commands)
//!    - Triggered by typing `/` at start of input
//!    - Shows available slash commands
//!    - Managed by the slash module
//!
//! ## Architecture
//!
//! This module serves as a coordinator that:
//! - Detects when palettes should open/close based on input
//! - Manages palette lifecycle (load, trigger, insert, close)
//! - Delegates to specific palette implementations (FilePalette, PromptPalette, SlashPalette)
//! - Provides helpers for reference insertion and input management
//!
//! ## Usage Pattern
//!
//! Palettes are checked in the following order during key events:
//! 1. Modal dialogs (highest priority)
//! 2. File palette
//! 3. Prompt palette
//! 4. Slash palette
//! 5. Regular input
//!
//! This ensures that only one palette is active at a time and provides
//! a consistent user experience across different reference types.

use std::path::PathBuf;

use crate::prompts::CustomPromptRegistry;
use super::Session;
use super::file_palette::{FilePalette, extract_file_reference};
use super::prompt_palette::{PromptPalette, extract_prompt_reference};
use super::PROMPT_COMMAND_PREFIX;
use vtcode_config::constants::prompts;

// ============================================================================
// File Palette Management
// ============================================================================

impl Session {
    /// Load the file palette with workspace files.
    ///
    /// This initializes the file palette with a list of files from the workspace.
    /// The palette is not immediately active - it activates when a file reference
    /// trigger is detected (@ character followed by text).
    ///
    /// # Arguments
    /// * `files` - List of file paths relative to the workspace
    /// * `workspace` - Root workspace directory path
    pub(super) fn load_file_palette(&mut self, files: Vec<String>, workspace: PathBuf) {
        let mut palette = FilePalette::new(workspace);
        palette.load_files(files);
        self.palette_state.file_palette = Some(palette);
        self.palette_state.file_palette_active = false;
        self.check_file_reference_trigger();
    }

    /// Check if the current input should trigger the file reference palette.
    ///
    /// This examines the input at the current cursor position to detect if the user
    /// is typing a file reference (e.g., "@filename"). If a reference pattern is found,
    /// the file palette becomes active and filters results based on the query.
    ///
    /// The pattern for file references is: `@` followed by optional text.
    /// The palette closes automatically if the cursor moves outside the reference.
    pub(super) fn check_file_reference_trigger(&mut self) {
        if let Some(palette) = self.palette_state.file_palette.as_mut() {
            if let Some((_, _, query)) =
                extract_file_reference(self.input_manager.content(), self.input_manager.cursor())
            {
                // Reset selection and clear previous state when opening
                palette.reset();
                palette.set_filter(query);
                self.palette_state.file_palette_active = true;
            } else {
                self.palette_state.file_palette_active = false;
            }
        }
    }

    /// Close the file palette and free resources.
    ///
    /// This deactivates the file palette and cleans up any allocated resources.
    /// The palette remains loaded and can be reactivated later without reloading files.
    pub(super) fn close_file_palette(&mut self) {
        self.palette_state.file_palette_active = false;

        // Clean up resources when closing to free memory
        if let Some(palette) = self.palette_state.file_palette.as_mut() {
            palette.cleanup();
        }
    }

    /// Handle keyboard events for the file palette.
    ///
    /// This processes navigation and selection keys when the file palette is active.
    /// Returns true if the key was handled, false otherwise.
    ///
    /// # Arguments
    /// * `key` - The keyboard event to process
    ///
    /// # Returns
    /// `true` if the key was handled by the palette
    pub(super) fn handle_file_palette_key(&mut self, key: &crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        if !self.palette_state.file_palette_active {
            return false;
        }

        let Some(palette) = self.palette_state.file_palette.as_mut() else {
            return false;
        };

        match key.code {
            KeyCode::Up => {
                palette.move_selection_up();
                self.mark_dirty();
                true
            }
            KeyCode::Down => {
                palette.move_selection_down();
                self.mark_dirty();
                true
            }
            KeyCode::PageUp => {
                palette.page_up();
                self.mark_dirty();
                true
            }
            KeyCode::PageDown => {
                palette.page_down();
                self.mark_dirty();
                true
            }
            KeyCode::Home => {
                palette.move_to_first();
                self.mark_dirty();
                true
            }
            KeyCode::End => {
                palette.move_to_last();
                self.mark_dirty();
                true
            }
            KeyCode::Esc => {
                self.close_file_palette();
                self.mark_dirty();
                true
            }
            KeyCode::Tab => {
                if let Some(entry) = palette.get_selected() {
                    let path = entry.relative_path.clone();
                    self.insert_file_reference(&path);
                    self.close_file_palette();
                    self.mark_dirty();
                }
                true
            }
            KeyCode::Enter => {
                if let Some(entry) = palette.get_selected() {
                    let path = entry.relative_path.clone();
                    self.insert_file_reference(&path);
                    self.close_file_palette();
                    self.mark_dirty();
                }
                true
            }
            _ => false,
        }
    }

    /// Insert a file reference into the input at the current cursor position.
    ///
    /// This replaces the current file reference pattern with the full path.
    ///
    /// # Arguments
    /// * `file_path` - The file path to insert
    fn insert_file_reference(&mut self, file_path: &str) {
        if let Some((start, end, _)) =
            extract_file_reference(self.input_manager.content(), self.input_manager.cursor())
        {
            let replacement = format!("@{}", file_path);
            let content = self.input_manager.content().to_string();
            let mut new_content = String::new();
            new_content.push_str(&content[..start]);
            new_content.push_str(&replacement);
            new_content.push_str(&content[end..]);
            self.input_manager.set_content(new_content);
            self.input_manager.set_cursor(start + replacement.len());
            self.input_manager.insert_char(' ');
        }
    }
}

// ============================================================================
// Prompt Palette Management
// ============================================================================

impl Session {
    /// Check if the current input should trigger the prompt reference palette.
    ///
    /// This examines the input at the current cursor position to detect if the user
    /// is typing a prompt reference (e.g., "/prompt:name" or legacy "/prompts:name").
    ///
    /// The palette is lazily initialized on first use, loading prompts from:
    /// 1. Custom prompts registry (if available)
    /// 2. .vtcode/prompts directory (fallback)
    /// 3. Core builtin prompts directory
    /// 4. Builtin prompts from registry
    ///
    /// Once initialized, the palette filters results based on the query text.
    pub(super) fn check_prompt_reference_trigger(&mut self) {
        // Initialize prompt palette on-demand if it doesn't exist
        if self.palette_state.prompt_palette.is_none() {
            let mut palette = PromptPalette::new();

            // Try loading from custom_prompts first
            let loaded = if let Some(ref custom_prompts) = self.palette_state.custom_prompts {
                if custom_prompts.enabled() && !custom_prompts.is_empty() {
                    palette.load_prompts(custom_prompts.iter());
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Fallback: load directly from filesystem if custom_prompts not available
            if !loaded {
                // Try default .vtcode/prompts directory
                if let Ok(current_dir) = std::env::current_dir() {
                    let prompts_dir = current_dir.join(".vtcode").join("prompts");
                    palette.load_from_directory(&prompts_dir);
                }
            }

            if let Ok(current_dir) = std::env::current_dir() {
                let core_dir = current_dir.join(prompts::CORE_BUILTIN_PROMPTS_DIR);
                palette.load_from_directory(&core_dir);
            }

            let builtin_prompts = CustomPromptRegistry::builtin_prompts();
            if !builtin_prompts.is_empty() {
                palette.append_custom_prompts(builtin_prompts.iter());
            }

            self.palette_state.prompt_palette = Some(palette);
        }

        if let Some(palette) = self.palette_state.prompt_palette.as_mut() {
            if let Some((_, _, query)) =
                extract_prompt_reference(self.input_manager.content(), self.input_manager.cursor())
            {
                // Reset selection and clear previous state when opening
                palette.reset();
                palette.set_filter(query);
                self.palette_state.prompt_palette_active = true;
            } else {
                self.palette_state.prompt_palette_active = false;
            }
        }
    }

    /// Close the prompt palette and free resources.
    ///
    /// This deactivates the prompt palette and cleans up any allocated resources.
    /// The palette remains loaded and can be reactivated later without reloading prompts.
    pub(super) fn close_prompt_palette(&mut self) {
        self.palette_state.prompt_palette_active = false;

        // Clean up resources when closing to free memory
        if let Some(palette) = self.palette_state.prompt_palette.as_mut() {
            palette.cleanup();
        }
    }

    /// Handle keyboard events for the prompt palette.
    ///
    /// This processes navigation and selection keys when the prompt palette is active.
    /// Returns true if the key was handled, false otherwise.
    ///
    /// # Arguments
    /// * `key` - The keyboard event to process
    ///
    /// # Returns
    /// `true` if the key was handled by the palette
    pub(super) fn handle_prompt_palette_key(&mut self, key: &crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        if !self.palette_state.prompt_palette_active {
            return false;
        }

        let Some(palette) = self.palette_state.prompt_palette.as_mut() else {
            return false;
        };

        match key.code {
            KeyCode::Up => {
                palette.move_selection_up();
                self.mark_dirty();
                true
            }
            KeyCode::Down => {
                palette.move_selection_down();
                self.mark_dirty();
                true
            }
            KeyCode::PageUp => {
                palette.page_up();
                self.mark_dirty();
                true
            }
            KeyCode::PageDown => {
                palette.page_down();
                self.mark_dirty();
                true
            }
            KeyCode::Home => {
                palette.move_to_first();
                self.mark_dirty();
                true
            }
            KeyCode::End => {
                palette.move_to_last();
                self.mark_dirty();
                true
            }
            KeyCode::Esc => {
                self.close_prompt_palette();
                self.mark_dirty();
                true
            }
            KeyCode::Tab | KeyCode::Enter => {
                if let Some(entry) = palette.get_selected() {
                    let prompt_name = entry.name.clone();
                    self.insert_prompt_reference(&prompt_name);
                    self.close_prompt_palette();
                    self.mark_dirty();
                }
                true
            }
            _ => false,
        }
    }

    /// Insert a prompt reference into the input.
    ///
    /// This replaces the current input with a prompt command reference.
    /// The format is `/prompt:name ` with the cursor positioned at the end.
    ///
    /// # Arguments
    /// * `prompt_name` - Name of the prompt to reference
    pub(super) fn insert_prompt_reference(&mut self, prompt_name: &str) {
        let mut command = String::from(PROMPT_COMMAND_PREFIX);
        command.push_str(prompt_name);
        command.push(' ');

        self.input_manager.set_content(command);
        self.input_manager.move_cursor_to_end();
        self.update_slash_suggestions();
    }
}

// ============================================================================
// Custom Prompts and Input Management
// ============================================================================

impl Session {
    /// Set the custom prompts registry and initialize the prompt palette.
    ///
    /// This updates the session with a new custom prompts registry. If the registry
    /// is enabled and contains prompts, the prompt palette is initialized immediately.
    /// Otherwise, the palette will be lazily initialized when first needed.
    ///
    /// The slash palette is also updated if the user is currently typing a slash command.
    ///
    /// # Arguments
    /// * `custom_prompts` - The custom prompts registry to use
    pub fn set_custom_prompts(&mut self, custom_prompts: CustomPromptRegistry) {
        // Initialize prompt palette when custom prompts are loaded
        if custom_prompts.enabled() && !custom_prompts.is_empty() {
            let mut palette = PromptPalette::new();
            palette.load_prompts(custom_prompts.iter());
            self.palette_state.prompt_palette = Some(palette);
        }

        self.palette_state.custom_prompts = Some(custom_prompts);
        // Update slash palette if we're currently viewing slash commands
        if self.input_manager.content().starts_with('/') {
            self.update_slash_suggestions();
        }
    }

    /// Clear the input field and reset related state.
    ///
    /// This clears the input manager, resets scroll offset, updates slash suggestions,
    /// and marks the session as dirty for redraw.
    ///
    /// This is typically called when the user wants to start fresh or when input
    /// needs to be programmatically cleared.
    pub fn clear_input(&mut self) {
        self.input_manager.clear();
        self.scroll_manager.set_offset(0);
        self.update_slash_suggestions();
        self.mark_dirty();
    }
}
