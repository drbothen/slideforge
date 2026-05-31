# AC-002 — notes content excluded from visual frames (no-bleed invariant)
# Traces to BC-1.14.001 postcondition 1: notes NOT in slide body shapes
Output docs/demo-evidence/STORY-035/AC-002-notes-no-bleed-to-frames.gif
Output docs/demo-evidence/STORY-035/AC-002-notes-no-bleed-to-frames.webm

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

Type "cargo nextest run -p slideforge-layout -E 'test(test_bc_3_06_001_register_tags_notes) | test(test_f004_no_bleed_register_text_not_in_frames)'"
Enter
Sleep 15s
Sleep 2s
