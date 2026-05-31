# AC-004 — Font loading helpers: load_font_data, system_font_fallback, FontBytes.
# Traces to BC-4.03.002 invariant 3.
# Tests: test_bc_4_03_002_load_font_data_reads_file
#         test_bc_4_03_002_load_font_data_errors_on_missing_path
#         test_bc_4_03_002_system_font_fallback_none_for_absent_family
#         test_bc_4_03_002_font_bytes_load_reads_file
#         test_font_bytes_from_bytes_round_trips
Output docs/demo-evidence/STORY-043/AC-004-font-loading.gif
Output docs/demo-evidence/STORY-043/AC-004-font-loading.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-043"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'binary(slideforge_pdf) & test(font)'"
Enter
Sleep 20s
Sleep 2s
