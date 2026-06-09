---
document_type: adversary-pass-report
story_id: STORY-081
pass: 6
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 577767eccbcabff8519d62d2a4c6101cb3843c37
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 3
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 0
findings_obs: 2
findings_open: 3
findings_remediated: 2
---

# Adversary Pass 6 — STORY-081 Slide-Level Inline Markup

**CLEAN (strict):** no
**CLEAN (PR-merge):** no
**Streak after this pass:** 0/3

---

## Pass Context

Fresh-context re-review at HEAD 577767ec. Pass-5 ResolvedFace refactor (MED-001 + OBS-P05-001) GENUINELY remediated and load-bearing. But fresh re-derivation surfaced a NEW measure/draw divergence in the BoldItalic fallback (5th consecutive pass where the prior PDF fix seeds the next PDF defect). Severity decaying — zero CRIT/HIGH 2nd consecutive pass.

---

## Pass-5 Verification

Both Pass-5 findings RESOLVED and load-bearing:

**ADV-P05-MED-001 — RESOLVED (load-bearing):** `ResolvedFace{font, raw, face_index}` (font.rs:69-95) co-locates all three fields. `resolve_styled_face_via_fontdb` (font.rs:455-497) captures `face_index` from `with_face_data` callback and builds krilla `Font` at SAME index. `compute_multi_span_x_positions` (exporter.rs:1224-1264) passes `face.face_index` (not 0) to `measure_text_width_pt`. Load-bearing test `test_adv_p05_med001_obs_p05_001_resolved_face_carries_face_index` (inline_markup_pdf_snapshot.rs:700-829) asserts that measurement differs by face index.

**OBS-P05-001 — RESOLVED (load-bearing):** Type invariant makes `Some(font) + None raw` unrepresentable. All constructor and access sites swept (LESSON-19), compile-checked against new shape.

---

## Findings

### ADV-P06-MED-001 [MEDIUM] — BoldItalic span: measurement-path and draw-path use divergent fallback chains

**Location:** exporter.rs:1243-1245 (measure path), exporter.rs:1294-1299 (draw path)

**Measure path** (`compute_multi_span_x_positions`, exporter.rs:1243-1245):
```
Bold | BoldItalic => bold.or(regular)
```

**Draw path** (`font_for_span`, exporter.rs:1294-1299):
```
BoldItalic => bold.or(italic).or(regular)
```

**Defect:** When `bold=None` and `italic=Some` (reachable — `resolve_font_set` queries bold/italic via independent fontdb queries; a system can have italic but no bold for a family), the measure path selects REGULAR while the draw path selects ITALIC. Different glyph advances → cursor mis-spacing.

**Reachability:** Reachable via nested bold+italic (EC-003 `**_bold italic_**`, `collect_spans` slide_pdf.rs:252-271). Effect: multi-span lines with a nested bold-italic run mis-space on systems with italic-but-no-bold for the brand font family.

**Why tests miss it:** The strictly-increasing invariant still holds (advances are positive even when wrong), so the positional regression test does not catch this — same blind spot as the original MED-001. All current fixtures populate bold, so the `bold=None` code path is never exercised.

**Required fix:**
1. Extract a single `face_for_span_kind(span_face, font_set) -> Option<&ResolvedFace>` helper called from BOTH `compute_multi_span_x_positions` AND `font_for_span`. This guarantees identical slot selection → identical `raw`/`face_index` for measure and draw.
2. Align the BoldItalic fallback to ONE chain: `bold → italic → regular`.
3. Add a load-bearing regression test: `bold=None`, `italic=Some` with distinct metrics, `BoldItalic` span. Assert that the measured advance uses the SAME face the draw path selects (i.e. italic, not regular).

---

### ADV-P06-OBS-001 [OBS] — Full suite not executed; cold_budget claim unverifiable

**Scope adjudication:** Full suite NOT executed (adversary read-only profile; Bash/exec denied). The `cold_budget` claim cannot be empirically confirmed or refuted by adversary.

**Static analysis:** `crates/slideforge-diagrams/tests/cold_budget.rs:46-86` `test_cold_budget_under_200ms` is a wall-clock timing gate (300ms macOS/Linux budget, ~58ms/19% headroom; comment records 242ms observed) — inherently non-deterministic; genuine PR-CI flake vector.

**Scope adjudication:** `cold_budget.rs` is NOT in the STORY-081 diff (STORY-081 = eval/layout/pptx/docx/pdf/html, not diagrams). STORY-081's new `load_system_fonts()` calls are in slideforge-pdf (font.rs:310-311, 418-419), and do NOT feed the diagrams `FONT_DB` OnceLock — there is NO causal link from STORY-081 to the diagrams cold_budget test.

The implementer's "flake passes in isolation" claim is consistent with timing-jitter on a shared CI runner. This is OUT of STORY-081 perimeter; should NOT block STORY-081 convergence; route to diagrams crate owner as a pre-existing structural issue.

---

### ADV-P06-OBS-002 [OBS, process-gap] — Wall-clock timing gate is a structural false-green/false-fail vector; STORY-081 adds double-scan without a call-count gate

**Structural issue:** Wall-clock timing as a test is a false-green/false-fail vector: false-fail on runner jitter; false-green when a 2x regression still lands under budget. The regression it guards ("accidental per-call font loading") is better caught by a deterministic call-count assertion (`load_count == 1`) than a timing threshold.

**STORY-081 contribution:** STORY-081 adds a double `load_system_fonts()` per PDF export — `resolve_font_set` (font.rs:310) + `resolve_regular_face` (font.rs:418) = two full scans per export. Yet slideforge-pdf has NO cold-budget or call-count gate. This is correct-but-slow; an ADR-023-deferred OnceLock optimization (NOT a defect / production-grade-compliant). A zero-risk refactor — reuse the `db` built in `resolve_font_set` inside `resolve_regular_face` — would halve the cold cost.

**Lessons-codification (OBS-002):** Pair or replace timing-threshold tests with a deterministic behavioral assertion of the proxied property (e.g., assert `load_count == 1` via a call-counting wrapper rather than asserting wall time).

---

## What is Sound (Do Not Re-touch)

- Pass-5 ResolvedFace invariant (`font + raw + face_index` co-located)
- `face_index` capture and use for all faces EXCEPT the BoldItalic slot-selection divergence
- `resolve_regular_face` override with `face_index: 0`
- Super/sub reduced size (0.583) + parent-relative shift (0.333) measured AND drawn consistently
- `measure_text_width_pt` algorithm
- Mono and non-nested Italic fallback chains MATCH between measure/draw paths
- Zero production `unwrap`/`expect`/`println!`
- Non-silent `tracing::warn` degradation paths
- Eval AC-001/AC-006 + E-EVL-015 warning
- C1-NEW/C2-NEW/CRIT-001/HIGH-001 (all prior-pass closures)
- PPTX/DOCX/HTML inline + eval/layout threading

---

## Finding Trajectory

```
P2: 1 CRIT + 2 HIGH + 2 MED = 5
P3: 2 CRIT + 1 HIGH      = 3
P4: 1 CRIT + 1 HIGH      = 2
P5: 1 MED                = 1
P6: 1 MED + 2 OBS        = 3 (0 CRIT/HIGH)
```

Same root pattern (measure != draw) re-surfacing one level up on each pass. A shared `face_for_span_kind` helper should end the cycle by making it structurally impossible for the two paths to diverge.

---

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) | no — 1 MED + 2 OBS |
| CLEAN (PR-merge) | no — 1 MED |
| Streak after this pass | 0/3 |
