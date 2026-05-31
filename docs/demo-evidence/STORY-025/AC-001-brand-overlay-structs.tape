# AC-001 — BrandOverlay + SlideOverlay + LogoOverride struct fields
# BC-2.02.001 precondition 2: overlay declares logo, footer_text, confidentiality
Output docs/demo-evidence/STORY-025/AC-001-brand-overlay-structs.gif
Output docs/demo-evidence/STORY-025/AC-001-brand-overlay-structs.webm

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

Type "cargo nextest run -p slideforge-brand -p slideforge-types -E 'test(test_bc_2_02_001_brand_overlay_fields_present) | test(test_bc_2_02_001_logo_override_fields_present) | test(test_bc_2_02_001_slide_overlay_fields_present) | test(test_bc_2_02_001_brand_overlay_hash_clone)'"
Enter
Sleep 30s
Sleep 2s
