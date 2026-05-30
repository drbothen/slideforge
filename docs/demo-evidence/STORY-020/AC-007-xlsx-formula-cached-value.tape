# AC-007 — Formula cells use cached/computed value; formula text never exposed
Output docs/demo-evidence/STORY-020/AC-007-xlsx-formula-cached-value.gif
Output docs/demo-evidence/STORY-020/AC-007-xlsx-formula-cached-value.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-020"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-data -E 'test(xlsx_formula_uses_cached)'"
Enter
Sleep 8s
Sleep 2s
