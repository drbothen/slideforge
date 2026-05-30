# AC-007 — watch --offline: dispatcher respects offline on every re-eval cycle (EC-004)
Output docs/demo-evidence/STORY-021/AC-007-watch-mode-repeated-dispatch.gif
Output docs/demo-evidence/STORY-021/AC-007-watch-mode-repeated-dispatch.webm

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

Type "# AC-007: EC-004 — watch mode; offline gate applies on every re-eval, HTTP never polled"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_watch_mode_repeated_dispatch)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
