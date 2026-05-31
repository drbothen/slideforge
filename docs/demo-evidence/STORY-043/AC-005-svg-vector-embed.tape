# AC-005 — SVG paths embedded as vector (no rasterization): usvg -> krilla Surface.
# Traces to BC-4.03.002 postcondition 4.
# Tests: test_bc_4_03_002_svg_embed_converts_rect_to_vector_paths
#         test_bc_4_03_002_svg_str_is_parseable_by_usvg
Output docs/demo-evidence/STORY-043/AC-005-svg-vector-embed.gif
Output docs/demo-evidence/STORY-043/AC-005-svg-vector-embed.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-043"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(svg)'"
Enter
Sleep 20s
Sleep 2s
