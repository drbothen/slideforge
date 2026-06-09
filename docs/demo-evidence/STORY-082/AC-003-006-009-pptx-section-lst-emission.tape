# AC-003, AC-004, AC-005, AC-006, AC-009 — PPTX sectionLst emission (happy path)
#
# AC-003: two sections → <p:extLst>/<p14:sectionLst> in presentation.xml with correct
#          uri, xmlns:p14 on <p:presentation>, p:extLst as last child.
# AC-004: no sections → no <p:extLst> emitted; bytes returned unchanged.
# AC-005: section name with & → XML-escaped as &amp; in p14:section/@name.
# AC-006: two single-slide sections → two p14:section, one p14:sldId each.
# AC-009: deterministic GUIDs — identical builds produce byte-identical output;
#          GUID format: brace-wrapped uppercase hex, version nibble=5, RFC 4122 variant.
#
# Tests:
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac003_presentation_xml_has_p14_section_lst
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac003_extlst_is_last_child_of_presentation
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac003_ext_uri_is_correct_fixed_constant
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac003_xmlns_p14_declared_on_presentation
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac004_no_ext_lst_when_no_sections
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_inject_no_op_empty_sections
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac005_xml_escaped_section_name
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac006_two_single_slide_sections
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac009_deterministic_guids_identical_across_builds
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac009_guid_format_braced_uppercase
#
# Traces to BC-4.01.003 postcondition 5, EC-002, EC-004, EC-005, invariant 3.

Output docs/demo-evidence/STORY-082/AC-003-006-009-pptx-section-lst-emission.gif
Output docs/demo-evidence/STORY-082/AC-003-006-009-pptx-section-lst-emission.webm

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

Type "# AC-003..006,009: PPTX sectionLst emission — happy path (SectionListBuilder unit tests)"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-pptx -E 'test(ac003) + test(ac004) + test(ac005) + test(ac006) + test(ac009)'"
Enter
Sleep 35s
Sleep 2s
