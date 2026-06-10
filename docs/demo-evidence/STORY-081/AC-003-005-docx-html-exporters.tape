Output docs/demo-evidence/STORY-081/AC-003-005-docx-html-exporters.gif
Output docs/demo-evidence/STORY-081/AC-003-005-docx-html-exporters.webm

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
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-081"
Enter
Sleep 1s
Type "cargo build -p slideforge-docx -p slideforge-html --tests 2>/dev/null"
Enter
Sleep 60s
Show

# ── AC-003: DOCX — all 8 inline forms produce correct w:rPr elements ─────────
Type "# AC-003: DOCX slide body with all 8 inline forms → w:b, w:i, w:rStyle, etc."
Enter
Type "cargo nextest run -p slideforge-docx -E 'test(ac003_docx_slide_body_inline_markup_wired)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-003: show what the test asserts (w:b, w:i, w:strike, w:highlight) ─────
Type "# Verify: w:b (bold), w:i (italic), w:strike (del), w:highlight (mark) in DOCX XML"
Enter
Type "cargo nextest run -p slideforge-docx -E 'test(ac003)' --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 6s

# ── AC-003: e2e DOCX: no underscore literals, w:b present (pipeline test) ────
Type "# AC-003 e2e: DOCX pipeline — w:b run property present; no literal markup"
Enter
Type "cargo nextest run -p slideforge -E 'test(c4_docx_body_bold_produces_wb_run_property)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-005: HTML — all 8 inline forms → semantic elements ────────────────────
Type "# AC-005: HTML slide body → <strong><em><code><a><sup><sub><del><mark>"
Enter
Type "cargo nextest run -p slideforge-html -E 'test(ac005_html_bullet_inline_markup_wired)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-005: e2e HTML — <strong> present in pipeline output ───────────────────
Type "# AC-005 e2e: full pipeline HTML output → <strong> element confirmed present"
Enter
Type "cargo nextest run -p slideforge -E 'test(c4_html_body_bold_produces_strong_element)' 2>&1 | tail -5"
Enter
Sleep 7s

# ── AC-003/AC-005: combined form e2e ─────────────────────────────────────────
Type "# Combined forms e2e: bold+italic, bold+link across DOCX and HTML pipelines"
Enter
Type "cargo nextest run -p slideforge -E 'test(f_p13_e2e_docx_bold_link|f_p13_e2e_html_bold_link|f_p13_e2e_docx_nested|f_p13_e2e_html_nested)' --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 8s

# Hold final frame
Sleep 2s
