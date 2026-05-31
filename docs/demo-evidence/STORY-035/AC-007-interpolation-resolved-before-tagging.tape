# AC-007 — {{ expr }} interpolation resolved BEFORE register tagging
# Traces to BC-1.14.001 postcondition 1: notes content uses {{ expr }} interpolation
# Shows: raw template -> eval_deck -> RegisteredContent with resolved value "Quarter: Q1"
Output docs/demo-evidence/STORY-035/AC-007-interpolation-resolved-before-tagging.gif
Output docs/demo-evidence/STORY-035/AC-007-interpolation-resolved-before-tagging.webm

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

Type "cargo nextest run -p slideforge-eval -E 'test(test_bc_1_14_001_interpolation_resolved_before_tagging) | test(test_f_p3_002_interpolation_resolved_before_register_tagging_eval_deck)'"
Enter
Sleep 15s
Sleep 2s
