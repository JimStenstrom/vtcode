#!/bin/bash
# Script to fix field access patterns after encapsulating state structs

set -e

FILES="vtcode-ui/src/tui/session.rs vtcode-ui/src/tui/session/*.rs"

for file in $FILES; do
    if [ -f "$file" ]; then
        echo "Processing $file..."

        # DisplayState fixes
        sed -i \
            -e 's/\.display\.theme\([^(_]\)/\.display\.theme()\1/g' \
            -e 's/\.display\.labels\([^(_]\)/\.display\.labels()\1/g' \
            -e 's/\.display\.lines\([^(_]\)/\.display\.lines()\1/g' \
            -e 's/\.display\.in_tool_code_fence\([^(_]\)/\.display\.is_in_tool_code_fence()\1/g' \
            -e 's/\.display\.line_revision_counter/\.display\.current_revision()/g' \
            "$file"

        # PromptState fixes
        sed -i \
            -e 's/\.prompt\.prefix\([^(_]\)/\.prompt\.prefix()\1/g' \
            -e 's/\.prompt\.style\([^(_]\)/\.prompt\.style()\1/g' \
            -e 's/\.prompt\.placeholder\([^(_]\)/\.prompt\.placeholder()\1/g' \
            -e 's/\.prompt\.placeholder_style\([^(_]\)/\.prompt\.placeholder_style()\1/g' \
            -e 's/\.prompt\.status_left\([^(_]\)/\.prompt\.status_left()\1/g' \
            -e 's/\.prompt\.status_right\([^(_]\)/\.prompt\.status_right()\1/g' \
            "$file"

        # UIState boolean flags fixes
        sed -i \
            -e 's/\.ui\.input_enabled\([^(_]\)/\.ui\.is_input_enabled()\1/g' \
            -e 's/\.ui\.cursor_visible\([^(_]\)/\.ui\.is_cursor_visible()\1/g' \
            -e 's/\.ui\.needs_redraw\([^(_]\)/\.ui\.needs_redraw()\1/g' \
            -e 's/\.ui\.needs_full_clear\([^(_]\)/\.ui\.needs_full_clear()\1/g' \
            -e 's/\.ui\.should_exit\([^(_]\)/\.ui\.should_exit()\1/g' \
            "$file"

        # UIState dimension fixes
        sed -i \
            -e 's/\.ui\.view_rows\([^(_]\)/\.ui\.view_rows()\1/g' \
            -e 's/\.ui\.input_height\([^(_]\)/\.ui\.input_height()\1/g' \
            -e 's/\.ui\.transcript_rows\([^(_]\)/\.ui\.transcript_rows()\1/g' \
            -e 's/\.ui\.transcript_width\([^(_]\)/\.ui\.transcript_width()\1/g' \
            -e 's/\.ui\.transcript_view_top\([^(_]\)/\.ui\.transcript_view_top()\1/g' \
            -e 's/\.ui\.show_timeline_pane\([^(_]\)/\.ui\.show_timeline_pane()\1/g' \
            "$file"

        # PaletteState fixes
        sed -i \
            -e 's/\.palette\.slash_palette\([^(_]\)/\.palette\.slash_palette()\1/g' \
            -e 's/\.palette\.file_palette\([^(_]\)/\.palette\.file_palette()\1/g' \
            -e 's/\.palette\.file_palette_active\([^(_]\)/\.palette\.is_file_palette_active()\1/g' \
            -e 's/\.palette\.deferred_file_browser_trigger\([^(_]\)/\.palette\.has_deferred_file_browser_trigger()\1/g' \
            -e 's/\.palette\.prompt_palette\([^(_]\)/\.palette\.prompt_palette()\1/g' \
            -e 's/\.palette\.prompt_palette_active\([^(_]\)/\.palette\.is_prompt_palette_active()\1/g' \
            -e 's/\.palette\.deferred_prompt_browser_trigger\([^(_]\)/\.palette\.has_deferred_prompt_browser_trigger()\1/g' \
            -e 's/\.palette\.custom_prompts\([^(_]\)/\.palette\.custom_prompts()\1/g' \
            "$file"

        # RenderState fixes
        sed -i \
            -e 's/\.render\.header_context\([^(_]\)/\.render\.header_context()\1/g' \
            -e 's/\.render\.header_rows\([^(_]\)/\.render\.header_rows()\1/g' \
            -e 's/\.render\.transcript_cache\([^(_]\)/\.render\.transcript_cache()\1/g' \
            -e 's/\.render\.queued_inputs\([^(_]\)/\.render\.queued_inputs()\1/g' \
            -e 's/\.render\.queue_overlay_cache\([^(_]\)/\.render\.queue_overlay_cache()\1/g' \
            -e 's/\.render\.queue_overlay_version\([^(_]\)/\.render\.queue_overlay_version()\1/g' \
            -e 's/\.render\.modal\([^(_]\)/\.render\.modal()\1/g' \
            -e 's/\.render\.plan\([^(_]\)/\.render\.plan()\1/g' \
            -e 's/\.render\.navigation_state\([^(_]\)/\.render\.navigation_state()\1/g' \
            "$file"
    fi
done

echo "Done! Please review changes and test compilation."
