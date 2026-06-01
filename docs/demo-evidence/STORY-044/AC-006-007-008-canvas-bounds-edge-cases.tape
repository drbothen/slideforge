# AC-006 / AC-007 / AC-008
# No element outside slide canvas + 4:3 non-standard size + zero-height element.
# Traces to BC-4.03.005 invariant 3, edge cases EC-004, EC-005.
# Tests: test_bc_4_03_005_no_element_outside_canvas_after_conversion
#         test_bc_4_03_005_ir_y_to_pdf_y_4x3_slide_mapping
#         test_bc_4_03_005_ir_y_to_pdf_y_zero_height_at_origin_is_slide_height
Output docs/demo-evidence/STORY-044/AC-006-007-008-canvas-bounds-edge-cases.gif
Output docs/demo-evidence/STORY-044/AC-006-007-008-canvas-bounds-edge-cases.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-044"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(no_element_outside_canvas) | test(4x3_slide) | test(zero_height)'"
Enter
Sleep 20s
Sleep 2s
