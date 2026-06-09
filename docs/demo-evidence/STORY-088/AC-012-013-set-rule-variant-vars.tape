Output docs/demo-evidence/STORY-088/AC-012-013-set-rule-variant-vars.gif
Output docs/demo-evidence/STORY-088/AC-012-013-set-rule-variant-vars.webm

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

# ── AC-012 positive: set content: bullets ["A","B"] parses to FieldValue::List ─
Type '# AC-012 (positive): set content: bullets ["A","B"] -> FieldValue::List set-rule default'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac012_set_rule_list_literal_parses)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-012 error: set content: bullets [42, true] -> E-PAR-024 ────────────────
Type '# AC-012 (error): set content: bullets [42, true] -> E-PAR-024 non-string element'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac012_set_rule_non_string_list_items)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-012 E-PAR-024 shows "got integer" in set-rule position ─────────────────
Type '# AC-012 E-PAR-024: "got integer" appears in set-rule error message'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(p5_med001_set_rule_integer_item)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-013 positive: variant vars: items: ["A","B"] parses to FieldValue::List ─
Type '# AC-013 (positive): variant vars: items: ["A","B"] -> FieldValue::List override'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac013_variant_vars_list_literal)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-013 error: variant vars non-string -> E-PAR-024 ───────────────────────
Type '# AC-013 (error): variant vars: items: [42, true] -> E-PAR-024 in variant position'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac013_variant_vars_non_string)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-013 nested list rejection in variant vars position ─────────────────────
Type '# AC-013 (nested): variant vars: items: [["x"]] -> E-PAR-024 nested list rejected'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac013_variant_vars_nested_list)' 2>&1 | tail -5"
Enter
Sleep 5s

# Hold final frame
Sleep 2s
