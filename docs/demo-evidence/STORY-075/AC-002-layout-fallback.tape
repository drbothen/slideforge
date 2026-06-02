# AC-002: When slideMaster1.xml has a footer placeholder but no text, BrandLoader
# falls back to slideLayout1.xml and uses its footer text.
# Tests: test_bc_2_01_001_detect_footer_empty_master_fallback_layout
#         test_bc_2_01_001_detect_footer_empty_master_no_layout_yields_none
#         test_bc_2_01_001_detect_footer_both_empty_yields_none
# Traces to BC-2.01.001 postcondition 1 v1.3.
Output docs/demo-evidence/STORY-075/AC-002-layout-fallback.gif
Output docs/demo-evidence/STORY-075/AC-002-layout-fallback.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-075"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(fallback_layout) | test(both_empty_yields_none) | test(no_layout_yields_none)'"
Enter
Sleep 20s
Sleep 2s
