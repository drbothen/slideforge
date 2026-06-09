Output docs/demo-evidence/STORY-072/AC-004-html-svg-linearGradient.gif
Output docs/demo-evidence/STORY-072/AC-004-html-svg-linearGradient.webm

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

# AC-004 HTML: SVG-native <defs><linearGradient id="sf-grad-..."> emission.
# fill="url(#sf-grad-...)" on shape <rect> (NOT CSS background-color).
# 8 tests: linearGradient style present, from/to colors, alt accessible name,
# decorative aria-hidden, EC-005 same-from-to, CSS helper format/direction/uppercase
Type "cargo nextest run -p slideforge-html -E 'test(gradient)' --no-fail-fast 2>&1 | tail -13"
Enter
Sleep 10s
Sleep 2s
