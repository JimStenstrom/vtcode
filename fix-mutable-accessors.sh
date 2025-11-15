#!/bin/bash
# Fix mutable accessor calls after state encapsulation

set -e

FILES="vtcode-ui/src/tui/session.rs vtcode-ui/src/tui/session/*.rs"

for file in $FILES; do
    if [ -f "$file" ]; then
        echo "Processing $file..."

        # Fix navigation_state() -> navigation_state_mut() when calling mutable methods
        sed -i \
            -e 's/self\.render\.navigation_state()\.select(/self.render.navigation_state_mut().select(/g' \
            -e 's/\*self\.render\.navigation_state()\.offset_mut()/*self.render.navigation_state_mut().offset_mut()/g' \
            "$file"

        # Fix file_palette() -> file_palette_mut() in pattern matching
        sed -i \
            -e 's/self\.palette\.file_palette()\.as_mut()/self.palette.file_palette_mut()/g' \
            "$file"

        # Fix prompt_palette() -> prompt_palette_mut() in pattern matching
        sed -i \
            -e 's/self\.palette\.prompt_palette()\.as_mut()/self.palette.prompt_palette_mut()/g' \
            "$file"

        # Fix modal() -> modal_mut() when calling mutable methods
        sed -i \
            -e 's/let Some(modal) = self\.render\.modal()/let Some(modal) = self.render.modal_mut()/g' \
            "$file"

        # Fix lines() -> lines_mut() when calling .last_mut()
        sed -i \
            -e 's/self\.display\.lines()\.last_mut()/self.display.lines_mut().last_mut()/g' \
            "$file"
    fi
done

echo "Done! Verifying compilation..."
