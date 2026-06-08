# VHS Demo — STORY-074 AC-001
# Brand font_size_emu drives em->EMU conversion.
# 48pt brand (609_600 EMU) and 24pt brand (304_800 EMU) produce DIFFERENT
# resolved EMU for the same Em(1000) shape. All 11 brand_em_sizing tests pass.

Output docs/demo-evidence/STORY-074/AC-001-brand-em-sizing.gif
Output docs/demo-evidence/STORY-074/AC-001-brand-em-sizing.webm

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

Type "cargo nextest run -p slideforge-layout --test brand_em_sizing 2>&1"
Enter
Sleep 12s
Sleep 2s
