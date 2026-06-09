---
document_type: adversary-pass-report
story_id: STORY-081
pass: 2
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: c11d6468
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 5
findings_crit: 1
findings_high: 2
findings_med: 2
findings_obs: 0
findings_open: 5
findings_remediated: 0
---

# STORY-081 Adversary Pass 2 Report

## Pass Metadata

| Field | Value |
|-------|-------|
| Branch | `feature/STORY-081` |
| Branch HEAD at review | `c11d6468` |
| develop base at branch cut | `cbebfd57` |
| develop target for rebase | `15838de1` (rebase at PR step) |
| Pass date | 2026-06-09 |
| CLEAN (strict) | **no** — 1 CRIT + 2 HIGH + 2 MED |
| CLEAN (PR-merge) | **no** — CRIT + HIGH present |
| Streak | 0/3 (reset) |
| Story | BC-3.02.002 v1.5, P0 / 13 pts, EPIC-18 (touches eval + layout + all 4 exporters) |

## Pass Context

Pass 2 is a full re-review of the complete STORY-081 rebuild. Pass 1 found 4 CRIT
dead-wiring defects. The implementer responded with a full re-implementation:
eval threads `FieldValue::Inlines`; PPTX inline runs + Code/Super/Sub/Highlight;
DOCX highlight + rich title; PDF `ActualText` via `slide_to_krilla_runs`; keystone
e2e test (`build()` → `b="1"` / `<w:b/>` / `<strong>` / PDF ActualText, no literal `**`).
1,190 tests pass. Pass 2 found live routing gaps and a new test vacuity issue.

## Findings Summary

| ID | Severity | Status | Title |
|----|----------|--------|-------|
| C1 | CRIT | OPEN | PDF body/bullet inline markup DEAD-WIRED — `slide_to_krilla_runs` has zero production callers |
| C2 | HIGH | OPEN | PDF tests vacuous for the production draw path |
| C3 | HIGH | OPEN | Subtitle inline markup dropped at layout seam for ALL exporters |
| I1 | MED | OPEN | PPTX Math-in-body silently dropped with no tracing::warn! |
| I2 | MED | OPEN | AC-006(3) partial — HTML/PDF rich title rendering deferred |

## Detailed Findings

### C1 [CRITICAL] — PDF body/bullet inline markup DEAD-WIRED

**Severity:** CRITICAL
**Violates:** AC-004, BC-3.02.002 PC8 v1.5, no-silent-drop constraint

**Root cause:** `slide_to_krilla_runs` in `crates/slideforge-pdf/src/slide_pdf.rs:153` has
**zero production callers**. The live draw path for body blocks uses `draw_body_blocks` /
`draw_body_blocks_tagged` (located in `crates/slideforge-pdf/src/exporter.rs:1143` /
`:1246`, Body arm ~line 423). These call `extract_inline_text` at `exporter.rs:1337-1351`
whose catch-all `_ => {}` arm **silently drops** text of:
Code, Strikethrough, Highlight, Superscript, Subscript, Link, Xref, Footnote, Math.

Additionally, `draw_body_blocks` draws bold/italic in the default face with no face switch
and no `y_offset` adjustment.

**Effect:** Bold and italic text appear in the PDF but use the default typeface. Code,
strikethrough, highlight, superscript, subscript, link, xref, footnote, and math text are
**silently dropped** — they produce no output in the PDF at all.

**Required fix:** Route PDF Body/Bullets draw path through `slide_to_krilla_runs` (which
implements per-span face selection + `y_offset`). The `_ => {}` catch-all arm in
`extract_inline_text` must never drop text — unknown variants must at minimum emit their
text content, logged via `tracing::warn!`.

---

### C2 [HIGH] — PDF tests vacuous for the production draw path

**Severity:** HIGH
**Violates:** TD-VSDD-059 (paper-fix detection), BC-5.39.001 test quality bar

**Root cause:** `crates/slideforge-pdf/tests/inline_markup_pdf_snapshot.rs` only
unit-tests `slide_to_krilla_runs` in isolation — it never builds a PDF via the
production pipeline. The e2e test `test_story_081_c4_pdf_body_bold_text_present_no_asterisks`
(at `crates/slideforge/tests/e2e/story_081_inline_markup_e2e.rs:219-264`) asserts only
that "bold text" is present and that `"**"` is absent. This assertion passes even while
C1 is broken (bold text IS emitted by the buggy path — just wrong typeface; and `**` is
indeed absent).

**Required fix:** Add a `build()`-driven PDF assertion (PDF byte extraction) that:
1. Every inline markup span's text content is present in the PDF output.
2. Bold/code spans produce a distinct font face vs. plain text (verifiable via the PDF
   font reference in the content stream, or via `ActualText` attributes).

---

### C3 [HIGH] — Subtitle inline markup dropped at layout seam for ALL exporters

**Severity:** HIGH
**Violates:** AC-001, BC-3.02.002 PC8 v1.5

**Root cause:** The eval + field-threading stages correctly convert subtitle to inlines:
- `for_eval.rs:99` — `INLINE_CONTENT_FIELDS` includes `subtitle`
- `field_to_block.rs:128-144` — threads subtitle through inline conversion

However, layout flattens the inline structure:
- `layout.rs:336-347` — `TextTag::Subtitle` arm calls `extract_inline_text_str(...)` which
  reduces inlines to plain text
- `FrameContent::Subtitle(Arc<str>)` in `crates/slideforge-types/src/types.rs:410` carries
  plain text only — inline structure is discarded here

**Effect:** No asterisk leak (the `**` delimiters are stripped), but the inline structure
is entirely lost. Subtitle markup (bold, italic, code, etc.) renders as plain text in ALL
four exporters (PPTX, DOCX, PDF, HTML).

**Required fix:** Widen `FrameContent::Subtitle` to carry inline structure, either:
- `FrameContent::Subtitle(Vec<InlineNode>)` (preferred), or
- Route subtitle as a tagged body block through the same path as body content

Update all four exporter arms that consume `FrameContent::Subtitle` to render richly.

**Note:** Same `FrameContent::Title(Arc<str>)` seam applies to I2/AC-006(3) — fixing
subtitle at this seam makes fixing title nearly zero marginal cost (see AC-006(3)
adjudication below).

---

### I1 [MEDIUM] — PPTX Math-in-body silently dropped, no warning

**Severity:** MEDIUM
**Violates:** no-silent-drop constraint, tracing discipline

**Location:** `crates/slideforge-pptx/src/slide_serializer.rs:945-948`

**Root cause:** `InlineNode::Math(_) => vec![]` — math inline in body content produces
no output and emits no warning or log entry.

**Required fix:** Either:
- Emit `tracing::warn!(story = "STORY-081", "Math inline in body not yet rendered in PPTX — EC-008 pattern")` and produce empty output (acceptable as a documented limitation), OR
- Implement OMML rendering for math-in-body if within scope

At minimum the silent drop must produce a diagnostic per the EC-008 pattern.

---

### I2 [MEDIUM] — AC-006(3) partial — HTML/PDF rich title deferred

**Severity:** MEDIUM
**Violates:** AC-006 clause (3): rich title rendering for HTML and PDF

**Root cause:** `title_inlines` (at `for_eval.rs:361-373`) is consumed only by DOCX
(`document_body.rs:178-200`). HTML (`render.rs:386-391`) and PDF render the plain
`Arc<str>` title from `FrameContent::Title`, not the inlines. No asterisk leak occurs
(de-marking strips `**`), but inline structure is lost.

**Note on severity:** This is the same `FrameContent::Title(Arc<str>)` seam as C3. Since
C3 already requires widening the `FrameContent` Title/Subtitle seam to carry inlines,
fixing HTML/PDF rich title is near-zero marginal cost in the same burst (see AC-006(3)
adjudication below).

---

## AC-006(3) Adjudication — OPEN DECISION FOR HUMAN

**Adversary recommendation:** Do NOT grant the HTML/PDF rich-title deferral as a
standalone tech-debt entry. Reasoning:

1. C3 (subtitle) already requires widening `FrameContent::Title`/`FrameContent::Subtitle`
   to carry `Vec<InlineNode>` instead of `Arc<str>`.
2. Once that seam is widened, wiring HTML and PDF to consume the inline structure adds
   minimal incremental effort.
3. The production-grade default (CLAUDE.md rule 3) requires explicit human direction to
   defer, a concrete future dependency that makes deferral necessary, AND a specific
   future story anchor. None of those conditions are met here.

**Therefore:** The adversary recommends fixing HTML/PDF rich title in the same burst as
C3. If the human elects to defer, it MUST be:
- A human-directed decision (not AI self-authorization)
- Anchored to a specific future story ID
- Recorded in tech-debt-register.md with all three CLAUDE.md rule-3 conditions met

**RESOLVED 2026-06-09 (human): fix HTML/PDF rich title in same burst as C3 — not deferred.**

---

## What IS Sound — Do Not Re-Touch

The following paths are genuinely live and correctly wired. Do not regress them:

- **PPTX inline body path** — `slide_serializer.rs` runs, bold/italic/code/strikethrough/
  highlight/super/sub/link produce correct `<a:r>` with `<a:rPr>` attributes
- **DOCX inline body path** — `document_body.rs` runs, highlight/bold/italic/code produce
  correct `<w:r>` with `<w:rPr>` attributes; rich title consumed from `title_inlines`
- **HTML body path** — `render.rs` Body arm emits `<strong>`, `<em>`, `<code>`, etc.
  correctly via inline rendering
- **e2e keystone test structure** — `test_story_081_c4_pdf_body_bold_text_present_no_asterisks`
  correctly calls `build()` end-to-end; its assertion logic needs strengthening (C2), not
  its pipeline invocation

---

## Process Gap Identified

**Exporter-wiring proof anti-pattern recurred (Pass 1 → Pass 2):**

Pattern: "rewrite pure dispatch fn + unit-test in isolation, but leave production draw
path on old flatten helper" appeared in both Pass 1 and Pass 2 for the PDF exporter.

**Recommended checklist item (codify as implementer exit-gate rule):**

For every new dispatch function added to an exporter:
1. `grep` the exporter's serializer/draw module to confirm the new function has ≥1 caller
   that is NOT a test function.
2. Add a `build()`-driven assertion (not just unit isolation test) that exercises the
   production call path end-to-end.

This closes the "vacuous isolation test" failure mode that has recurred across passes.

---

## Next Steps

1. **Human decision on AC-006(3):** Confirm whether HTML/PDF rich title is fixed in the
   same burst as C3 (adversary-recommended) or deferred with a human-directed tech-debt
   anchor.
2. **Implementer fix burst:** Address C1 + C2 + C3 + I1, and HTML/PDF title per
   AC-006(3) decision.
3. **Exit gate:** Full pre-push gate (LESSON-21: nextest + shared-process cargo test +
   rustdoc + cross-platform filesystem discipline).
4. **Adversary Pass 3:** Fresh-context re-review. 3-CLEAN streak starts here (0/3 → need
   3 consecutive clean passes).
