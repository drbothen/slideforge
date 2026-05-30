# AC-003 — File-based sources (FileDataSource) always loaded regardless of offline flag
Output docs/demo-evidence/STORY-021/AC-003-file-sources-always-loaded.gif
Output docs/demo-evidence/STORY-021/AC-003-file-sources-always-loaded.webm

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

Type "# AC-003: file sources (supports_offline=false) loaded regardless of --offline flag"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_loads_file_sources) | test(dispatcher_loads_real_file_source)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
