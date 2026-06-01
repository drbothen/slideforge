Output AC-002-xref-warning-in-bullet.gif
Output AC-002-xref-warning-in-bullet.webm

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

Type "# AC-002: Xref to unknown target inside bullet -> XrefTargetNotFound warning (not error)"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout --test bullets_layout_integration test_bc_3_05_001_story073_ac_int1_unknown_xref_in_bullet_warns --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 15s
Sleep 2s
