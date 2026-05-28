# AC-001 — HttpDataSource struct + DataSource impl
# Verifies: source_id() returns "http", fetch() succeeds on JSON endpoint
Output docs/demo-evidence/STORY-019/AC-001-http-datasource-struct.gif
Output docs/demo-evidence/STORY-019/AC-001-http-datasource-struct.webm

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

# AC-001: HttpDataSource struct + DataSource impl
Type "cargo nextest run -p slideforge-data -E 'test(http_source_id) | test(http_json_happy_path)'"
Enter
Wait+Screen /Summary/
Sleep 2s
