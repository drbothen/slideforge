Output docs/demo-evidence/STORY-050/AC-009-missing-alt-accessibility-enforcement.gif
Output docs/demo-evidence/STORY-050/AC-009-missing-alt-accessibility-enforcement.webm

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

# ── Section 1: AC-009 — strict=true rejects missing alt (E-A11-001) ──────────
Type "# AC-009: chart missing alt → E-A11-001 (strict rejects; warn-only passes)"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac009)' -q 2>&1 | tail -10"
Enter
Sleep 20s

# ── Section 2: AC-006 — parse error propagation with file:line:col ────────────
Type "# AC-006: bad syntax → ParseFailed with structured diagnostics"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac006)' -q 2>&1 | tail -8"
Enter
Sleep 15s

# Hold final frame
Sleep 2s
