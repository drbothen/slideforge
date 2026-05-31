# AC-004 — confidentiality label text passes through to BrandOverlay
# BC-2.02.001 postcondition 3: confidentiality label added/updated
Output docs/demo-evidence/STORY-025/AC-004-confidentiality-passthrough.gif
Output docs/demo-evidence/STORY-025/AC-004-confidentiality-passthrough.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_02_001_resolve_overlay_confidentiality_passthrough)'"
Enter
Sleep 30s
Sleep 2s
