# AC-010 — All 10 slides with brand_overlay; still single master (data-layer proof)
# BC-2.02.002 edge case EC-002: overlay on every slide, resolve_overlay called N times
# Each call returns independently; the function has no shared state
Output docs/demo-evidence/STORY-025/AC-010-all-slides-resolve-independently.gif
Output docs/demo-evidence/STORY-025/AC-010-all-slides-resolve-independently.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_02_002_all_slides_have_overlays_resolve_independently)'"
Enter
Sleep 30s
Sleep 2s
