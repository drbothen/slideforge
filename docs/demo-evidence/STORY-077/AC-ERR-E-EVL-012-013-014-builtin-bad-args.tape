# VHS Demo — STORY-077 Error AC: E-EVL-012 / E-EVL-013 / E-EVL-014
# Empty-arg or missing-arg builtins are strict-build-fatal at eval time:
#   figref() with no/bad arg  -> E-EVL-012
#   ref("") with empty id     -> E-EVL-013
#   footnote() with no/bad arg -> E-EVL-014
#
# Covers: DIR-077-002 §5, error-taxonomy.md E-EVL-012/013/014
#
# Modality: cargo nextest (CLI binary STORY-055 not yet built)
# Commit: f7e4d444

Output .factory/demos/STORY-077/AC-ERR-E-EVL-012-013-014-builtin-bad-args.gif
Output .factory/demos/STORY-077/AC-ERR-E-EVL-012-013-014-builtin-bad-args.webm

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
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-077"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# E-EVL-012/013/014: bad-arg builtins figref/ref/footnote are strict-build-fatal at eval"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-eval -E 'test(e_evl_012)|test(e_evl_013)|test(e_evl_014)|test(figref)|test(footnote_zero)|test(ref_zero)|test(ref_empty)|test(footnote_empty)|test(footnote_ident)|test(ref_ident)' --no-fail-fast 2>&1 | tail -25"
Enter
Sleep 30s
