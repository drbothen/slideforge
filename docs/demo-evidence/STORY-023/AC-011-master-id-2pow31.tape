# AC-011 — Master ID = 2^31; layout_id_start = 2^31+1; slide_id_start = 256
Output docs/demo-evidence/STORY-023/AC-011-master-id-2pow31.gif
Output docs/demo-evidence/STORY-023/AC-011-master-id-2pow31.webm

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

Type "cargo test -p slideforge-brand ac011"
Enter
Sleep 8s
