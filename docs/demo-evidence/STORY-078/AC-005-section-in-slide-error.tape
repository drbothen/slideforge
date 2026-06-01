# AC-005 — section <type>: block inside a slide block is a parse error (E-PAR-018)
# Traces to BC-3.02.002 precondition 3, EC-002
# Demonstrates: section nested in slide → fatal error; also covers @for and @if nesting (MED-1)
Output docs/demo-evidence/STORY-078/AC-005-section-in-slide-error.gif
Output docs/demo-evidence/STORY-078/AC-005-section-in-slide-error.webm

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

Type "cargo nextest run -p slideforge-syntax -E 'test(test_section_nested_in_slide_error) | test(test_section_nested_in_for_error) | test(test_section_nested_in_if_error)'"
Enter
Sleep 20s
