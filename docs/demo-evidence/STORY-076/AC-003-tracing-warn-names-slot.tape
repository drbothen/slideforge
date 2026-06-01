# AC-003: tracing::warn! names the specific slot and each transform type found.
# Example: "slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with is_derived=true"
# Traces to BC-2.01.001 EC-006 observability requirement.
# Tests: test_BC_2_01_001_EC006_srgbclr_warn_names_slot_and_transform
#         test_BC_2_01_001_EC006_srgbclr_multiple_transforms_single_warn
Output docs/demo-evidence/STORY-076/AC-003-tracing-warn-names-slot.gif
Output docs/demo-evidence/STORY-076/AC-003-tracing-warn-names-slot.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(warn_names_slot) or test(multiple_transforms_single_warn)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 15s

Sleep 2s
