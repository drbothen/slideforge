# AC-002 — Dispatcher contract: skipped HTTP source absent from scope (evaluator produces E-EVL-001)
Output docs/demo-evidence/STORY-021/AC-002-scope-absent-produces-evl001.gif
Output docs/demo-evidence/STORY-021/AC-002-scope-absent-produces-evl001.webm

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

Type "# AC-002: dispatcher absent-scope contract — no scope entry for skipped source"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_does_not_silently_substitute_empty_value) | test(offline_unreferenced_http_source)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
