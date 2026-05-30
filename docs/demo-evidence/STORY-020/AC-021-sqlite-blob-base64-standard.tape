# AC-021 — BLOB columns encoded with STANDARD RFC 4648 §4 base64; [0x00,0xFF,0x42] → "AP9C"
Output docs/demo-evidence/STORY-020/AC-021-sqlite-blob-base64-standard.gif
Output docs/demo-evidence/STORY-020/AC-021-sqlite-blob-base64-standard.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(vp_033_blob_canonical_base64) | test(sqlite_blob_base64)'"
Enter
Sleep 8s
Sleep 2s
