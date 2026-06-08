# VHS Demo — STORY-074 AC-003
# BrandFonts::default() backward compatibility: font_size_emu=457_200
# preserves pre-STORY-074 output, identical to old DEFAULT_EM_IN_EMU constant.
# All 3 AC-003 tests plus full 11-test suite pass (regression guard).

Output docs/demo-evidence/STORY-074/AC-003-default-brand-compat.gif
Output docs/demo-evidence/STORY-074/AC-003-default-brand-compat.webm

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

Type "cargo nextest run -p slideforge-layout --test brand_em_sizing 2>&1 | grep -E 'ac003|default_brand|Summary'"
Enter
Sleep 12s
Sleep 2s
