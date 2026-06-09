---
document_type: adversary-pass-report
story_id: STORY-081
pass: 8
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: f1bd9f99
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 1
findings_crit: 0
findings_high: 1
findings_med: 0
findings_low: 0
findings_obs: 2
findings_open: 1
findings_remediated: 0
---

# Adversary Pass 8 — STORY-081 Slide-Level Inline Markup

**Verdict: CLEAN (strict): no / CLEAN (PR-merge): no / Streak after this pass: 0/3.**

Fresh-context independent re-review at HEAD f1bd9f99 (unchanged since Pass 7). Prior passes were PDF-body-anchored; this pass re-derived the AC-006 dual-title path across ALL FOUR exporters and found a real load-bearing gap 7 prior passes missed.

---

## Findings

### ADV-P08-HIGH-001 [HIGH] — PDF title with inline markup renders in the REGULAR face, not bold; AC-006 clause 3 unimplemented for the PDF exporter

**Seam:** `crates/slideforge-pdf/src/exporter.rs` — FrameContent::Title rendering path.

**Evidence:**

- `exporter.rs:419-429`: extracts `title_inlines` shadow field but FLATTENS to plain `String` via `extract_all_inline_text(nodes)`, discarding `InlineNode::Bold` / `InlineNode::Italic` / `InlineNode::Code`.
- `exporter.rs:751-758`: `FrameContent::Title` arm uses `font_set.regular` UNCONDITIONALLY.
- All four `draw_frame` call-sites (lines 475, 509, 559, 577) pass `title_inlines_text.as_deref()` — a plain `Option<&str>`, never `Vec<InlineNode>`.

**Root cause:** PDF I2 dual-title was wired as a plain-text alternative title source (markup-stripped), NOT rich inline rendering. The inline comment rationalizing "PDF inherits the same convention — no per-word bold in a title heading" is a self-authored rationalization with NO spec backing.

**Spec divergence (3 sources naming PDF explicitly):**

1. AC-006 clause 3 (story line 281-282): "DOCX, PDF, HTML output with the title rendered as bold"
2. EC-002 (story line 524): "DOCX/PDF/HTML output has bold title"
3. DIR-077-002 §4 (line 358): inline structure for "title for DOCX/PDF"; single-run constraint PPTX-only

**Cross-exporter asymmetry:**

- DOCX (`document_body.rs:187-194`): `make_inline_paragraph` → `<w:b/>` — CONFORMS
- HTML (`render.rs:976-993`): `render_inline_nodes` → `<strong>` — CONFORMS
- PDF: flattens to plain string, renders regular face — DOES NOT CONFORM

Effect: `title:"**Bold Title**"` → bold in DOCX + HTML, plain (non-bold) in PDF — a multi-renderer parity defect and BC-3.02.002 PC8 violation extended to titles by AC-006.

**Capability already exists:** The `SubtitleInlines` path (`exporter.rs:766-771`) uses `slide_to_krilla_runs` + `draw_inline_spans` — the title path was simply never wired to it.

**Coverage gap (TD-VSDD-059):** `inline_markup_pdf_snapshot.rs` — only Title fixture is plain (line 349). `story_081_inline_markup_e2e.rs` PDF assertions cover BODY bold only (line 223); zero PDF title-bold coverage. This gap is what allowed the defect to survive 7 passes.

**Required fix (in-scope, production-grade — no deferral):**

1. Capture title `Vec<InlineNode>` (not flattened `String`) from the `title_inlines` shadow field; thread `Option<&[InlineNode]>` to `draw_frame` in parallel to the DOCX/HTML approach.
2. `FrameContent::Title` arm: when title inlines are present, render via `slide_to_krilla_runs(title_inlines)` + `draw_inline_spans` (same path as `SubtitleInlines`) so `Bold`/`Italic`/`Code` dispatch to correct `ResolvedFontSet` face. Fall back to regular-face plain when no inline structure.
3. Remove the contradicting rationalization comment.
4. Add a load-bearing PDF test asserting that a bold-markup title uses/embeds a distinct bold face (mirror the C2-NEW distinctness pattern; drive from production export path).

---

## Observations (non-blocking)

### OBS-P08-001 [OBS, non-blocking] — PPTX InlineNode::Highlight glyph-fill vs background semantics

`slide_serializer.rs:1031-1044`: `InlineNode::Highlight` renders via `<a:solidFill>` on run `rPr` = glyph FILL color (yellow text), not background highlight. The code comment "functionally equivalent background highlight" is technically inaccurate. HOWEVER: story EC-008 explicitly directs this ("emit a yellow solid-color highlight fill on the run via typed builder") — code conforms to its spec. Under the source-of-truth standing rule (code already conforms to its spec), this is NOT a divergence. Noted for spec author to revisit; does NOT gate this pass.

### OBS-P08-002 [OBS, out-of-scope] — Timing gate + double load_system_fonts

`slideforge-diagrams` `cold_budget` timing gate (`FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE`) + double `load_system_fonts` per PDF export — correct-but-slow; no causal link to the title finding. Pre-existing; tracked under FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE.

---

## Confirmed Sound (re-derived this pass; all hold)

- Eval AC-001: `INLINE_CONTENT_FIELDS→Inlines` + error-recovery
- AC-006: `E-EVL-015` warning + `title_inlines` shadow field wiring
- Layout: `SubtitleInlines` variant + PPTX single-run title
- PPTX: nested bold/italic + strike/super/sub/Math-warn/Link
- DOCX: all 8 inline forms + rich title
- HTML: all 8 inline forms + rich title/subtitle
- PDF BODY/subtitle: shared `face_for_span_kind` selector — `measure == draw`; effective size; `face_index`; non-silent degradation — body path sound; TITLE path defective only
- Zero production `unwrap`/`expect`/`println!`
- Integer-EMU coordinates
- Hash determinism
- `/ActualText` coverage

---

## Process Note

AC-006(3) PDF rich title was CLAIMED closed at fix-burst-2 (human-adjudicated 2026-06-09 to fix alongside C3). HTML closure was real; PDF closure was only half-real — wired to flattened title text, not rich rendering. The per-exporter claim was not independently verified at the time; Pass-8 fresh title-seam re-derivation caught it.

**Lesson:** Multi-exporter "rich rendering" closures must be verified PER EXPORTER with a load-bearing test each, not by analogy.

---

## Trajectory

P2(5) → P3(3) → P4(2) → P5(1) → P6(1 MED + 2 OBS) → P7(0, 1/3) → P8(1 HIGH, reset 0/3)

**Novelty: HIGH** — new seam (AC-006 dual-title PDF leg), not a retread of the P2→P6 body-span axis (which is structurally eliminated and re-confirmed sound).

---

## Next Action

Implementer fix burst:

1. Wire `FrameContent::Title` to render `title_inlines` richly via `slide_to_krilla_runs` + `draw_inline_spans` (mirror `SubtitleInlines` path).
2. Thread `Vec<InlineNode>` to `draw_frame`; remove rationalization comment.
3. Add load-bearing PDF title-bold distinctness test (mirror C2-NEW pattern; assert distinct embedded bold face).

Then: LESSON-21 exit gate → adversary Pass 9 (fresh 3-clean streak).

REMINDER: FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE must be resolved before the STORY-081 PR (separate from convergence).
