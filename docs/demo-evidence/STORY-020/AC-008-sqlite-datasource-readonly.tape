# AC-008 — SqliteDataSource implements DataSource; opens with SQLITE_OPEN_READ_ONLY
Output docs/demo-evidence/STORY-020/AC-008-sqlite-datasource-readonly.gif
Output docs/demo-evidence/STORY-020/AC-008-sqlite-datasource-readonly.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(sqlite_happy_path) | test(sqlite_readonly_connection_flag) | test(vp_027)'"
Enter
Sleep 8s
Sleep 2s
