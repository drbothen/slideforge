# AC-019 — Extension check then ZIP magic-byte check; wrong magic → E-DAT-003 EC-013 (error path)
Output docs/demo-evidence/STORY-020/AC-019-xlsx-extension-magic-bytes.gif
Output docs/demo-evidence/STORY-020/AC-019-xlsx-extension-magic-bytes.webm

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

Type "cargo nextest run -p slideforge-data -E 'test(vp_026_wrong_magic) | test(unsupported_extension_rejected)'"
Enter
Sleep 8s
Sleep 2s
