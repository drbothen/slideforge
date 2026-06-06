Output docs/demo-evidence/STORY-086/AC-001-019-023-text-routing.gif
Output docs/demo-evidence/STORY-086/AC-001-019-023-text-routing.webm

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

# ── Section 1: AC-001 — PPTX title in title placeholder (not body) ───────────
Type "# AC-001: PPTX title text runs in <p:ph type=title> placeholder, NOT body shape"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac001_pptx_title)' -q 2>&1 | tail -6"
Enter
Sleep 15s

# ── Section 2: AC-019 — body text in body placeholder, distinct from title ───
Type "# AC-019: body text in body placeholder; title and body in DISTINCT <p:sp> shapes"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac019_body_text_in_body_placeholder)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 3: AC-023 — TextTag::Title is tag-driven not position-driven ─────
Type "# AC-023: TextTag routing is tag-driven — reversed block order still routes correctly"
Enter
Type "cargo nextest run -p slideforge-layout -E 'test(ac023)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# ── Section 4: AC-003 — DOCX title in Heading1, co-location verified ─────────
Type "# AC-003: DOCX title routes to Heading1 paragraph (tag-driven, not positional fallback)"
Enter
Type "cargo nextest run -p slideforge -E 'test(ac003)' -q 2>&1 | tail -6"
Enter
Sleep 10s

# Hold final frame
Sleep 2s
