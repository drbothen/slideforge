# AC-005 — HTTP source declared but unreferenced → no dispatcher error (EC-002)
Output docs/demo-evidence/STORY-021/AC-005-unreferenced-http-no-error.gif
Output docs/demo-evidence/STORY-021/AC-005-unreferenced-http-no-error.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-021"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-005: EC-002 — HTTP source declared, unreferenced; dispatcher emits zero errors"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_unreferenced_http_source)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
