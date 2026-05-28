# AC-009 — HTTP warning emitted for plain HTTP (non-HTTPS)
# Verifies: http:// URL triggers a diagnostic warning via tracing
Output docs/demo-evidence/STORY-019/AC-009-http-warning-emitted.gif
Output docs/demo-evidence/STORY-019/AC-009-http-warning-emitted.webm

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

# AC-009: HTTP (non-HTTPS) warning emitted
Type "cargo nextest run -p slideforge-data -E 'test(http_warning_emitted)'"
Enter
Wait+Screen /Summary/
Sleep 2s
