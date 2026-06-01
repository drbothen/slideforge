# AC-004: srgbClr WITHOUT transforms: is_derived=false, no warn!, no TOML comment.
# STORY-022 regression guard — existing behavior unaffected.
# Also covers AC-005: clippy clean, forbid(unsafe_code) (shown via clippy pass).
# Tests: test_BC_2_01_001_EC006_srgbclr_without_transforms_not_derived
#         test_BC_2_01_001_EC006_srgbclr_clean_val_regression
#         test_BC_2_01_001_EC006_derived_flag_not_set_on_sysclr
#         test_BC_2_01_003_EC003_extractor_clean_srgbclr_no_comment
Output docs/demo-evidence/STORY-076/AC-004-clean-srgbclr-regression.gif
Output docs/demo-evidence/STORY-076/AC-004-clean-srgbclr-regression.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 700
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

Type "cargo nextest run -p slideforge-brand -E 'test(without_transforms_not_derived) or test(clean_val_regression) or test(derived_flag_not_set_on_sysclr) or test(extractor_clean_srgbclr_no_comment)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 15s

Type "cargo clippy -p slideforge-brand -- -D warnings 2>&1 | tail -5"
Enter
Sleep 15s

Sleep 2s
