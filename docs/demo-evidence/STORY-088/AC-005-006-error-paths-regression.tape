Output docs/demo-evidence/STORY-088/AC-005-006-error-paths-regression.gif
Output docs/demo-evidence/STORY-088/AC-005-006-error-paths-regression.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-088"
Enter
Sleep 1s
Show

# ── AC-005: non-string list items produce E-PAR-024, NOT a panic ─────────────
Type '# AC-005 (error path): bullets: [42, true] -> E-PAR-024, NOT a panic'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac005_non_string_list_items)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── E-PAR-024 code includes "got integer" ────────────────────────────────────
Type '# E-PAR-024: "got integer" -- integer list element shows type in error message'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(med001a_integer_item_produces_e_par_024)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── E-PAR-024 code includes "got boolean" ────────────────────────────────────
Type '# E-PAR-024: "got boolean" -- boolean list element shows type in error message'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(med001b_boolean_item_produces_e_par_024)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-005: nested list -> E-PAR-024 "got nested list" ───────────────────────
Type '# AC-005 (nested list): bullets: [["x"]] -> E-PAR-024 nested list rejection'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(med_p3_001_deck_nested_list_item)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-006: @var binding form is not regressed ────────────────────────────────
Type '# AC-006: @var items = [...] still parses -- no regression in @var list form'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac006_vars_block_list_assignment)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-006: original ident-reference path still works ────────────────────────
Type '# AC-006 regression guard: bullets: items (ident ref) still parses without error'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac006_regression_bullets_ident)' 2>&1 | tail -5"
Enter
Sleep 5s

# Hold final frame
Sleep 2s
