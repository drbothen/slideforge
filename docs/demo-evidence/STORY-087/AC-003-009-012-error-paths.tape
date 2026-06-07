Output docs/demo-evidence/STORY-087/AC-003-009-012-error-paths.gif
Output docs/demo-evidence/STORY-087/AC-003-009-012-error-paths.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-087"
Enter
Sleep 1s
Show

# ── Section 1: AC-003 — status missing label → E-A11-002 (WCAG 1.4.1 enforcement) ─
Type "# AC-003: status missing label (strict=true) → Err(ValidationFailed) with E-A11-002 (WCAG 1.4.1)"
Enter
Type "cargo nextest run -p slideforge -E 'test(~ac003_status_missing_label_message_contains_title)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 2: AC-009 — progress_bar missing label → E-A11-002 ──────────────
Type "# AC-009: progress_bar missing label (strict=true) → Err(ValidationFailed) with E-A11-002 message containing 'Sprint 4'"
Enter
Type "cargo nextest run -p slideforge -E 'test(~ac009_progress_bar_missing_label_message_contains_title)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 3: AC-005 — missing label in warn-only mode → Ok ─────────────────
Type "# AC-005: status missing label with strict=false → Ok (warn-only mode, non-conformant output allowed)"
Enter
Type "cargo nextest run -p slideforge -E 'test(~AC_005_status_missing_label_warn_only_is_ok)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 4: AC-012 — progress_bar value=101 → E-VAL-011 ──────────────────
Type "# AC-012: progress_bar value=101 → Err(ValidationFailed) with E-VAL-011 (ValueRangeValidator, Stage 5)"
Enter
Type "cargo nextest run -p slideforge -E 'test(~build_progress_bar_value_101_is_validation_failed)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# ── Section 5: AC-013 — progress_bar value=-1 → E-VAL-011 ───────────────────
Type "# AC-013: progress_bar value=-1 → Err(ValidationFailed) with E-VAL-011 (negative value out of range [0,100])"
Enter
Type "cargo nextest run -p slideforge -E 'test(~build_progress_bar_value_neg1_is_validation_failed)' --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
