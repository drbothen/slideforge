Output docs/demo-evidence/STORY-072/AC-001-dsl-gradient-syntax-accepted.gif
Output docs/demo-evidence/STORY-072/AC-001-dsl-gradient-syntax-accepted.webm

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

# AC-001: DSL parser accepts fill gradient "#FF0000" to "#0000FF" syntax
# Three passing tests: E-PAR-016 removed, gradient syntax accepted (upper+lower hex),
# interim AST string pinned as exactly "gradient #FF0000 to #0000FF"
Type "cargo nextest run -p slideforge-syntax -E 'test(gradient)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
