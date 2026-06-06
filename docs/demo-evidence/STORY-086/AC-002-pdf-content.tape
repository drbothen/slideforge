Output docs/demo-evidence/STORY-086/AC-002-pdf-content.gif
Output docs/demo-evidence/STORY-086/AC-002-pdf-content.webm

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

# ── Section 1: AC-002 — PDF byte stream contains title and body text ─────────
Type "# AC-002: PDF byte stream contains both 'Heading' and 'Body paragraph text'"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac002_pdf_contains_body_text)' -q 2>&1 | tail -6"
Enter
Sleep 15s

# ── Section 2: AC-018 — Wave 4 Gate 3 re-pass: all 3 formats strict=true ─────
Type "# AC-018: Wave 4 Gate 3 re-pass — 3-slide deck strict=true (PPTX/PDF/DOCX all pass)"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac018)' -q 2>&1 | tail -8"
Enter
Sleep 15s

# Hold final frame
Sleep 2s
