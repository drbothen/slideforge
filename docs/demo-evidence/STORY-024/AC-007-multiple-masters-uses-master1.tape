# AC-007 — Multiple slide masters: only slideMaster1.xml extracted; lint warning emitted
Output docs/demo-evidence/STORY-024/AC-007-multiple-masters-uses-master1.gif
Output docs/demo-evidence/STORY-024/AC-007-multiple-masters-uses-master1.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-024"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_multiple_slide_masters_uses_master1_only)'"
Enter
Sleep 20s
Sleep 2s
