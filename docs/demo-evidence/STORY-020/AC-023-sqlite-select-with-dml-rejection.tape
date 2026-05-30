# AC-023 — SELECT/WITH prefix allowed; non-SELECT DML → EC-003; non-DML errors pass through
Output docs/demo-evidence/STORY-020/AC-023-sqlite-select-with-dml-rejection.gif
Output docs/demo-evidence/STORY-020/AC-023-sqlite-select-with-dml-rejection.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(vp_029) | test(vp_036) | test(sqlite_comment_prefix) | test(sqlite_column_alias)'"
Enter
Sleep 8s
Sleep 2s
