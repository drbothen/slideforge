# AC-001 & AC-002 — DSL parsing: section "Name": accepted; disambiguation from section IDENT:
#
# AC-001: section "Background": parses into SectionGroupNode with name == "Background".
# AC-002: file with both section methodology: (bare-ident) and section "Background":
#          (quoted-string) produces two distinct AST node types without collision.
#
# Tests:
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac001_section_group_node_parses_with_name
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac002_section_group_distinct_from_section_block
#
# Traces to BC-4.01.003 precondition 5.

Output docs/demo-evidence/STORY-082/AC-001-002-dsl-parsing-happy-path.gif
Output docs/demo-evidence/STORY-082/AC-001-002-dsl-parsing-happy-path.webm

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

Type "# AC-001: section Name: accepted — SectionGroupNode produced with correct name"
Enter
Sleep 1s
Type "# AC-002: disambiguation — quoted-string vs bare-ident produce distinct AST node types"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-syntax -E 'test(ac001_section_group_node_parses_with_name) + test(ac002_section_group_distinct_from_section_block)'"
Enter
Sleep 30s
Sleep 2s
