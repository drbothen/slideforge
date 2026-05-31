# AC-008 — No Master Switch Invariant (BC-2.02.002)
# BrandOverlay and SlideOverlay have NO master-switching field (compile-time proof)
# Exhaustive struct construction tests prove the structural absence at the type level
Output docs/demo-evidence/STORY-025/AC-008-no-master-switch-invariant.gif
Output docs/demo-evidence/STORY-025/AC-008-no-master-switch-invariant.webm

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

Type "cargo nextest run -p slideforge-brand -p slideforge-types -E 'test(test_bc_2_02_002_invariant_brand_overlay_no_master_field) | test(test_bc_2_02_002_invariant_no_second_master_api) | test(test_bc_2_02_002_invariant_no_master_path_field) | test(test_bc_2_02_002_slide_overlay_type_has_no_master_path)'"
Enter
Sleep 30s
Sleep 2s
