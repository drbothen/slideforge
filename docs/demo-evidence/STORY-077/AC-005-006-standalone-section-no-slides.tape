# VHS Demo — STORY-077 AC-005 / AC-006
# Standalone section (no slides) -> register content on SECTION node, NOT on LaidOutSlide
#
# AC-005: standalone section methodology: with only detail: -> register on section, not slide
# AC-006: section with both detail: and report: -> two RegisteredContent entries
#
# Modality: cargo nextest (CLI binary STORY-055 not yet built)
# Commit: f7e4d444

Output .factory/demos/STORY-077/AC-005-006-standalone-section-no-slides.gif
Output .factory/demos/STORY-077/AC-005-006-standalone-section-no-slides.webm

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
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-077"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-005/006: standalone section (no slides) -> register content on section node only"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-eval -E 'test(ac006)|test(ac005)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 30s
