# AC-007 & AC-008 — Non-PPTX format skip and BC-1.14.003 non-interference
#
# AC-007: deck with section "Name": groupings exported to DOCX contains no sectionLst;
#          SectionListBuilder is PPTX-only — non-PPTX exporters never call it.
#          Full-pipeline DOCX export asserts no "sectionLst" in any DOCX XML part.
# AC-008: detail: and report: register content does NOT bleed into <p:extLst> /
#          <p14:sectionLst>. Two sentinels verified absent from the p:extLst block.
#          Full-pipeline variant (OBS-1 strengthening): parse→eval→layout→PPTX export,
#          ppt/presentation.xml extracted from ZIP, sentinels not found in p:extLst.
#
# Tests:
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_ac007_section_lst_skipped_for_non_pptx
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_med2_docx_has_no_section_lst
#   slideforge-pptx::tests::sections_tests::test_BC_1_14_003_ac008_no_register_content_in_ext_lst
#   slideforge-pptx::tests::sections_tests::test_BC_1_14_003_ac008_no_register_bleed_full_pipeline
#
# Traces to BC-4.01.003 EC-006; BC-1.14.003 postconditions 3 + 5.

Output docs/demo-evidence/STORY-082/AC-007-008-non-pptx-skip-and-non-interference.gif
Output docs/demo-evidence/STORY-082/AC-007-008-non-pptx-skip-and-non-interference.webm

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

Type "# AC-007: sectionLst skipped for DOCX; AC-008: no register bleed into p:extLst (BC-1.14.003)"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-pptx -E 'test(ac007_section_lst_skipped_for_non_pptx) + test(med2_docx_has_no_section_lst) + test(ac008_no_register_content_in_ext_lst) + test(ac008_no_register_bleed_full_pipeline)'"
Enter
Sleep 35s
Sleep 2s
