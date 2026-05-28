# AC-012 — supports_offline() returns true
# Verifies: HttpDataSource reports supports_offline = true (no network required flag)
Output docs/demo-evidence/STORY-019/AC-012-supports-offline.gif
Output docs/demo-evidence/STORY-019/AC-012-supports-offline.webm

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

# AC-012: supports_offline() returns true
Type "cargo nextest run -p slideforge-data -E 'test(supports_offline_returns_true)'"
Enter
Wait+Screen /Summary/
Sleep 2s
