#!/bin/bash
# Fix setter calls after state encapsulation

set -e

FILES="vtcode-ui/src/tui/session.rs vtcode-ui/src/tui/session/*.rs"

for file in $FILES; do
    if [ -f "$file" ]; then
        echo "Processing $file..."

        # UIState setters
        sed -i \
            -e 's/self\.ui\.should_exit() = true/self.ui.request_exit()/g' \
            -e 's/self\.ui\.should_exit() = false/self.ui.set_should_exit(false)/g' \
            -e 's/self\.ui\.needs_redraw() = true/self.ui.request_redraw()/g' \
            -e 's/self\.ui\.needs_redraw() = false/self.ui.clear_redraw_flag()/g' \
            -e 's/self\.ui\.is_cursor_visible() = \([^;]*\);/self.ui.set_cursor_visible(\1);/g' \
            -e 's/self\.ui\.is_input_enabled() = \([^;]*\);/self.ui.set_input_enabled(\1);/g' \
            -e 's/self\.ui\.needs_full_clear() = \([^;]*\);/self.ui.set_needs_full_clear(\1);/g' \
            -e 's/self\.ui\.view_rows() = \([^;]*\);/self.ui.set_view_rows(\1);/g' \
            -e 's/self\.ui\.input_height() = \([^;]*\);/self.ui.set_input_height(\1);/g' \
            -e 's/self\.ui\.transcript_rows() = \([^;]*\);/self.ui.set_transcript_rows(\1);/g' \
            -e 's/self\.ui\.transcript_width() = \([^;]*\);/self.ui.set_transcript_width(\1);/g' \
            -e 's/self\.ui\.transcript_view_top() = \([^;]*\);/self.ui.set_transcript_view_top(\1);/g' \
            -e 's/self\.ui\.show_timeline_pane() = \([^;]*\);/self.ui.set_show_timeline_pane(\1);/g' \
            "$file"

        # PromptState setters
        sed -i \
            -e 's/self\.prompt\.prefix() = \([^;]*\);/self.prompt.set_prefix(\1);/g' \
            -e 's/self\.prompt\.style() = \([^;]*\);/self.prompt.set_style(\1);/g' \
            -e 's/self\.prompt\.placeholder() = \([^;]*\);/self.prompt.set_placeholder(\1);/g' \
            -e 's/self\.prompt\.placeholder_style() = \([^;]*\);/self.prompt.set_placeholder_style(\1);/g' \
            -e 's/self\.prompt\.status_left() = \([^;]*\);/self.prompt.set_status_left(\1);/g' \
            -e 's/self\.prompt\.status_right() = \([^;]*\);/self.prompt.set_status_right(\1);/g' \
            "$file"

        # DisplayState setters
        sed -i \
            -e 's/self\.display\.theme() = \([^;]*\);/self.display.set_theme(\1);/g' \
            -e 's/self\.display\.labels() = \([^;]*\);/self.display.set_labels(\1);/g' \
            -e 's/self\.display\.is_in_tool_code_fence() = \([^;]*\);/self.display.set_in_tool_code_fence(\1);/g' \
            "$file"

        # RenderState setters
        sed -i \
            -e 's/self\.render\.header_context() = \([^;]*\);/self.render.set_header_context(\1);/g' \
            -e 's/self\.render\.header_rows() = \([^;]*\);/self.render.set_header_rows(\1);/g' \
            -e 's/self\.render\.transcript_cache() = \([^;]*\);/self.render.set_transcript_cache(\1);/g' \
            -e 's/self\.render\.modal() = \([^;]*\);/self.render.set_modal(\1);/g' \
            -e 's/self\.render\.plan() = \([^;]*\);/self.render.set_plan(\1);/g' \
            "$file"

        # PaletteState setters
        sed -i \
            -e 's/self\.palette\.file_palette() = \([^;]*\);/self.palette.set_file_palette(\1);/g' \
            -e 's/self\.palette\.is_file_palette_active() = \([^;]*\);/self.palette.set_file_palette_active(\1);/g' \
            -e 's/self\.palette\.has_deferred_file_browser_trigger() = \([^;]*\);/self.palette.set_deferred_file_browser_trigger(\1);/g' \
            -e 's/self\.palette\.prompt_palette() = \([^;]*\);/self.palette.set_prompt_palette(\1);/g' \
            -e 's/self\.palette\.is_prompt_palette_active() = \([^;]*\);/self.palette.set_prompt_palette_active(\1);/g' \
            -e 's/self\.palette\.has_deferred_prompt_browser_trigger() = \([^;]*\);/self.palette.set_deferred_prompt_browser_trigger(\1);/g' \
            -e 's/self\.palette\.custom_prompts() = \([^;]*\);/self.palette.set_custom_prompts(\1);/g' \
            "$file"
    fi
done

echo "Done! Checking compilation..."
