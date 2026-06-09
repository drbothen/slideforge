Output docs/demo-evidence/STORY-072/AC-004-pptx-gradFill-emission.gif
Output docs/demo-evidence/STORY-072/AC-004-pptx-gradFill-emission.webm

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

# AC-004 PPTX: native a:gradFill emission via ooxmlsdk typed builders.
# 9 tests: a:gradFill present, stop positions 0/100000, a:lin ang=5400000,
# stop colors match from/to, from precedes to in XML, alt text on cNvPr,
# decorative empty descr, EC-005 same-from-to, full slide builds
Type "cargo nextest run -p slideforge-pptx -E 'test(gradient)' --no-fail-fast 2>&1 | tail -14"
Enter
Sleep 12s
Sleep 2s
