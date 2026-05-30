Output AC-005-12-inline-variants.gif
Output AC-005-12-inline-variants.webm

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

Type "# AC-005: All 12 InlineNode variants (Plain/Bold/Italic/Code/Link/Math/Footnote/Xref/Superscript/Subscript/Strikethrough/Highlight)"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-layout -E 'test(ac005)' --no-fail-fast 2>&1 | grep -E '(PASS|FAIL|Summary)'"
Enter
Sleep 8s
Sleep 2s
