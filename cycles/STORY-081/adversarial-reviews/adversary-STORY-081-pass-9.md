---
document_type: adversary-pass-report
story_id: STORY-081
pass: 9
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: c739d59c84f54f8e779c45817b606194a3e99ca4
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: true
verdict_clean_pr_merge: true
streak_after: "1/3"
findings_count: 0
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 0
findings_obs: 1
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 9 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict): yes / CLEAN (PR-merge): yes / Streak after this pass: 1/3.**

Fresh-context independent re-review at HEAD c739d59c. Pass-8 ADV-P08-HIGH-001 (PDF title rendered regular-face) FULLY + LOAD-BEARINGLY remediated. Full fresh re-derivation of every seam found NO new gap. Implementation has converged on correctness.

---

## Pass-8 Remediation Verification (load-bearing, TD-VSDD-059)

`draw_frame` signature now `title_inlines_override: Option<&[InlineNode]>` (exporter.rs:760). Flatten-to-String (`extract_all_inline_text`) GONE from Title draw path — now only used for `/ActualText` in `tag_engine.rs`, which is a legitimate and correct use. `FrameContent::Title` arm (exporter.rs:763-779) renders richly via `slide_to_krilla_runs + draw_inline_spans(.., 36.0)` when inlines present (Bold→bold, Italic→italic, Code→mono via shared `face_for_span_kind`); falls back to plain regular-face `draw_text_at_bbox(36.0)` when `None`. Title size 36.0 preserved. All FOUR call-sites thread `title_inlines_nodes.as_deref()`: decorative (479), single-block-tagged (513), multi-block-fallback (563), untagged (581).

Load-bearing test `test_BC_3_02_002_adv_p08_high001_pdf_title_bold_uses_distinct_font_resource` (inline_markup_pdf_snapshot.rs:1044-1225) drives production `export_uncompressed`, injects `title_inlines=[Bold("Bold"), Plain(" Title")]`, asserts BOTH Tuffy (bold) + LatinModernMath-Regular present. Fails against flatten path (Tuffy absent). No new defect seeded: PPTX title stays single-run plain (`FrameContent::Title(Arc<str>)` by type, slide_serializer.rs:411 `build_shape` plain); DOCX (document_body.rs:187) + HTML (exporter.rs:375) title-inline paths unchanged + symmetric with PDF (all read `title_inlines` via `deck.slides.get(source_index)` + `FieldValue::Inlines`, no off-by-one).

---

## Independent Re-Derivation (all seams, equal scrutiny): SOUND

**Eval:** AC-001 (`INLINE_CONTENT_FIELDS=[bullets,body,caption,description,subtitle]`, title excluded), AC-006 (E-EVL-015 warning + plain title + `title_inlines` shadow), EC-007 (resolved-var stays `Plain`, `chunks_has_inline_markup` matches structural variants not `Expr`; test :553), error-recovery fallback.

**Layout:** title→`FrameContent::Title(Arc<str>)` always plain; subtitle→`SubtitleInlines` when non-Plain (layout.rs:343); `FrameContent` Hash+Eq+Clone (ADR-013).

**PPTX:** all-8-forms via `RunContext`, nested b/i, super/sub, Code→Courier, Link→runs+rels, Highlight→`solidFill` (EC-008-directed), Math→`tracing::warn`+empty, title single-run.

**DOCX:** all-8 structural + scheme-validated hyperlink + rich title via `make_inline_paragraph`.

**HTML:** all-8 semantic + `is_safe_link_scheme` + rich title/subtitle.

**PDF:** Bold/Italic/Code distinct faces via `ResolvedFontSet`+`face_for_span_kind` (single shared measure==draw selector, ADV-P06-MED-001 structurally eliminated), super/sub reduced size 0.583 + point-space baseline shift, non-silent fontdb fallback.

**Cross-cutting:** zero production `unwrap`/`expect`/`println` (all in `#[cfg(test)]`/doc), integer-EMU coords, Hash determinism, `/ActualText` coverage, e2e drives all 4 formats from real DSL asserting format-native markup + no literal `**`.

---

## Findings

### OBS-P09-001 [OBS — out-of-scope, non-blocking]

`slideforge-diagrams` cold_budget timing gate (`FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE`) + double `load_system_fonts` per PDF export — no causal link to STORY-081 correctness. This finding has NO effect on convergence streak. However, it MUST be resolved before STORY-081 PR (per pre-PR blocker registered in STATE.md). Carry forward as open follow-up.

---

## Novelty Assessment

**LOW.** Pass-8's single finding fully closed; fresh re-derivation of every seam (eval/layout/PPTX/DOCX/HTML/PDF × body/title/subtitle/caption/description × all-8-forms+Math+nested) found NO new gap. Body-span axis (P2→P6) and title-leg axis (P8) both structurally eliminated + re-confirmed sound.

---

## Trajectory

P2(5)→P3(3)→P4(2)→P5(1)→P6(1MED+2OBS)→P7(0,1/3)→P8(1HIGH,reset)→P9(0,1/3)

---

## Next Step

Adversary Pass 10 (sequential, LESSON-7; same HEAD c739d59c — no code change between clean passes). Need 2 more consecutive strict-CLEAN to converge 3/3.
