# EC-004 — XML-entity decoding: escaped sentinel (R&amp;D) IS detected; unknown entity (&copy;)
#          maps to U+FFFD (no false positive/false negative); bounded decoder never panics
# Traces to BC-1.14.004 EC-004
Output docs/demo-evidence/STORY-036/EC-004-xml-entity-decoding.gif
Output docs/demo-evidence/STORY-036/EC-004-xml-entity-decoding.webm

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

Type "cargo nextest run -p slideforge-eval --features test-utils -E 'binary(bleed_tests) & test(ec004)'"
Enter
Sleep 15s
Sleep 2s
