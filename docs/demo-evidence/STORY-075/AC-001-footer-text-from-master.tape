# AC-001: BrandLoader reads slideMaster1.xml for <p:ph type="ftr"/> footer placeholders.
# When found, full concatenated <a:t> text is stored as BrandTemplate.footer_text = Some(...).
# Tests: test_bc_2_01_001_detect_footer_text_from_master
#         test_bc_2_01_001_brand_template_has_footer_flags_field
# Traces to BC-2.01.001 postcondition 1 v1.3 (full footer text if detected).
Output docs/demo-evidence/STORY-075/AC-001-footer-text-from-master.gif
Output docs/demo-evidence/STORY-075/AC-001-footer-text-from-master.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(detect_footer_text_from_master) | test(brand_template_has_footer_flags)'"
Enter
Sleep 20s
Sleep 2s
