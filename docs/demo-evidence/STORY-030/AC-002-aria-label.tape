Output AC-002-aria-label.gif
Output AC-002-aria-label.webm

Require cargo
Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-030"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-math -E 'test(test_bc_1_10_003_mathml_has_aria_label) + test(test_bc_1_10_003_aria_label_for_pythagorean) + test(test_bc_1_10_003_aria_label_for_simple_sum) + test(test_bc_1_10_003_aria_label_empty_ast_returns_fallback)' --success-output=immediate-final"
Enter
Sleep 30s
Sleep 2s
