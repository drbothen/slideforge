# AC-003 — footer_text Some("") clears footer; None means no change
# BC-2.02.001 postcondition 2 + invariant 3: Some("") distinct from None
Output docs/demo-evidence/STORY-025/AC-003-footer-some-empty-vs-none.gif
Output docs/demo-evidence/STORY-025/AC-003-footer-some-empty-vs-none.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-025"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-brand -p slideforge-types -E 'test(test_bc_2_02_001_resolve_overlay_footer_text_some_empty_distinct_from_none) | test(test_bc_2_02_001_resolve_overlay_footer_text_none) | test(test_bc_2_02_001_invariant_footer_some_empty_distinct_from_none)'"
Enter
Sleep 30s
Sleep 2s
