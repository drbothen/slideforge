# AC-013 — Zero TCP connections for SSRF-blocked hosts
# Verifies: AtomicUsize counter stays at 0 when host is blocked
Output docs/demo-evidence/STORY-019/AC-013-zero-tcp-blocked.gif
Output docs/demo-evidence/STORY-019/AC-013-zero-tcp-blocked.webm

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

# AC-013: Zero TCP connections when SSRF-blocked (AtomicUsize counter = 0)
Type "cargo nextest run -p slideforge-data -E 'test(ssrf_blocked_no_network)'"
Enter
Wait+Screen /Summary/
Sleep 2s
