# AC-007 — Snapshot test for a 2-section deck AST (regression baseline for STORY-077)
# Traces to BC-3.02.002 precondition 1, postcondition 6
# Demonstrates: insta snapshot matches 2-section deck (methodology + scope) in source order
Output docs/demo-evidence/STORY-078/AC-007-two-section-snapshot.gif
Output docs/demo-evidence/STORY-078/AC-007-two-section-snapshot.webm

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

Type "cargo nextest run -p slideforge-syntax -E 'test(test_BC_3_02_002_two_section_deck_ast_snapshot)'"
Enter
Sleep 20s
