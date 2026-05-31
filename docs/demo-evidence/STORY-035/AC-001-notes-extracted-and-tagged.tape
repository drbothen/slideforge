# AC-001 — notes fields extracted and tagged into register_content at eval stage
# Traces to BC-1.14.001 invariant 1: routing determined at Evaluate stage
Output docs/demo-evidence/STORY-035/AC-001-notes-extracted-and-tagged.gif
Output docs/demo-evidence/STORY-035/AC-001-notes-extracted-and-tagged.webm

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

Type "cargo nextest run -p slideforge-eval -E 'test(test_bc_1_14_001_notes_extracted_and_tagged) | test(test_bc_1_14_001_notes_content_captured)'"
Enter
Sleep 15s
Sleep 2s
