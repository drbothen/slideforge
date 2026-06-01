# AC-001 — "section" keyword no longer E-PAR-006 reserved-to-error
# Traces to BC-3.02.002 preconditions 1 and 3
# Demonstrates: classify_keyword("section") returns None; minimal section block parses with 0 errors
Output docs/demo-evidence/STORY-078/AC-001-section-keyword-unreserved.gif
Output docs/demo-evidence/STORY-078/AC-001-section-keyword-unreserved.webm

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

Type "cargo nextest run -p slideforge-syntax -E 'test(test_section_keyword_no_longer_reserved) | test(test_BC_3_02_002_section_keyword_not_in_reserved_map)'"
Enter
Sleep 20s
