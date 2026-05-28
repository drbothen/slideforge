Output AC-001-mathml.gif
Output AC-001-mathml.webm

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

Type "cargo nextest run -p slideforge-math -E 'test(test_bc_1_10_003_mathml_simple_expression) + test(test_bc_1_10_003_mathml_superscript) + test(test_bc_1_10_003_mathml_fraction) + test(test_bc_1_10_003_mathml_inline_uses_display_inline) + test(test_bc_1_10_003_mathml_display_uses_display_block)' --success-output=immediate-final"
Enter
Sleep 30s
Sleep 2s
