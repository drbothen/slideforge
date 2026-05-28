# AC-002 — SSRF blocked + remediation hint in error
# Verifies: blocked host returns error with no TCP; error message contains allowlist hint
Output docs/demo-evidence/STORY-019/AC-002-ssrf-blocked-remediation-hint.gif
Output docs/demo-evidence/STORY-019/AC-002-ssrf-blocked-remediation-hint.webm

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

# AC-002: SSRF blocked (no network) + error contains remediation hint
Type "cargo nextest run -p slideforge-data -E 'test(ssrf_blocked_no_network) | test(ssrf_error_contains_remediation_hint)'"
Enter
Wait+Screen /Summary/
Sleep 2s
