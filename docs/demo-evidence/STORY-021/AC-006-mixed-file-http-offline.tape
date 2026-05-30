# AC-006 — Mixed file + HTTP sources, offline=true → file loaded, HTTP skipped (EC-003)
Output docs/demo-evidence/STORY-021/AC-006-mixed-file-http-offline.gif
Output docs/demo-evidence/STORY-021/AC-006-mixed-file-http-offline.webm

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

Type "# AC-006: EC-003 — mixed file+HTTP offline; file scope present, HTTP absent; partial-load"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_with_file_and_http) | test(partial_load_continues_on_error)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
