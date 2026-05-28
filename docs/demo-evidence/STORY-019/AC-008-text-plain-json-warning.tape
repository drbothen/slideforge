# AC-008 — text/plain JSON parse + warning emitted
# Verifies: text/plain body parsed as JSON with a diagnostic warning
Output docs/demo-evidence/STORY-019/AC-008-text-plain-json-warning.gif
Output docs/demo-evidence/STORY-019/AC-008-text-plain-json-warning.webm

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

# AC-008: text/plain body parsed as JSON — warning emitted
Type "cargo nextest run -p slideforge-data -E 'test(text_plain_warning_emitted)'"
Enter
Wait+Screen /Summary/
Sleep 2s
