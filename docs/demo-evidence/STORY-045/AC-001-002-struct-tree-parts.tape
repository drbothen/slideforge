Output docs/demo-evidence/STORY-045/AC-001-002-struct-tree-parts.gif
Output docs/demo-evidence/STORY-045/AC-001-002-struct-tree-parts.webm
Require cargo
Set Shell bash
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

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-045 && PS1='$ ' && clear"
Enter
Sleep 3s
Show

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(struct_tree_root)|test(one_part_per_slide)' 2>&1 | tail -8"
Enter
Sleep 25s

Sleep 2s
