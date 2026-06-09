Output docs/demo-evidence/STORY-072/AC-002-fillspec-gradient-ir-construction.gif
Output docs/demo-evidence/STORY-072/AC-002-fillspec-gradient-ir-construction.webm

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

# AC-002: FillSpec::Gradient { from: Rgb, to: Rgb } IR construction
# Five tests: variant constructable, Hash+Eq+Clone, u8 channels (no f64),
# inequality, distinct from SolidColor/None; EC-005 same-from-to valid
Type "cargo nextest run -p slideforge-types -E 'test(gradient)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
