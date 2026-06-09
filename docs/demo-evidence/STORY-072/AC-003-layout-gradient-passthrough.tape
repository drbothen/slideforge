Output docs/demo-evidence/STORY-072/AC-003-layout-gradient-passthrough.gif
Output docs/demo-evidence/STORY-072/AC-003-layout-gradient-passthrough.webm

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
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-072"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# AC-003: layout::run() passes FillSpec::Gradient through to ShapeFrame.fill verbatim.
# 12 tests: fill preserved, from+to colors verbatim, alt text preserved,
# missing-alt error (EC-001), decorative (EC-006), flat gradient (EC-005),
# multi-shape error accumulation (EC-004)
Type "cargo nextest run -p slideforge-layout -E 'test(gradient)' --no-fail-fast 2>&1 | tail -16"
Enter
Sleep 12s
Sleep 2s
