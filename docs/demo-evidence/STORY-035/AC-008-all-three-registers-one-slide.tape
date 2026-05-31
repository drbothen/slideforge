# AC-008 — all three registers on one slide: exactly 3 entries, frames carry only visual content
# Traces to BC-1.14.004 invariant 3: LaidOutSlide carries register-tagged content
Output docs/demo-evidence/STORY-035/AC-008-all-three-registers-one-slide.gif
Output docs/demo-evidence/STORY-035/AC-008-all-three-registers-one-slide.webm

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

Type "cargo nextest run -p slideforge-eval -E 'test(test_bc_1_14_004_all_three_registers_on_one_slide) | test(test_f003_eval_deck_populates_register_content_from_all_three_fields)'"
Enter
Sleep 15s
Sleep 2s
