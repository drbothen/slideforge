# STORY-042 — DOCX Auto-Generated Document Sections
# VHS tape: AC-001 through AC-008 coverage via `cargo run --example demo_sections`
#
# Records the `demo_sections` example running against the live implementation.
# Every PASS line in the output corresponds to one or more acceptance criteria.

Output docs/demo-evidence/STORY-042/AC-ALL-demo-sections.gif
Output docs/demo-evidence/STORY-042/AC-ALL-demo-sections.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 680
Set FontSize 14
Set Padding 20
Set Framerate 24
Set PlaybackSpeed 1.0

# ── Run the pre-built demo_sections binary directly ───────────────────────────
# The binary is already compiled at target/debug/examples/demo_sections.
# Running it directly avoids any cargo compilation timeout in VHS.
Type "/Users/jmagady/Dev/slideforge/.worktrees/STORY-042/target/debug/examples/demo_sections"
Enter
Sleep 4s

# Hold final frame so all AC lines are visible
Sleep 2s
