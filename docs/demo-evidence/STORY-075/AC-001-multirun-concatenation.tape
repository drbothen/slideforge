# AC-001 (multi-run clause): Footer text split across multiple <a:r> runs is fully
# concatenated in document order. "First run only" semantics are explicitly forbidden.
# Test: test_bc_2_01_001_detect_footer_text_multirun
# Expected: two runs "Acme Confidential " + "2026" -> "Acme Confidential 2026"
# Traces to BC-2.01.001 postcondition 1 v1.3 + adversary M-1 clarification.
Output docs/demo-evidence/STORY-075/AC-001-multirun-concatenation.gif
Output docs/demo-evidence/STORY-075/AC-001-multirun-concatenation.webm

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

Type "cargo nextest run -p slideforge-brand -E 'test(detect_footer_text_multirun)'"
Enter
Sleep 20s
Sleep 2s
