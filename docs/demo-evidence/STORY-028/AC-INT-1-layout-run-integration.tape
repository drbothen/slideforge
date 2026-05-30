Output AC-INT-1-layout-run-integration.gif
Output AC-INT-1-layout-run-integration.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
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

Type "# AC-INT-1: layout::run end-to-end — shape frames + xref warning in LaidOutDeck"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(ac_int_1) or test(vp_049) or test(vp_050)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
