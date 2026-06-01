# AC-002: BrandExtractor emits "# derived via tint/shade; may not match exact color"
# for ColorSlot.is_derived=true, regardless of origin (srgbClr or schemeClr).
# Traces to BC-2.01.003 EC-003 (widened).
# Tests: test_BC_2_01_003_EC003_extractor_derived_srgbclr_emits_comment
#         test_BC_2_01_003_EC003_extractor_unified_flag_drives_comment
Output docs/demo-evidence/STORY-076/AC-002-extractor-derived-comment.gif
Output docs/demo-evidence/STORY-076/AC-002-extractor-derived-comment.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 620
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-076"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(extractor_derived_srgbclr_emits_comment) or test(extractor_unified_flag_drives_comment)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 15s

Sleep 2s
