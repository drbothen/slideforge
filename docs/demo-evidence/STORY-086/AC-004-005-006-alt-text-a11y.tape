Output docs/demo-evidence/STORY-086/AC-004-005-006-alt-text-a11y.gif
Output docs/demo-evidence/STORY-086/AC-004-005-006-alt-text-a11y.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-086"
Enter
Sleep 1s
Show

# ── Section 1: AC-004 — chart WITH alt + strict → Ok, PPTX carries descr ────
Type "# AC-004: chart WITH alt + strict=true -> Ok (PPTX carries accessibility descr)"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac004_strict_chart_with_alt)' -q 2>&1 | tail -6"
Enter
Sleep 15s

# ── Section 2: AC-005 — chart WITHOUT alt + strict → Err E-A11-001 ───────────
Type "# AC-005: chart WITHOUT alt + strict=true -> Err(ValidationFailed) + exactly 1 E-A11-001"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac005_strict_chart_no_alt)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 3: AC-006 — decorative chart + strict → Ok (no E-A11-001) ────────
Type "# AC-006: decorative:true chart + strict=true -> Ok (AltText::Decorative is valid opt-out)"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac006_decorative_chart)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 4: AC-007 (unit) — Value::List produces ContentBlock::Bullets ────
Type "# AC-007 (unit): Value::List -> ContentBlock::Bullets path (Stage 2b pure threading)"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac007_value_list)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
