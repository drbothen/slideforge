Output AC-INT-1-end-to-end-bullets-layout.gif
Output AC-INT-1-end-to-end-bullets-layout.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set FontFamily "FiraCode Nerd Font Mono"
Set Padding 20
Set Margin 10
Set MarginFill "#282a36"

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-073"
Enter
Sleep 500ms
Show

Type "# AC-INT-1: layout::run end-to-end with ContentBlock::Bullets -- all 13 integration tests pass"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout --test bullets_layout_integration --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 15s
Sleep 2s
