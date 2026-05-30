Output AC-BC-A5-missing-alt-span.gif
Output AC-BC-A5-missing-alt-span.webm

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

Type "# AC-BC-A5: LayoutError::MissingAlt carries SourceSpan (file:line:col)"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(vp_038) or test(missing_alt_carries)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
