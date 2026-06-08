# AC-002 — Deeply nested SVG rejected after parse, before re-serialization (SEC-001, CWE-674)
# Traces to BC-1.12.003 postcondition 7.
# Tests: test_bc_1_12_003_sec_dos_svg_deep_nesting_rejected
#         test_bc_1_12_003_sec_depth_exact_boundary_passes
#
# Guard uses iterative stack-based DFS over the parsed usvg::Tree (not raw string).
# Root group counts as depth 1.  MAX_SVG_NESTING_DEPTH = 64.
# SVG with 65 nested <g> elements returns E-EXP-004 with depth message.
# SVG with exactly 64 nested <g> elements passes through.
Output docs/demo-evidence/STORY-079/AC-002-deep-nesting-rejected.gif
Output docs/demo-evidence/STORY-079/AC-002-deep-nesting-rejected.webm

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

Type "# AC-002: SEC-001 nesting-depth guard — deeply nested SVG rejected after parse"
Enter
Sleep 1s
Type "cargo nextest run -p slideforge-diagrams -E 'test(sec_dos_svg_deep) + test(sec_depth)' --no-fail-fast"
Enter
Sleep 30s
Sleep 2s
