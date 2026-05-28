# AC-007 — Network error retried once
# Verifies: transient network error triggers exactly one retry
Output docs/demo-evidence/STORY-019/AC-007-network-retry-once.gif
Output docs/demo-evidence/STORY-019/AC-007-network-retry-once.webm

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

# AC-007: Network error retries once (success on retry)
Type "cargo nextest run -p slideforge-data -E 'test(network_error_retries_once)'"
Enter
Wait+Screen /Summary/
Sleep 2s
