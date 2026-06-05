Output docs/demo-evidence/STORY-084/AC-001-005-section-types.gif
Output docs/demo-evidence/STORY-084/AC-001-005-section-types.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree ─────────────────────────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-084"
Enter
Sleep 1s
Show

# ── Visible: run the demo binary (pre-built, completes in < 0.5s) ─────────────
Type "cargo run --example story_084_section_types -p slideforge-plugin-api -q"
Enter
Sleep 10s

# Hold final frame so the viewer can read the results
Sleep 2s
