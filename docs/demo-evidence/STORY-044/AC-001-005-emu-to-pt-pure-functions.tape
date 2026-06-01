# AC-001 / AC-002 / AC-003 / AC-004 / AC-005
# emu_to_pt canonical test vectors + ir_y_to_pdf_y + rounding precision + purity.
# Traces to BC-4.03.005 postconditions 1-5 and invariants 1-2.
# Tests: test_bc_4_03_005_emu_to_pt_slide_width_720
#         test_bc_4_03_005_emu_to_pt_slide_height_405
#         test_bc_4_03_005_emu_to_pt_one_point
#         test_bc_4_03_005_emu_to_pt_zero_is_zero
#         test_bc_4_03_005_ir_y_to_pdf_y_top_left_element_305
#         test_bc_4_03_005_ir_y_to_pdf_y_bottom_edge_zero
#         test_bc_4_03_005_ir_y_to_pdf_y_zero_height_at_origin_is_slide_height
#         test_bc_4_03_005_emu_to_pt_rounding_error_below_0_001
#         test_bc_4_03_005_emu_to_pt_is_deterministic
#         test_bc_4_03_005_ir_y_to_pdf_y_is_deterministic
Output docs/demo-evidence/STORY-044/AC-001-005-emu-to-pt-pure-functions.gif
Output docs/demo-evidence/STORY-044/AC-001-005-emu-to-pt-pure-functions.webm

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

Type "cargo nextest run -p slideforge-pdf -E 'test(emu_to_pt) | test(ir_y_to_pdf_y) | test(rounding_error)'"
Enter
Sleep 20s
Sleep 2s
