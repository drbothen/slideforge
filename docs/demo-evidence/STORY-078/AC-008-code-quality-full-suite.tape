# AC-008 — Code quality invariants: clippy pedantic clean + full nextest suite green
# Traces to BC-3.02.002 precondition 1, NFR-021, NFR-022, NFR-024
# Demonstrates: all 23 section tests pass; full 316-test slideforge-syntax suite green
Output docs/demo-evidence/STORY-078/AC-008-code-quality-full-suite.gif
Output docs/demo-evidence/STORY-078/AC-008-code-quality-full-suite.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-078"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-syntax --no-fail-fast 2>&1 | tail -6"
Enter
Sleep 30s
