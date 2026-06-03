# VHS Demo — STORY-077 Error AC: E-PAR-020
# Empty inline markup span is STRICT-BUILD-FATAL.
# "****" (empty bold) in a section field -> E-PAR-020.
#
# Covers: DIR-077-002 §5, error-taxonomy.md E-PAR-020
#
# Modality: cargo nextest (CLI binary STORY-055 not yet built)
# Commit: f7e4d444

Output .factory/demos/STORY-077/AC-ERR-E-PAR-020-empty-span.gif
Output .factory/demos/STORY-077/AC-ERR-E-PAR-020-empty-span.webm

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

Type "# E-PAR-020: empty inline markup span (****) is strict-build-fatal"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-syntax -E 'test(E_PAR_020)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 30s
