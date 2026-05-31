# Fixture parse — canonical three-register-slide.sf parses with no fatal errors;
#                 exactly one slide item produced
# Traces to BC-1.14.004 invariant 2 / STORY-036
Output docs/demo-evidence/STORY-036/FIXTURE-PARSE-three-register-slide.gif
Output docs/demo-evidence/STORY-036/FIXTURE-PARSE-three-register-slide.webm

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

Type "cargo nextest run -p slideforge-eval --features test-utils -E 'test(fixture_parses)'"
Enter
Sleep 15s
Sleep 2s
