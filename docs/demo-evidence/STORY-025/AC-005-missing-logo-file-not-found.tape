# AC-005 — missing logo path produces BrandError::FileNotFound (E-BRD-001)
# BC-2.02.001 edge case EC-001: missing logo path -> E-BRD-001; fatal, exit 4
Output docs/demo-evidence/STORY-025/AC-005-missing-logo-file-not-found.gif
Output docs/demo-evidence/STORY-025/AC-005-missing-logo-file-not-found.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_02_001_resolve_overlay_missing_logo_file_not_found)'"
Enter
Sleep 30s
Sleep 2s
