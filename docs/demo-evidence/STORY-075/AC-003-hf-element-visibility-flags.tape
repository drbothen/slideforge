# AC-003: BrandLoader reads <p:hf> element from slideMaster1.xml for footer visibility
# flags (ftr, dt, sldNum). NOT from presProps.xml (adversary H-1 correction, BC-2.01.001 v1.3).
# Tests: test_bc_2_01_001_detect_footer_flags_from_hf_element
#         test_bc_2_01_001_detect_footer_flags_all_three_from_hf_element
#         test_bc_2_01_001_v1_3_canonical_vector_ftr1_dt1_sldnum0
#         test_bc_2_01_001_detect_footer_flags_hf_boolean_string_form
#         test_bc_2_01_001_detect_footer_flags_hf_partial_attrs
#         test_bc_2_01_001_detect_footer_flags_absent_hf_element
# Traces to BC-2.01.001 postcondition 1 v1.3 corrected (H-1).
Output docs/demo-evidence/STORY-075/AC-003-hf-element-visibility-flags.gif
Output docs/demo-evidence/STORY-075/AC-003-hf-element-visibility-flags.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(footer_flags)'"
Enter
Sleep 20s
Sleep 2s
