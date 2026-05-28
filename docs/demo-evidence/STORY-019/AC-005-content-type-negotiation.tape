# AC-005 — Content-Type negotiation (JSON, CSV, YAML, aliases)
# Verifies: correct parser dispatched per Content-Type header
Output docs/demo-evidence/STORY-019/AC-005-content-type-negotiation.gif
Output docs/demo-evidence/STORY-019/AC-005-content-type-negotiation.webm

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

# AC-005: Content-Type negotiation — JSON, CSV, YAML, aliases
Type "cargo nextest run -p slideforge-data -E 'test(http_json_happy_path) | test(http_csv_happy_path) | test(http_yaml_happy_path) | test(content_type_aliases)'"
Enter
Wait+Screen /Summary/
Sleep 2s
