# Paint-State Determinism (OBS-044-22-01)
# Text fill color is NOT leaked from a preceding SVG/Diagram frame on the same Surface.
# draw_text_at_bbox explicitly sets solid-black fill before surface.draw_text().
# Traces to OBS-044-22-01.
# Tests: test_obs_044_22_01_text_fill_not_leaked_from_svg_frame
Output docs/demo-evidence/STORY-044/AC-PAINT-paint-state-determinism.gif
Output docs/demo-evidence/STORY-044/AC-PAINT-paint-state-determinism.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-044"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p slideforge-pdf -E 'test(obs_044_22_01)'"
Enter
Sleep 20s
Sleep 2s
