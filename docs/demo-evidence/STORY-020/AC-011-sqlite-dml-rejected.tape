# AC-011 — Only SELECT permitted; DML at load() → E-DAT-003 "only SELECT queries allowed" (error path)
Output docs/demo-evidence/STORY-020/AC-011-sqlite-dml-rejected.gif
Output docs/demo-evidence/STORY-020/AC-011-sqlite-dml-rejected.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(sqlite_readonly_rejects_dml) | test(sqlite_readonly_rejects_insert) | test(sqlite_readonly_rejects_update)'"
Enter
Sleep 8s
Sleep 2s
