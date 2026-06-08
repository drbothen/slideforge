# VHS Demo — STORY-074 AC-002
# DEFAULT_EM_IN_EMU is no longer pub in production code (test-only).
# Success: test accessing the private const passes (value confirmed 457_200).
# Error-path: grep finds no pub const in production shapes.rs.

Output docs/demo-evidence/STORY-074/AC-002-default-em-removed.gif
Output docs/demo-evidence/STORY-074/AC-002-default-em-removed.webm

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
Set TypingSpeed 40ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-074"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-layout -p slideforge-types 2>&1 | grep -E 'ac002|default_em|Summary'"
Enter
Sleep 12s
Sleep 1s

Type "grep -n 'pub const DEFAULT_EM_IN_EMU' crates/slideforge-layout/src/shapes.rs && echo FAIL || echo 'CONFIRMED: no pub const DEFAULT_EM_IN_EMU in production path'"
Enter
Sleep 3s
Sleep 2s
