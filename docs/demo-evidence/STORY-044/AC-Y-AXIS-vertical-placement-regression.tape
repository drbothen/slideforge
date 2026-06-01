# Y-Axis Placement — Title at TOP, Footer at BOTTOM (DIR-044-001 mirror-bug regression)
# Proves krilla top-left Y-down draw path (emu_to_pt(ir_y), NOT ir_y_to_pdf_y at draw time).
# Traces to BC-4.03.005 + DIR-044-001.
# Tests: test_vertical_placement_title_at_top
#         test_vertical_placement_footer_at_bottom
#         test_bc_4_03_005_ac006_export_all_elements_within_canvas
Output docs/demo-evidence/STORY-044/AC-Y-AXIS-vertical-placement-regression.gif
Output docs/demo-evidence/STORY-044/AC-Y-AXIS-vertical-placement-regression.webm

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

Type "cargo nextest run -p slideforge-pdf -E 'test(vertical_placement) | test(ac006_export)'"
Enter
Sleep 20s
Sleep 2s
