# AC-006 — Merged cells in header row → E-DAT-003 "merged cells in header row" (error path)
Output docs/demo-evidence/STORY-020/AC-006-xlsx-merged-header-error.gif
Output docs/demo-evidence/STORY-020/AC-006-xlsx-merged-header-error.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(xlsx_merged_header) | test(xlsx_vertical_merge)'"
Enter
Sleep 8s
Sleep 2s
