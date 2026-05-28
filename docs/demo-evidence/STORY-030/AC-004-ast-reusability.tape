Output AC-004-ast-reusability.gif
Output AC-004-ast-reusability.webm

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

Type "cargo nextest run -p slideforge-math -E 'test(test_bc_1_10_003_mathml_ast_reusable_across_calls) + test(test_bc_1_10_003_pdf_paths_ast_reusable_across_calls)' --success-output=immediate-final"
Enter
Sleep 30s
Sleep 2s
