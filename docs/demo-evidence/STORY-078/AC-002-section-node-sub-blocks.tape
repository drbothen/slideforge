# AC-002 — section <type>: with recognized type and indented sub-blocks parses to SectionNode
# Traces to BC-3.02.002 precondition 2, postconditions 5 and 8
# Demonstrates: section methodology: produces SectionNode with kind="methodology", 2 fields, FieldValue::Template
Output docs/demo-evidence/STORY-078/AC-002-section-node-sub-blocks.gif
Output docs/demo-evidence/STORY-078/AC-002-section-node-sub-blocks.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-078"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-syntax -E 'test(test_section_recognized_type_with_sub_blocks)'"
Enter
Sleep 20s
