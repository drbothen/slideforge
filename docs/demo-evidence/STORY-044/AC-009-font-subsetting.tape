# AC-009 — Font Subsetting via krilla internal (no system tooling)
# export() draws text via krilla Surface; krilla subsets font internally.
# For a 2-glyph draw ("Hi") from LM Math (4802 glyphs, 717 KiB), embedded font < 100 KB.
# Traces to BC-4.03.002 invariant 3.
# Tests: test_bc_4_03_002_ac009_font_subset_smaller_than_full_font
#         test_bc_4_03_002_ac009_lm_math_fixture_exists_and_is_large
#         test_bc_4_03_002_no_direct_subsetter_call
Output docs/demo-evidence/STORY-044/AC-009-font-subsetting.gif
Output docs/demo-evidence/STORY-044/AC-009-font-subsetting.webm

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

Type "cargo nextest run -p slideforge-pdf --test ac009_font_subsetting --no-capture 2>&1"
Enter
Sleep 30s
Sleep 2s
