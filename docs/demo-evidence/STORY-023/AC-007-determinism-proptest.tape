# AC-007 / VP-012 — Synthesis determinism proptest
Output docs/demo-evidence/STORY-023/AC-007-determinism-proptest.gif
Output docs/demo-evidence/STORY-023/AC-007-determinism-proptest.webm

Require cargo

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

Sleep 2s

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-023"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo test -p slideforge-brand proptest_vp012"
Enter
Sleep 10s
