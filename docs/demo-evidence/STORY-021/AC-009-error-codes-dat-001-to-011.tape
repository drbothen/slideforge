# AC-009 — DataError enum: E-DAT-001..006..011 Display includes "[E-DAT-NNN]" prefix
Output docs/demo-evidence/STORY-021/AC-009-error-codes-dat-001-to-011.gif
Output docs/demo-evidence/STORY-021/AC-009-error-codes-dat-001-to-011.webm

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

Type "# AC-009: error codes E-DAT-001..011 Display contains [E-DAT-NNN] prefix"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(error_code_dat) | test(dispatcher_routes_xlsx_bad_magic)' --no-fail-fast"
Enter
Sleep 12s
Sleep 2s
