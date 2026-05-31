# AC-006 — empty brand_overlay block returns Ok(None) (no overlay applied)
# BC-2.02.001 edge case EC-004: empty block -> warning, no overlay applied
Output docs/demo-evidence/STORY-025/AC-006-empty-overlay-noop.gif
Output docs/demo-evidence/STORY-025/AC-006-empty-overlay-noop.webm

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

Type "cargo nextest run -p slideforge-brand -p slideforge-types -E 'test(test_bc_2_02_001_resolve_overlay_empty_returns_none) | test(test_bc_2_02_001_empty_overlay_is_empty)'"
Enter
Sleep 30s
Sleep 2s
