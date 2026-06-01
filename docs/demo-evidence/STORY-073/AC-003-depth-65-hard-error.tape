Output AC-003-depth-65-hard-error.gif
Output AC-003-depth-65-hard-error.webm

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

Type "# AC-003: Bullet item with 65-deep inline tree -> LayoutError::InlineDepthExceeded (hard error)"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout --test bullets_layout_integration test_bc_3_05_001_story073_ac_int1_depth_65_bullet_is_hard_error --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 15s
Sleep 2s
