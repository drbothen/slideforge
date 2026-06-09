---
document_type: adversary-pass-report
story_id: STORY-081
pass: 7
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: f1bd9f99
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
findings_obs: 0
findings_open: 0
findings_remediated: 1
---

# Adversary Pass 7 — STORY-081 — CLEAN (strict + PR-merge)

**Verdict: CLEAN (strict): yes / CLEAN (PR-merge): yes / Streak after this pass: 1/3.** First strictly-clean pass.

---

## Pass Context

Fresh-context full re-review at HEAD f1bd9f99. Independently re-derived the complete eval→layout→4-exporter inline path; verified Pass-6 ADV-P06-MED-001 remediation per TD-VSDD-059 (load-bearing). The recurring measure≠draw divergence (P2→P6) is STRUCTURALLY eliminated by the single shared `face_for_span_kind` selector. No 6th defect seeded. Zero findings of any severity.

---

## Pass-6 Remediation Verification (ADV-P06-MED-001 — RESOLVED, load-bearing)

`face_for_span_kind(kind, font_set) -> Option<&ResolvedFace>` (exporter.rs:1219-1234) is the SOLE per-`FontFaceKind` slot selector; called by BOTH `compute_multi_span_x_positions` (measure, exporter.rs:1298) AND `font_for_span` (draw, exporter.rs:1344). Grep confirms no duplicate per-kind dispatch (lines 756/760 are Title/Subtitle single-run plain paths by design). BoldItalic unified to one chain `bold→italic→regular` for both paths. Effective size unified (`effective_span_font_size` both paths).

Load-bearing test `test_adv_p06_med001_bolditalic_measure_uses_same_slot_as_draw_when_bold_absent` (inline_markup_pdf_snapshot.rs:875-994, plain `#[test]`, real fixtures Tuffy+LM-Math) drives production `compute_multi_span_x_positions` on a BoldItalic span with `bold=None`/`italic=Some`; asserts measured advance == italic (Tuffy) and != regular (LM-Math); fails pre-fix, passes post-fix.

---

## New-Defect Hunt (6th-Defect Check): NONE FOUND

All `FontFaceKind` fallback chains correct; nested face merging (`collect_spans` slide_pdf.rs:252-271, EC-003 BoldItalic) correct; selector size↔face consistency; no inconsistent slot read; determinism/Hash intact; zero production `unwrap`/`expect`/`println`/`panic`/`todo` (all in `#[cfg(test)]` or doc comments); non-silent `tracing::warn` degradation; accessibility `/ActualText` intact.

---

## Eval/Layout/Exporter Path Re-Derivation: SOUND

- Eval AC-001: `INLINE_CONTENT_FIELDS → FieldValue::Inlines`, error-recovery fallback
- Eval AC-006: E-EVL-015 `InlineMarkupInTitle` warning + `title_inlines` shadow
- Layout: `BulletItem.inlines → FrameContent::TextRun`
- PDF: `SubtitleInlines`/Body/TextRun → shared `face_for_span_kind` selector
- PPTX: `RunContext` nested b/i
- DOCX/HTML: all 8 forms

---

## Prior-Confirmed-Sound Paths: ALL HOLD

C1-NEW/C2-NEW/CRIT-001/HIGH-001, `ResolvedFace` type invariant, fontdb non-silent degradation, `resolve_regular_face` override `face_index=0`, PPTX/DOCX/HTML inline + eval/layout threading.

---

## Observations (Non-Blocking, Out-of-Scope)

- **Double `load_system_fonts()` per PDF export** (ADR-023-deferred, correct-but-slow, NOT a defect): `resolve_font_set` + `resolve_regular_face` each scan the fontdb. Zero-risk refactor would halve cold cost. Separately tracked as ADV-P06-OBS-002 / FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE.
- **`slideforge-diagrams` cold_budget timing gate** (FU-DIAGRAMS-COLD-BUDGET-TIMING-GATE, separate story, no causal link to STORY-081 inline markup path).

Neither observation blocks strict-CLEAN verdict.

---

## Finding Trajectory

P2(1C+2H+2M=5) → P3(2C+1H=3) → P4(1C+1H=2) → P5(1MED=1) → P6(1MED+2OBS) → P7(0). First strictly-clean pass; streak 1/3.
