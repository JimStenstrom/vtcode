#!/bin/bash
# Fix remaining field accesses that were missed by initial sed script

set -e

FILES="vtcode-ui/src/tui/session.rs vtcode-ui/src/tui/session/*.rs"

for file in $FILES; do
    if [ -f "$file" ]; then
        echo "Processing $file..."

        # UIState field accesses (at end of expressions)
        sed -i \
            -e 's/self\.ui\.should_exit$/self.ui.should_exit()/g' \
            -e 's/self\.ui\.input_enabled$/self.ui.is_input_enabled()/g' \
            -e 's/self\.ui\.input_enabled\([^(]\)/self.ui.is_input_enabled()\1/g' \
            -e 's/self\.ui\.input_height\([^(]\)/self.ui.input_height()\1/g' \
            "$file"

        # RenderState field accesses
        sed -i \
            -e 's/self\.render\.modal\([^(_]\)/self.render.modal()\1/g' \
            -e 's/self\.render\.plan\([^(_]\)/self.render.plan()\1/g' \
            -e 's/self\.render\.transcript_cache\([^(_]\)/self.render.transcript_cache()\1/g' \
            "$file"

        # DisplayState field accesses
        sed -i \
            -e 's/self\.display\.lines\([^(_]\)/self.display.lines()\1/g' \
            -e 's/self\.display\.theme\([^(_]\)/self.display.theme()\1/g' \
            -e 's/self\.display\.labels\([^(_]\)/self.display.labels()\1/g' \
            "$file"

        # PaletteState field accesses
        sed -i \
            -e 's/self\.palette\.slash_palette\([^(_]\)/self.palette.slash_palette()\1/g' \
            -e 's/self\.palette\.file_palette_active\([^(_]\)/self.palette.is_file_palette_active()\1/g' \
            -e 's/self\.palette\.prompt_palette_active\([^(_]\)/self.palette.is_prompt_palette_active()\1/g' \
            "$file"

        # PromptState field accesses
        sed -i \
            -e 's/\.prompt\.status_left\([^(_]\)/.prompt.status_left()\1/g' \
            -e 's/\.prompt\.status_right\([^(_]\)/.prompt.status_right()\1/g' \
            "$file"
    fi
done

echo "Done!"
