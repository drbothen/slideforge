Output docs/demo-evidence/STORY-072/AC-005-alt-text-required-on-gradient-shapes.gif
Output docs/demo-evidence/STORY-072/AC-005-alt-text-required-on-gradient-shapes.webm

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

# AC-005: alt text + decorative semantics preserved for gradient shapes.
# Error path: gradient shape missing alt => LayoutError::MissingAlt (EC-001)
# Success path: gradient shape with alt => AltText::Provided preserved verbatim
# Decorative: gradient shape with decorative:true => AltText::Decorative (EC-006)
Type "cargo nextest run -p slideforge-layout -E 'test(ac005) + test(ec006) + test(missing_alt)' --no-fail-fast 2>&1 | tail -12"
Enter
Sleep 12s
Sleep 2s
