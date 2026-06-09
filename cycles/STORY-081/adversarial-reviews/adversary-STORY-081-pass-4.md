---
document_type: adversary-pass-report
story_id: STORY-081
pass: 4
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 626ae472b4a6f9f0f44096fbc032e35d675b59dd
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 1
findings_high: 1
findings_med: 0
findings_obs: 1
findings_open: 3
findings_remediated: 2
---

# Adversary Pass 4 — STORY-081 Inline Markup

**CLEAN (strict):** no
**CLEAN (PR-merge):** no
**Streak after this pass:** 0/3

---

## Pass Context

Fresh-context full re-review at HEAD 626ae472. Pass-3's C1-NEW + C2-NEW are GENUINELY
remediated (load-bearing, not paper-fixes). But the C1-NEW fix introduced a NEW CRITICAL
rendering regression (multi-span overprinting), and H1-NEW only partially closed
(slide_pdf.rs docstrings still describe an unimplemented draw_glyphs mechanism).

---

## Pass-3 Fix Verification

**C1-NEW — RESOLVED.** `font_for_span` in exporter.rs:1122-1138 consumes `span.face`;
`draw_inline_spans_at_y` (1176-1234) consumes `span.y_offset_units`; wired into
SubtitleInlines 734-737, TextRun 746-748, `draw_body_blocks` 1267/1277,
`draw_body_blocks_tagged` 1395/1409; `generate_pdf_inner` calls `resolve_font_set`
340-343; distinguishing fields genuinely consumed by non-test readers — OBS-1 pattern
broken.

**C2-NEW — RESOLVED.** `test_BC_3_02_002_ac004_pdf_bold_span_uses_distinct_font_resource`
in inline_markup_pdf_snapshot.rs:261-418 injects `ResolvedFontSet{regular:LM-Math,
bold:Tuffy}`, draws Bold+Plain bullets via production `export_uncompressed`, asserts BOTH
PostScript names embed; genuinely fails against single-face.

**H1-NEW — PARTIALLY_RESOLVED.** e2e docstring fixed; slide_pdf.rs:69-70, 78-83, 154-155
still wrong — see HIGH-001.

**OBS-1 — CARRIED.** Routed to follow-up per Pass-3 verdict; not a streak blocker.

---

## Findings

### ADV-P04-CRIT-001 [CRITICAL] — Multi-span inline content overprints at a single X in PDF

**Regression introduced by C1-NEW fix.**

`exporter.rs:1224` in `draw_inline_spans_at_y` (1176-1234): every span is drawn at
`surface_x = emu_to_pt(bbox.x)` with NO horizontal cursor advance between spans. krilla
advances the cursor WITHIN a `draw_text` call, but each span is a separate call resetting
to `bbox.x`. Effect: any bullet/body line with more than one span (canonical example:
`bullets: ["**Key finding**: revenue up 12%"]` which decomposes to
`[Bold("Key finding"), Plain(": revenue up 12%")]`) renders overprinted and illegible —
violates BC-3.02.002 PC8 ("renders as visually bold text").

The pre-C1 path (`draw_text_at_bbox` 1090-1104) drew the whole flattened string in ONE
call, correctly positioned (only the face was wrong). C1-NEW corrected the face at the
cost of horizontal layout.

Untested: the distinctness test uses separate baselines (lines 334-346); the e2e test
asserts only substring presence, not layout.

**Required fix:** `draw_inline_spans_at_y` must maintain a horizontal cursor that
advances by each drawn span's measured width before drawing the next span. A regression
test must assert that span 2's draw-origin X is strictly greater than span 1's — i.e.,
no overprinting on a multi-span line.

The "for now ... correct, if slightly suboptimal" comment at exporter.rs:1172-1175 is a
forbidden MVP rationalization (CLAUDE.md rule 1) masking a visual defect. It must be
removed along with the fix.

---

### ADV-P04-HIGH-001 [HIGH] — slide_pdf.rs docstrings assert a draw_glyphs/KrillaGlyph.y_offset super/subscript mechanism the code does not implement (H1-NEW recurrence + spec divergence)

`slide_pdf.rs:69-70`, 78-83 (`KrillaTextSpan y_offset_units` doc), 154-155. Code
(`exporter.rs:1198-1219`) uses whole-span `draw_text` baseline shift normalized by
hardcoded `1000.0`, NOT `units_per_em`; `draw_glyphs`/`KrillaGlyph` appear only in
comments.

**RESOLUTION — architect ruling 2026-06-09:** The baseline-shift mechanism IS BLESSED.
`draw_glyphs` is unusable in krilla 0.6.0 because `naive_shape` is `pub(crate)` and no
public shaping API exists to obtain `GlyphId`s. The original ADR prescription of
`y_offset = units_per_em/3` was dimensionally wrong (krilla normalizes `y_offset` by
upem at construction; `upem/3` for a 2048-upem TrueType font would equal ~682, which
is nonsensical). ADR-023 has been amended; story spec corrected to v1.3 (commit 52a841e8).

**Implementer must:**
1. Correct `slide_pdf.rs:69-70`, 78-83, 154-155 docstrings to describe the blessed
   `draw_text` mechanism (two calls with adjusted `font_size` and `baseline_y`).
2. Implement the REAL FUNCTIONAL GAP: AC-004/BC implicitly require super/sub to render
   SMALLER (size reduction). The current code does NOT reduce font size. Implementer must
   add module constants and apply them:
   - `SUPER_SUB_SCALE = 0.583` (standard typographic ratio per Unicode TR #25)
   - `SUPER_RISE_FRACTION = 0.333` (superscript baseline-shift as fraction of parent em)
   - `SUB_DROP_FRACTION = 0.333` (subscript drop as fraction of parent em)
   - Baseline shift computed at the PARENT span's font size, not the reduced size.
   - Render: `draw_text(text, font_size * SUPER_SUB_SCALE, baseline_y - (font_size * SUPER_RISE_FRACTION))`
     for superscript; `baseline_y + (font_size * SUB_DROP_FRACTION)` for subscript.

---

### OBS-P04-001 [process-gap] — PDF exporter draw path proven only by "text/face reaches PDF," never "text laid out correctly"

Meta-pattern: Pass-2 text-drop → Pass-3 face-drop → Pass-4 position-collapse. The exit
gate has repeatedly caught the first layer of wiring but missed the next.

Codify: PDF inline-rendering exit-gate must include a multi-span-same-line positional
assertion (two spans on one line → strictly increasing draw-origin X), not only per-face
distinctness. This extends FU-EXIT-GATE-DISTINGUISHING-OUTPUT (from Pass-3 OBS-1).

Route to rules/lessons-codification.md.

---

## What Is Sound — Do Not Re-Touch

- C1-NEW + C2-NEW remediations (load-bearing; do not revert)
- `fontdb =0.23.0` direct pin + ADR-023 supply-chain claim
- `fontdb` OS/2 metadata query + `.ttc` face-index handling
- Non-silent `tracing::warn!` graceful degradation in `resolve_font_set`
- Zero production `unwrap`/`expect`/`println!` in `slideforge-pdf/src`
- PPTX/DOCX/HTML inline + eval/layout subtitle/title threading (no regression in this diff)

---

## Verdict

**CLEAN (strict):** no
**CLEAN (PR-merge):** no
**Streak after this pass:** 0/3

Next action: implementer fix burst — CRIT-001 (horizontal cursor advance in
`draw_inline_spans_at_y` + multi-span-same-line positional regression test + remove "for
now" comment) + HIGH-001 (implement super/sub size reduction with module constants
SUPER_SUB_SCALE/SUPER_RISE_FRACTION/SUB_DROP_FRACTION; correct `slide_pdf.rs:69-70`,
78-83, 154-155 docstrings). Then LESSON-21 exit gate → adversary Pass 5 (fresh 3-clean
streak).
