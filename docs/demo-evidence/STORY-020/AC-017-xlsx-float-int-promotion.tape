# AC-017 — Whole-number Float→Int promotion; NaN/Infinity → ParseError (success + error paths)
Output docs/demo-evidence/STORY-020/AC-017-xlsx-float-int-promotion.gif
Output docs/demo-evidence/STORY-020/AC-017-xlsx-float-int-promotion.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(vp_021) | test(vp_022) | test(vp_023)'"
Enter
Sleep 8s
Sleep 2s
