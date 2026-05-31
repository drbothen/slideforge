# AC-002 — resolve_overlay loads logo bytes from path
# BC-2.02.001 postcondition 1: overlay slide logo contains override bytes
Output docs/demo-evidence/STORY-025/AC-002-logo-bytes-loaded.gif
Output docs/demo-evidence/STORY-025/AC-002-logo-bytes-loaded.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_02_001_resolve_overlay_logo_path_loads_bytes)'"
Enter
Sleep 30s
Sleep 2s
