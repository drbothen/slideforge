# AC-005 — detail fields extracted and tagged into register_content at eval stage (slide-level)
# Traces to BC-1.14.003 invariant 1: detail routing determined at Evaluate stage
# Note: section-level detail routing is out of scope (STORY-077)
Output docs/demo-evidence/STORY-035/AC-005-detail-extracted-and-tagged.gif
Output docs/demo-evidence/STORY-035/AC-005-detail-extracted-and-tagged.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-035"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-eval -E 'test(test_bc_1_14_003_detail_extracted_and_tagged) | test(test_bc_1_14_003_detail_content_captured_correctly)'"
Enter
Sleep 15s
Sleep 2s
