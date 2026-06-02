# STORY-045 AC-003: Text elements tagged with correct semantic structure types
# H1/H2/P/L/LI structure elements appear for title, subtitle, body, bullet frames
#
# Evidence: run the AC-003 test from tests/pdf_ua1.rs

Output docs/demo-evidence/STORY-045/AC-003-text-tagged-types.gif
Output docs/demo-evidence/STORY-045/AC-003-text-tagged-types.webm

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

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(text_elements_tagged_correct_types)' --no-fail-fast"
Enter
Wait+Line /Summary/
Sleep 2s
