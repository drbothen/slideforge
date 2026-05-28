# AC-006 — 4xx/5xx HTTP status error handling
# Verifies: 404 produces error, 500 produces error, 500 retried succeeds
Output docs/demo-evidence/STORY-019/AC-006-4xx-5xx-status-handling.gif
Output docs/demo-evidence/STORY-019/AC-006-4xx-5xx-status-handling.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-019"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-006: 4xx/5xx HTTP status handling
Type "cargo nextest run -p slideforge-data -E 'test(http_404_error) | test(http_500_error) | test(http_500_retried_succeeds)'"
Enter
Wait+Screen /Summary/
Sleep 2s
