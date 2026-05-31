# EC-003 — path scoping: sentinel in ppt/notesSlides/ or ppt/theme/ NOT flagged by slide-body check
# Traces to BC-1.14.004 EC-003: BleedChecker checks specifically-scoped ZIP paths
Output docs/demo-evidence/STORY-036/EC-003-path-scoping.gif
Output docs/demo-evidence/STORY-036/EC-003-path-scoping.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-036"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-eval --features test-utils -E 'binary(bleed_tests) & test(ec003)'"
Enter
Sleep 15s
Sleep 2s
