# AC-003 — Exact host[:port] match in allowlist
# Verifies: port mismatch is blocked; subdomain is not matched
Output docs/demo-evidence/STORY-019/AC-003-exact-host-port-match.gif
Output docs/demo-evidence/STORY-019/AC-003-exact-host-port-match.webm

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

# AC-003: Exact host[:port] match — port mismatch blocked, subdomain not matched
Type "cargo nextest run -p slideforge-data -E 'test(port_mismatch_blocked) | test(subdomain_not_matched)'"
Enter
Wait+Screen /Summary/
Sleep 2s
