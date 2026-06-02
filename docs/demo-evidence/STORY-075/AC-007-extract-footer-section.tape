# AC-007: BrandTemplate.footer_text flows through BrandExtractor::extract() to produce
# a [footer] section in brand.toml. Closes adversary finding F-024A-OBS-1.
# Test: test_bc_2_01_001_extract_brand_toml_includes_footer_section
# End-to-end path: load PPTX fixture with footer placeholder -> extract -> brand.toml
# contains [footer] with text = "Acme Corp Confidential".
# Traces to BC-2.01.001 postcondition 1 cross-story (activates STORY-024 [footer] writer).
Output docs/demo-evidence/STORY-075/AC-007-extract-footer-section.gif
Output docs/demo-evidence/STORY-075/AC-007-extract-footer-section.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(extract_brand_toml_includes_footer_section)'"
Enter
Sleep 20s
Sleep 2s
