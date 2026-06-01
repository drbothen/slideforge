# AC-003 — Parser stores section TYPE verbatim, no built-in-list rejection at parse time
# Traces to BC-3.02.002 precondition 2, DIR-077-001-A Ruling 3
# Demonstrates: section foobar: parses with 0 errors, 0 warnings, kind.value == "foobar"
Output docs/demo-evidence/STORY-078/AC-003-verbatim-type-no-rejection.gif
Output docs/demo-evidence/STORY-078/AC-003-verbatim-type-no-rejection.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-078"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-syntax -E 'test(test_section_unknown_type_parsed_verbatim)'"
Enter
Sleep 20s
