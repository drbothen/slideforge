Output docs/demo-evidence/STORY-089/AC-009-010-field-validation-e-val-104.gif
Output docs/demo-evidence/STORY-089/AC-009-010-field-validation-e-val-104.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree, warm the build ─────────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-089"
Enter
Sleep 1s
Type "cargo build -p slideforge --example story_089_field_validation -q 2>/dev/null"
Enter
Sleep 30s
Show

# ── Intro ─────────────────────────────────────────────────────────────────────
Type "# STORY-089: E-VAL-104 field-value type validation live at slideforge::build()"
Enter
Sleep 2s

# ── Full demo: all 4 scenes ───────────────────────────────────────────────────
Type "cargo run --example story_089_field_validation -p slideforge -q"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
