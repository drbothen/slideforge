# AC-002 — First row is header; subsequent rows are Value::Map; empty sheet → E-DAT-003
Output docs/demo-evidence/STORY-020/AC-002-xlsx-first-row-header.gif
Output docs/demo-evidence/STORY-020/AC-002-xlsx-first-row-header.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(xlsx_empty_header_row) | test(xlsx_three_data_rows) | test(xlsx_first_sheet_default)'"
Enter
Sleep 8s
Sleep 2s
