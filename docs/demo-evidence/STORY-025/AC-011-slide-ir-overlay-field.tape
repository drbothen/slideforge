# AC-011 — Slide IR gains overlay: Option<SlideOverlay> field
# BC-2.02.001 invariant 2: overlay stored in Deck IR as metadata on slide node
# Slide::overlay field is Hash+Clone; None is the default
Output docs/demo-evidence/STORY-025/AC-011-slide-ir-overlay-field.gif
Output docs/demo-evidence/STORY-025/AC-011-slide-ir-overlay-field.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-025"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-types -E 'test(test_bc_2_02_001_slide_overlay_none_is_default) | test(test_bc_2_02_002_slide_overlay_type_has_no_master_path) | test(test_bc_2_02_001_slide_has_overlay_field_hash_clone)'"
Enter
Sleep 30s
Sleep 2s
