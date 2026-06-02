# STORY-045 AC-004 + AC-005: Figure /Alt text + Decorative elements as PDF Artifacts
# AC-004: /Figure elements carry /Alt from DSL alt "..." (including Diagram/Chart forward obligation)
# AC-005: Decorative elements (empty alt) are PDF Artifacts, NOT /Figure in the structure tree
#
# Evidence: run AC-004 and AC-005 related tests from tests/pdf_ua1.rs

Output docs/demo-evidence/STORY-045/AC-004-005-figure-alt-decorative-artifact.gif
Output docs/demo-evidence/STORY-045/AC-004-005-figure-alt-decorative-artifact.webm

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
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-045"
Enter
Wait+Line /\$/
Type "clear"
Enter
Wait+Line /\$/
Show

Type "cargo nextest run -p slideforge-pdf --test pdf_ua1 -E 'test(figure_alt)|test(decorative)|test(diagram_frame)|test(chart_frame)|test(body_level_chart_diagram)|test(ec001_artifacts)' --no-fail-fast"
Enter
Wait+Line /Summary/
Sleep 2s
