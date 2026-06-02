Output docs/demo-evidence/STORY-045/AC-012-013-validator-ua1-ci-gate.gif
Output docs/demo-evidence/STORY-045/AC-012-013-validator-ua1-ci-gate.webm
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

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(validator_ua1)|test(ac013)|test(ci_workflow)' 2>&1 | tail -12"
Enter
Sleep 45s

Sleep 2s
