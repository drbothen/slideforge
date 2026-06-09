Output docs/demo-evidence/STORY-088/AC-001-004-list-literal-parse.gif
Output docs/demo-evidence/STORY-088/AC-001-004-list-literal-parse.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree, warm cargo cache ───────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-088"
Enter
Sleep 1s
Type "cargo build -p slideforge-syntax --tests 2>/dev/null"
Enter
Sleep 60s
Show

# ── AC-001: bullets: ["A","B","C"] parses to FieldValue::List ────────────────
Type '# AC-001: bullets: ["A","B","C"] parses to FieldValue::List(Vec<FieldValue>)'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac001_bullets_field_list_literal)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-002: bullets: [] empty list parses to FieldValue::List([]) ─────────────
Type '# AC-002: bullets: [] parses to FieldValue::List(vec![]) -- empty list valid'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac002_empty_list_literal)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-003: single-item list bullets: ["Only"] is a list not a bare string ────
Type '# AC-003: bullets: ["Only"] is FieldValue::List, not a bare string'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac003_single_item_list)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-004: list-literal inside full slide block (multiple fields) ─────────────
Type '# AC-004: list-literal alongside title + body fields -- no partial-parse failure'
Enter
Type "cargo nextest run -p slideforge-syntax -E 'test(ac004_list_literal_in_full_slide)' 2>&1 | tail -5"
Enter
Sleep 5s

# Hold final frame
Sleep 2s
