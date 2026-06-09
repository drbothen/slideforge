Output docs/demo-evidence/STORY-072/AC-004-docx-solid-fallback-warning.gif
Output docs/demo-evidence/STORY-072/AC-004-docx-solid-fallback-warning.webm

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
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-072"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-004 DOCX: gradient downgraded to solid (from color) + lint warning.
# Warning text: "DOCX gradient fill downgraded to solid (DOCX does not support shape gradient fills)"
# 5 tests: export no error, warning emitted, solid fallback uses from color, EC-003, EC-005
Type "cargo nextest run -p slideforge-docx -E 'test(gradient)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 12s
Sleep 2s
