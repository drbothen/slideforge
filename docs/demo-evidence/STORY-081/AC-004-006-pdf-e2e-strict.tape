Output docs/demo-evidence/STORY-081/AC-004-006-pdf-e2e-strict.gif
Output docs/demo-evidence/STORY-081/AC-004-006-pdf-e2e-strict.webm

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
Type "cargo build -p slideforge-pdf -p slideforge --tests 2>/dev/null"
Enter
Sleep 90s
Show

# ── AC-004: PDF — bold span uses distinct font resource (C2 distinctness) ────
Type "# AC-004: PDF bold span → distinct font resource embedded (C2 assertion)"
Enter
Type "cargo nextest run -p slideforge-pdf -E 'test(ac004_pdf_bold_span_uses_distinct_font_resource)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-004: PDF — bold/italic face dispatch via ResolvedFontSet ──────────────
Type "# AC-004: Bold→ResolvedFontSet.bold, Italic→ResolvedFontSet.italic face dispatch"
Enter
Type "cargo nextest run -p slideforge-pdf -E 'test(ac004_pdf_bold_uses_font_face_dispatch|ac004_pdf_italic_uses_font_face_dispatch)' --no-fail-fast 2>&1 | tail -6"
Enter
Sleep 8s

# ── AC-004: PDF — superscript RAISED (baseline - offset) + smaller font ──────
Type "# AC-004 PC-5: Superscript raised (baseline-0.333*fs) + smaller (0.583*fs)"
Enter
Type "cargo nextest run -p slideforge-pdf -E 'test(ac004_pc5_superscript_baseline_raised_subscript_lowered)' 2>&1 | tail -5"
Enter
Sleep 7s

# ── AC-004: PDF — code uses mono font face ────────────────────────────────────
Type "# AC-004: InlineNode::Code → ResolvedFontSet.mono face in PDF"
Enter
Type "cargo nextest run -p slideforge-pdf -E 'test(ac004_pdf_code_uses_mono_font_face)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-004: e2e PDF — inline text content preserved, no asterisk artifacts ───
Type "# AC-004 e2e: full PDF pipeline — bold text present, no literal '**' in output"
Enter
Type "cargo nextest run -p slideforge -E 'test(c4_pdf_body_bold_text_present_no_asterisks)' 2>&1 | tail -5"
Enter
Sleep 10s

# ── AC-006 strict mode: title with bold markup → BUILD FAILS (E-EVL-015) ─────
Type "# AC-006 strict (default): title with markup → fatal E-EVL-015 → non-zero exit"
Enter
Type "cargo nextest run -p slideforge -E 'test(fp25_high_001_strict_mode_title_markup_fails_build)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-006 warn-only mode: title with bold markup → WARNING + output produced ─
Type "# AC-006 --warn-only: title markup → warning emitted, build SUCCEEDS, plain title"
Enter
Type "cargo nextest run -p slideforge -E 'test(fp25_high_001_warn_only_title_markup_succeeds)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-006: e2e PPTX — no underscore/asterisk literals in body runs ──────────
Type "# AC-006 e2e: PPTX body output → zero literal ** or _ in a:t text run elements"
Enter
Type "cargo nextest run -p slideforge -E 'test(c4_pptx_body_no_underscore_literals)' 2>&1 | tail -5"
Enter
Sleep 7s

# Hold final frame
Sleep 2s
