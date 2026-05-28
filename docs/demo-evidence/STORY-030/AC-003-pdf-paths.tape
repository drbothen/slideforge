Output AC-003-pdf-paths.gif
Output AC-003-pdf-paths.webm

Require cargo
Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-030"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-math -E 'test(test_bc_1_10_003_pdf_paths_svg_has_no_text_elements) + test(test_bc_1_10_003_pdf_paths_svg_has_no_image_elements) + test(test_bc_1_10_003_pdf_paths_svg_contains_path_elements) + test(test_bc_1_10_003_pdf_paths_wellformed_xml) + test(test_bc_1_10_003_svg_paths_struct_fields_accessible)' --success-output=immediate-final"
Enter
Sleep 30s
Sleep 2s
