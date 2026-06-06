Output docs/demo-evidence/STORY-050/AC-001-009-full-e2e-suite.gif
Output docs/demo-evidence/STORY-050/AC-001-009-full-e2e-suite.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-050"
Enter
Sleep 1s
Show

# ── Section 1: Full E2E suite — 62 tests passing ─────────────────────────────
Type "# STORY-050: Full E2E integration test suite (62 tests)"
Enter
Type "cargo nextest run -p slideforge --no-fail-fast 2>&1 | tail -30"
Enter
Sleep 35s

# Hold final frame
Sleep 2s
