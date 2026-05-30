# AC-020 — TEXT columns decoded via strict from_utf8; invalid UTF-8 → E-DAT-003 EC-008 (error path)
Output docs/demo-evidence/STORY-020/AC-020-sqlite-strict-utf8.gif
Output docs/demo-evidence/STORY-020/AC-020-sqlite-strict-utf8.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(vp_032_invalid_utf8) | test(sqlite_text_value)'"
Enter
Sleep 8s
Sleep 2s
