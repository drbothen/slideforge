# AC-010 — Error path: empty section name rejected with E-PAR-023, exit 1
#
# section "": (empty quoted name) must:
#   1. Produce ParseError::EmptySectionGroupName (E-PAR-023).
#   2. Return Err from parse() — always fatal per error-taxonomy.md §24.
#   3. Produce no SectionGroupNode for the rejected block in the recovered AST.
#
# Tests:
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac010_empty_section_name_rejected_e_par_023
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac010_empty_section_name_no_section_group_node
#
# Traces to BC-4.01.003 postcondition 7 + invariant 5; error-taxonomy.md §24.

Output docs/demo-evidence/STORY-082/AC-010-empty-name-error-path.gif
Output docs/demo-evidence/STORY-082/AC-010-empty-name-error-path.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-082"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-010 (error path): section empty-name → E-PAR-023 fatal, parse() returns Err, no SectionGroupNode"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-syntax -E 'test(ac010_empty_section_name_rejected_e_par_023) + test(ac010_empty_section_name_no_section_group_node)'"
Enter
Sleep 30s
Sleep 2s
