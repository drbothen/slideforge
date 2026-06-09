---
document_type: adversary-pass-report
story_id: STORY-081
pass: 3
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 885d8302
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 4
findings_crit: 2
findings_high: 1
findings_med: 0
findings_obs: 1
findings_open: 4
findings_remediated: 0
---

# Adversary Pass 3 — STORY-081 — NOT CLEAN

**Pass context:** Fresh-context full re-review at HEAD 885d8302. Pass-2's 5 findings verified; the text-drop symptom was fixed but exposed a deeper AC-004 font-FACE dead-wiring.

**Pass-2 remediation verification:**
- C1 PARTIAL (text-drop fixed; font-face rendering still dead-wired → C1-NEW)
- C2 NOT MET (test asserts text presence + /ActualText only, not glyph face → C2-NEW)
- C3 FALSE-for-PDF (PPTX/DOCX/HTML render richly; PDF flattens subtitle via `extract_all_inline_text` at exporter.rs:760-764)
- I2/AC-006(3) HTML met (`title_inlines_override` live) but PDF NOT met (title flattened to plain String, exporter.rs:331-341,749)
- I1 CLOSED (genuine `tracing::warn!` at slide_serializer.rs:981-986)

---

## Findings

### C1-NEW [CRITICAL] — PDF inline font-face rendering dead-wired; AC-004 unmet in production

`extract_all_inline_text` (slide_pdf.rs:263-268) discards every `KrillaTextSpan` field except `.text`; `FontFaceKind` (Bold/Italic/Mono) + `y_offset_units` have ZERO production readers. Bold renders in regular face, code not monospace, super/sub on baseline. Code comments say "future enhancement"/"for now" (forbidden MVP rationalization, CLAUDE.md rule 1).

Flatten call sites: exporter.rs:1212,1222 (`draw_body_blocks`), 1341,1355 (`draw_body_blocks_tagged`), 772 (`TextRun`), 760-764 (`SubtitleInlines`), 331-341/749 (rich title).

Required fix per ADR-023: `ResolvedFontSet` via fontdb metadata; consume `.face`/`.y_offset_units`; draw super/sub via `Surface::draw_glyphs`.

### C2-NEW [CRITICAL] — PDF tests vacuous for the AC-004 font-face claim

`test_story_081_c2_production_pdf_draw_path_preserves_inline_text_content` (exporter.rs:3087-3212) asserts text non-empty + `%PDF-` header + `/ActualText` (accessibility text, not drawn glyph face). The `slide_to_krilla_runs` unit tests assert only the pure `FontFaceKind` enum value, never consumed in production. Every PDF test passes whether or not bold renders in a bold face.

Required fix: `build()`-driven assertion distinguishing bold-span font resource from plain-span font resource in actual PDF content stream; must FAIL against the single-face path.

### H1-NEW [HIGH] — Misleading docstrings assert font-switching that does not occur

story_081_inline_markup_e2e.rs:206-208; slide_pdf.rs:25-27. Correct alongside C1-NEW.

### OBS-1 [process-gap] — Exporter-wiring proof anti-pattern recurred a 3rd time

Text-drop → face-drop. Pass-2's recommended exit-gate (grep for ≥1 non-test caller) would NOT catch this because `slide_to_krilla_runs` HAS a non-test caller that discards the load-bearing fields. Suggested codification: exit-gate must verify the non-test caller consumes the dispatch fn's DISTINGUISHING output (`.face`/`.y_offset_units`), not merely that a caller exists. Route to rules/lessons-codification.md.

---

## Architectural Asymmetry Adjudication

Title shadow-field vs SubtitleInlines variant = ACCEPTABLE (not a defect). AC-006(2) requires PPTX title stay single-run plain; widening `FrameContent::Title` would force every PPTX consumer to re-strip; the `title_inlines` shadow-field matches the sound DOCX path. Subtitle has no single-run constraint so `SubtitleInlines` variant is natural. Per source-of-truth precedence (story spec supersedes BC on impl scope).

Caveat: does NOT rescue PDF (C1-NEW is the real defect).

---

## What IS sound (do not re-touch)

- PPTX inline body/subtitle (`inline_node_to_ooxml_runs` live; Math arm warns)
- DOCX inline body/title/subtitle (`make_inline_paragraph` live)
- HTML body/subtitle/title (`render_inline_nodes` + `title_inlines_override` live)
- eval/layout subtitle threading (layout.rs:343-350 → `SubtitleInlines` via `has_non_plain_inline`)

---

## Verdict

```
CLEAN (strict): no
CLEAN (PR-merge): no
Streak after this pass: 0/3
```
