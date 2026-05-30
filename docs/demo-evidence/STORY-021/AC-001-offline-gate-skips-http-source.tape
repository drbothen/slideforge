# AC-001 — DataSourceDispatcher skips HTTP sources when ctx.offline=true (no scope entry)
Output docs/demo-evidence/STORY-021/AC-001-offline-gate-skips-http-source.gif
Output docs/demo-evidence/STORY-021/AC-001-offline-gate-skips-http-source.webm

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

Type "# AC-001: offline gate — HTTP source absent from scope when ctx.offline=true"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_skips_http_source) | test(online_loads_all_sources)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
