# AC-008 — --offline NEVER produces empty data silently; skipped = absent from scope
Output docs/demo-evidence/STORY-021/AC-008-offline-never-empty-silent.gif
Output docs/demo-evidence/STORY-021/AC-008-offline-never-empty-silent.webm

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

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-021"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-008: invariant 1 — no empty Value::Map substitution; skipped source absent from scope"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-data -E 'test(offline_does_not_silently_substitute_empty_value)' --no-fail-fast"
Enter
Sleep 8s
Sleep 2s
