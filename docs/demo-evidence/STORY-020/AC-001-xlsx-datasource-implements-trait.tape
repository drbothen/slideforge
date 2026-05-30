# AC-001 — XlsxDataSource implements DataSource trait; load() returns Value::List
Output docs/demo-evidence/STORY-020/AC-001-xlsx-datasource-implements-trait.gif
Output docs/demo-evidence/STORY-020/AC-001-xlsx-datasource-implements-trait.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(xlsx_happy_path) | test(xlsx_source_id) | test(xlsx_struct_fields)'"
Enter
Sleep 8s
Sleep 2s
