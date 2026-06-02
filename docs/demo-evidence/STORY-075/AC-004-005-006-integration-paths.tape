# AC-004: footer_text + footer_flags populated BEFORE BrandTemplate is returned.
# AC-005: absent slideMaster1.xml -> silent skip, footer_text: None, debug log, no panic.
# AC-006: DOCX input -> footer_text: None, footer_flags: default, no XML reads attempted.
# Tests: test_bc_2_01_001_load_pptx_with_footer
#         test_bc_2_01_001_load_pptx_without_footer
#         test_bc_2_01_001_load_pptx_footer_flags_populated_from_hf_element
#         test_bc_2_01_001_detect_footer_absent_master_xml
#         test_bc_2_01_001_detect_footer_docx_returns_default
#         test_bc_2_01_001_load_docx_footer_always_none
# Traces to BC-2.01.001 postconditions 1+2 and invariant 2.
Output docs/demo-evidence/STORY-075/AC-004-005-006-integration-paths.gif
Output docs/demo-evidence/STORY-075/AC-004-005-006-integration-paths.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-075"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -E 'test(load_pptx_with_footer) | test(load_pptx_without_footer) | test(load_pptx_footer_flags) | test(absent_master_xml) | test(docx_returns_default) | test(load_docx_footer)'"
Enter
Sleep 20s
Sleep 2s
