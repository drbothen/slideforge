# AC-003 (end-to-end) — Full pipeline: parse → eval → layout → PPTX export;
# ppt/presentation.xml extracted from ZIP contains <p14:sectionLst>.
#
# The keystone e2e test (HIGH-3) proves the complete 5-stage pipeline is wired:
#   parse() populates SectionGroupNode → eval_deck() produces slide_sections on Deck →
#   layout::run() passes slide_sections to LaidOutDeck → PptxExporter calls
#   SectionListBuilder::inject() → presentation.xml in the PPTX ZIP contains
#   <p:extLst>/<p14:sectionLst> with the correct uri, xmlns:p14, and section children.
#
# CRIT-A test proves @for expansion before a section group correctly advances the
# flat slide-index counter, so section slide IDs are correct (not offset from 256).
#
# Tests:
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_high3_e2e_pipeline_section_lst_in_presentation_xml
#   slideforge-pptx::tests::sections_tests::test_BC_4_01_003_crit_a_for_before_section_correct_slide_ids
#
# Traces to BC-4.01.003 (all ACs), CRIT-1/2/3/4.

Output docs/demo-evidence/STORY-082/AC-003-e2e-pipeline-section-lst-in-xml.gif
Output docs/demo-evidence/STORY-082/AC-003-e2e-pipeline-section-lst-in-xml.webm

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

Type "# AC-003 end-to-end: parse→eval→layout→PPTX; presentation.xml contains <p14:sectionLst>"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-pptx -E 'test(high3_e2e_pipeline_section_lst_in_presentation_xml) + test(crit_a_for_before_section_correct_slide_ids)'"
Enter
Sleep 35s
Sleep 2s
