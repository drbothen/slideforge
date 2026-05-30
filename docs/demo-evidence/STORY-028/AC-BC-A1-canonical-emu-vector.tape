Output AC-BC-A1-canonical-emu-vector.gif
Output AC-BC-A1-canonical-emu-vector.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-028"
Enter
Sleep 500ms
Show

Type "# AC-BC-A1: Canonical EMU test vector — layout::run produces BoundingBox{x:457200,y:914400,w:1828800,h:914400}"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(canonical) or test(f_high_002)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
