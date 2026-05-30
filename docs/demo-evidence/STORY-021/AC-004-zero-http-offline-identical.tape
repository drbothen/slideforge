# AC-004 — Zero HTTP sources + ctx.offline=true → identical to offline=false (EC-001)
Output docs/demo-evidence/STORY-021/AC-004-zero-http-offline-identical.gif
Output docs/demo-evidence/STORY-021/AC-004-zero-http-offline-identical.webm

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

Type "# AC-004: EC-001 — offline=true with only file sources, no behavior change"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_zero_http_sources)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
