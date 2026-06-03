# VHS Demo — STORY-077 AC-003 / AC-004
# section detail: -> RegisteredContent{Register::Detail}
# section report: -> RegisteredContent{Register::Report}
#
# AC-003: detail: sub-block produces RegisteredContent{Register::Detail} on the section node
# AC-004: report: sub-block produces RegisteredContent{Register::Report} on the section node
#
# Modality: cargo nextest (CLI binary STORY-055 not yet built)
# Commit: f7e4d444

Output .factory/demos/STORY-077/AC-003-004-section-register-routing-happy-path.gif
Output .factory/demos/STORY-077/AC-003-004-section-register-routing-happy-path.webm

Require cargo

Set Shell "zsh"
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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-077"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-003/004: detail: -> Register::Detail, report: -> Register::Report on section node"
Enter
Sleep 500ms

Type "cargo nextest run -p slideforge-eval -E 'test(ac003)|test(ac004)' --no-fail-fast 2>&1 | tail -20"
Enter
Sleep 30s
