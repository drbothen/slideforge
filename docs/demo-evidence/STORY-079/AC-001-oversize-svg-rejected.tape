# AC-001 — Oversized SVG rejected before parse (SEC-002, CWE-400)
# Traces to BC-1.12.003 postcondition 7.
# Tests: test_bc_1_12_003_sec_dos_svg_oversize_rejected
#         test_bc_1_12_003_sec_size_one_over_boundary_fails
#         test_bc_1_12_003_sec_size_exact_boundary_passes
#
# Guard rejects any RawDiagramSvg whose byte length exceeds MAX_SVG_BYTES (50 MiB)
# BEFORE calling usvg::Tree::from_str.  Error display contains "E-EXP-004".
# Exact boundary (== 50 MiB) passes; +1 byte over fails.
Output docs/demo-evidence/STORY-079/AC-001-oversize-svg-rejected.gif
Output docs/demo-evidence/STORY-079/AC-001-oversize-svg-rejected.webm

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

Type "# AC-001: SEC-002 byte-size guard — oversize SVG rejected before usvg parse"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-diagrams -E 'test(sec_dos_svg_oversize) + test(sec_size)' --no-fail-fast"
Enter
Sleep 30s
Sleep 2s
