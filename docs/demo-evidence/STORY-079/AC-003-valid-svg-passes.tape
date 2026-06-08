# AC-003 — Valid SVG within limits passes through unaffected
# Traces to BC-1.12.003 postcondition 7.
# Tests: test_bc_1_12_003_sec_dos_svg_valid_input_unaffected
#
# Guards MUST NOT alter normalisation output for legitimate SVG.
# Real-world Mermaid flowchart SVG (well within both limits) returns Ok(NormalizedDiagramSvg).
# This is the non-regression test: existing behaviour is unchanged after guards were added.
Output docs/demo-evidence/STORY-079/AC-003-valid-svg-passes.gif
Output docs/demo-evidence/STORY-079/AC-003-valid-svg-passes.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-079"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-003: Valid SVG within both guards passes through — normalisation unchanged"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-diagrams -E 'test(sec_dos_svg_valid)' --no-fail-fast"
Enter
Sleep 30s
Sleep 2s
