# AC-011 — Warning path: duplicate section names emit W-PAR-002, exit 0, same GUID
#
# Two section "Background": blocks must:
#   1. Produce ParseWarning::DuplicateSectionGroupName (W-PAR-002).
#   2. Parse succeeds — exit 0 (cosmetic warning, not fatal).
#   3. Both SectionGroupNode entries appear in the AST.
#   4. Both p14:section/@id attributes are identical (same name → same SHA-256 → same GUID).
#
# Tests:
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac011_duplicate_section_name_warning_w_par_002
#   slideforge-syntax::parser::section_group_tests::test_BC_4_01_003_ac011_duplicate_section_name_both_sections_present
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac011_duplicate_section_name_same_guid
#
# Traces to BC-4.01.003 postcondition 8 + invariant 5.

Output docs/demo-evidence/STORY-082/AC-011-duplicate-name-warning-path.gif
Output docs/demo-evidence/STORY-082/AC-011-duplicate-name-warning-path.webm

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

Type "# AC-011 (warning path): duplicate section names → W-PAR-002, exit 0, same GUID"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-syntax -p slideforge-pptx -E 'test(ac011)'"
Enter
Sleep 35s
Sleep 2s
