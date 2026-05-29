# AC-004 — Missing logo path returns BrandError::LogoRequired (error path)
Output docs/demo-evidence/STORY-023/AC-004-logo-required-error.gif
Output docs/demo-evidence/STORY-023/AC-004-logo-required-error.webm

Require cargo

Set Shell "bash"
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

Sleep 2s

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-023"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo test -p slideforge-brand ac004_synthesize_rejects_missing_logo"
Enter
Sleep 8s
