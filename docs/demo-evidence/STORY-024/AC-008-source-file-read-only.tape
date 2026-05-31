# AC-008 — Source .pptx is NEVER modified; SHA-256 before and after extraction identical
Output docs/demo-evidence/STORY-024/AC-008-source-file-read-only.gif
Output docs/demo-evidence/STORY-024/AC-008-source-file-read-only.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-024"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(test_bc_2_01_003_invariant_source_file_unmodified)'"
Enter
Sleep 20s
Sleep 2s
