# AC-010 — file:// scheme rejected with error code
# Verifies: file:// and FILE:// URLs both rejected; error carries ForbiddenScheme code
Output docs/demo-evidence/STORY-019/AC-010-file-scheme-rejected.gif
Output docs/demo-evidence/STORY-019/AC-010-file-scheme-rejected.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-019"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-010: file:// scheme rejected (both lowercase and uppercase)
Type "cargo nextest run -p slideforge-data -E 'test(file_scheme_rejected_with_code) | test(uppercase_file_scheme_rejected)'"
Enter
Wait+Screen /Summary/
Sleep 2s
