Output AC-BC-A8-math-xref-boundary.gif
Output AC-BC-A8-math-xref-boundary.webm

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

Type "# AC-BC-A8: Xref inside MathNode NOT validated (Math is a leaf at layout time)"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(vp_046)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
