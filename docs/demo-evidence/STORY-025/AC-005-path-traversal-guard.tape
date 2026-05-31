# AC-005 (error path) — path-traversal containment guard (F-025-001)
# ../escape and symlink-escape in logo_path rejected with LogoOutsideBrandDir (E-BRD-007)
# Also covers empty logo_path guard -> LogoRequired (F-025-A)
Output docs/demo-evidence/STORY-025/AC-005-path-traversal-guard.gif
Output docs/demo-evidence/STORY-025/AC-005-path-traversal-guard.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(test_f025_001) | test(test_f025_a_empty_logo_path_rejected_with_logo_required)'"
Enter
Sleep 30s
Sleep 2s
