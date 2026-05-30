# AC-010 — query: field is required; empty query defensive check at data layer
Output docs/demo-evidence/STORY-020/AC-010-sqlite-query-required.gif
Output docs/demo-evidence/STORY-020/AC-010-sqlite-query-required.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-020"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-data -E 'test(sqlite_empty_query_defensive) | test(f_pass14_low2_empty_query) | test(f_pass15_low1_whitespace)'"
Enter
Sleep 8s
Sleep 2s
