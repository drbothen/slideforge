# AC-022 — Closed extension list (.db/.sqlite/.sqlite3) + SQLite magic-byte check (error paths)
Output docs/demo-evidence/STORY-020/AC-022-sqlite-extension-magic-bytes.gif
Output docs/demo-evidence/STORY-020/AC-022-sqlite-extension-magic-bytes.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-020"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-data -E 'test(vp_034_wrong_sqlite_magic) | test(vp_035_unsupported_extension) | test(obs3_sqlite_no_extension)'"
Enter
Sleep 8s
Sleep 2s
