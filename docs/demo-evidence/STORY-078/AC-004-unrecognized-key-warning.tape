# AC-004 — Unrecognized sub-block KEY produces W-PAR-001 warning AND retained in AST
# Traces to BC-3.02.002 invariant 4, EC-005, DIR-077-001-A Ruling 2
# Demonstrates: foo: in section body → parse succeeds, 1 W-PAR-001 warning, foo retained in fields
Output docs/demo-evidence/STORY-078/AC-004-unrecognized-key-warning.gif
Output docs/demo-evidence/STORY-078/AC-004-unrecognized-key-warning.webm

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

Type "cargo nextest run -p slideforge-syntax -E 'test(test_section_unrecognized_key_warning_at_parse_time) | test(test_BC_3_02_002_unrecognized_key_warning_span_points_to_foo_token)'"
Enter
Sleep 20s
