# STORY-045 AC-008 + AC-009: veraPDF proxy validation + searchable text
# AC-008: UA-1 composite proxy (StructTreeRoot + MarkInfo + /Lang + /Figure/Alt)
# AC-009: Text searchable via ToUnicode CMap font subsetting (ac009_font_subsetting tests)
#
# Note: veraPDF is NOT installed locally. The definitive AC-008 gate runs in CI
# via .github/workflows/pdf-ua1.yml with veraPDF installed from the official
# verapdf-greenfield installer (Java-based). The AC-013 test is #[ignore]
# locally and is un-ignored by CI using --run-ignored.

Output docs/demo-evidence/STORY-045/AC-008-009-ua1-proxy-searchable.gif
Output docs/demo-evidence/STORY-045/AC-008-009-ua1-proxy-searchable.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-045"
Enter
Wait+Line /\$/
Type "clear"
Enter
Wait+Line /\$/
Show

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(ua1_export_proxy)' --no-fail-fast && cargo nextest run -p slideforge-pdf --test ac009_font_subsetting --no-fail-fast"
Enter
Wait+Line /Summary/
Sleep 2s
