# AC-001: srgbClr with lumMod/tint/shade sets is_derived=true; base hex stored verbatim.
# Traces to BC-2.01.001 EC-006 (loading side).
# Tests: test_BC_2_01_001_EC006_srgbclr_lummod_sets_derived_flag
#         test_BC_2_01_001_EC006_srgbclr_tint_sets_derived_flag
#         test_BC_2_01_001_EC006_srgbclr_shade_sets_derived_flag
#         test_BC_2_01_001_EC006_srgbclr_lumoff_sets_derived_flag
#         test_BC_2_01_001_EC006_srgbclr_lummod_preserves_base_hex
#         test_BC_2_01_001_EC006_srgbclr_no_hsl_resolution_performed
Output docs/demo-evidence/STORY-076/AC-001-srgbclr-derived-flag.gif
Output docs/demo-evidence/STORY-076/AC-001-srgbclr-derived-flag.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(srgbclr_lummod) or test(srgbclr_tint) or test(srgbclr_shade) or test(srgbclr_lumoff) or test(preserves_base_hex) or test(no_hsl_resolution)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 15s

Sleep 2s
