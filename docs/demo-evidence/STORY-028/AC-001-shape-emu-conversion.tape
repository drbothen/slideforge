Output AC-001-shape-emu-conversion.gif
Output AC-001-shape-emu-conversion.webm

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

Type "# AC-001: Shape EMU conversion — 0.5in→457200, 1.0in→914400, 2.0in→1828800"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(ac001)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
