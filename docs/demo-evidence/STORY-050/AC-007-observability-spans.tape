Output docs/demo-evidence/STORY-050/AC-007-observability-spans.gif
Output docs/demo-evidence/STORY-050/AC-007-observability-spans.webm

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

# ── Section 1: AC-007 — 6 canonical tracing spans ────────────────────────────
Type "# AC-007: 6 canonical pipeline spans — parse/evaluate/brand/validate/layout/export"
Enter
Type "cargo nextest run -p slideforge -E 'test(observability)' -q 2>&1 | tail -8"
Enter
Sleep 20s

# ── Section 2: AC-004/005 — registry dog-fooding, no bypass paths ─────────────
Type "# AC-004/005: no cross-exporter deps; external plugin registers alongside bundled"
Enter
Type "cargo nextest run -p slideforge -E 'test(registry) or test(external_plugin)' -q 2>&1 | tail -8"
Enter
Sleep 20s

# Hold final frame
Sleep 2s
