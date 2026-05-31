# AC-003 — SlideTagEngine maps LaidOutSlide to PDF structure tags (Part/H1/Figure/Document).
# Traces to BC-4.03.002 postcondition 3.
# Tests: test_bc_4_03_002_export_produces_tagged_pdf
#         test_bc_4_03_002_tag_slide_produces_one_part_per_slide
#         test_bc_4_03_002_assemble_deck_tag_tree_one_part_per_slide
#         test_bc_4_03_002_figure_alt_text_threaded_from_frame
#         test_bc_4_03_002_decorative_frame_excluded_from_tag_tree
#         test_bc_4_03_002_tag_table_builds_table_tr_th_td
#         test_bc_4_03_002_empty_bullets_produces_no_tag_group
Output docs/demo-evidence/STORY-043/AC-003-tagged-pdf-structure.gif
Output docs/demo-evidence/STORY-043/AC-003-tagged-pdf-structure.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-043"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(tagged_pdf) | test(tag_slide) | test(assemble_deck_tag_tree) | test(figure_alt) | test(decorative_frame) | test(tag_table) | test(empty_bullets)'"
Enter
Sleep 20s
Sleep 2s
