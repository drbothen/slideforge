---
document_type: adversary-pass-report
story_id: STORY-081
pass: 5
date: 2026-06-09
branch: feature/STORY-081
branch_head_at_review: 46ed9eb9
develop_base: cbebfd57
develop_target_for_rebase: 15838de1
verdict_clean_strict: false
verdict_clean_pr_merge: false
streak_after: "0/3"
findings_count: 2
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 0
findings_obs: 1
findings_open: 2
findings_remediated: 3
---

# Adversary Pass 5 — STORY-081 Slide-Level Inline Markup

**Branch head at review:** `46ed9eb9`
**Date:** 2026-06-09
**Verdict:** NOT CLEAN — 1 MEDIUM + 1 OBS (zero CRIT/HIGH)
**Streak after this pass:** 0/3

---

## Pass Context

Fresh-context full re-review at HEAD `46ed9eb9`. Pass-4's three findings (CRIT-001 overprint, HIGH-001 super/sub size + docstrings, OBS-001) GENUINELY remediated and load-bearing. The CRIT-001 fix introduced ONE new MEDIUM correctness defect in the width-measurement path (face_index discarded). 4th consecutive pass where the prior fix seeded the next defect, all in the PDF inline draw path.

---

## Pass-4 Fix Verification — ALL RESOLVED, LOAD-BEARING

**ADV-P04-CRIT-001 — RESOLVED (load-bearing)**
`compute_multi_span_x_positions` (exporter.rs:1213-1244) maintains horizontal cursor advancing by `measure_text_width_pt` (real cmap→hmtx advances scaled by `font_size/upem`, font.rs:134-180, NOT a constant estimate); wired into `draw_inline_spans_at_y` (1331-1385 @ line 1351). Regression test `test_BC_3_02_002_adv_p04_crit001_multi_span_positions_strictly_increasing` (inline_markup_pdf_snapshot.rs:480-538) asserts `positions[1] > positions[0]`, fails a same-X path. Forbidden "for now" comment GONE.

**ADV-P04-HIGH-001 size — RESOLVED (load-bearing)**
`effective_span_font_size` (1153-1160) returns `base * SUPER_SUB_SCALE` (0.583) for super/sub. `adjusted_baseline_y` (1174-1187) shifts `±base * 0.333` against PARENT size. Constants at exporter.rs:97-109. Tests assert reduced size `== base * 0.583`. Width measured AND drawn both at reduced size (consistent).

**ADV-P04-HIGH-001 docstrings — RESOLVED (load-bearing)**
slide_pdf.rs:1-49/83-94/178-193 describe live `draw_text` baseline-shift mechanism. All `draw_glyphs`/`KrillaGlyph`/`units_per_em` refs NEGATIVE. No `1000.0` math, no MVP comments in draw path.

**OBS-P04-001 — CARRIED (non-blocking)**
Not addressed this pass; carried forward as OBS-P05-001 is the successor.

---

## Findings

### ADV-P05-MED-001 [MEDIUM] — Multi-span cursor width measurement hardcodes face_index 0; wrong glyph advances for .ttc collection faces

**Location:** exporter.rs:1239

**Description:**
`cursor_x += measure_text_width_pt(raw, 0, effective_size, &span.text)` hardcodes `face_index=0`.

The drawn krilla `Font` is constructed at the fontdb-resolved `face_index` in `resolve_styled_face_via_fontdb` (font.rs:406-419, `with_face_data` callback — `face_index` used for draw but DISCARDED). `ResolvedFontSet` (font.rs:72-103) stores only `raw_*` bytes; NO per-face `face_index`.

For `.ttc` collections (macOS Helvetica/Times bundles), fontdb resolves non-zero `face_index`; krilla draws correct face N but `measure_text_width_pt` parses face 0 → measured width from a DIFFERENT face than drawn → multi-span lines mis-spaced (gaps or partial overlap).

The strictly-increasing invariant still holds (advances are positive) so the positional regression test passes, but SPACING is wrong.

`measure_text_width_pt` already accepts `face_index: u32` — it is just not threaded through.

**Untested:** All fixtures are single-face index-0 `.otf`/`.ttf` files.

**Required fix:**
1. Add per-face `face_index` to `ResolvedFontSet`, populated from `with_face_data` callbacks in `resolve_styled_face_via_fontdb`.
2. Thread per-span `face_index` into `compute_multi_span_x_positions`; pass it (not `0`) to `measure_text_width_pt`.
3. Add regression test with `.ttc`/non-zero-index fixture asserting measured width matches drawn face. Cheaper variant: unit test asserting `measure_text_width_pt(ttc, correct_index) != measure_text_width_pt(ttc, 0)`.

---

### OBS-P05-001 [OBS — process-gap] — Width-measurement seam can produce silent overprint when Font present but raw bytes absent

**Location:** exporter.rs:1241 (`compute_multi_span_x_positions`)

**Confidence:** LOW (test-seam; production pairs them)

**Description:**
`compute_multi_span_x_positions` (exporter.rs:1241): if no raw bytes are present, cursor does not advance — all spans land at `start_x`. Production `resolve_font_set` always pairs `raw_*` with `Some(font)` (font.rs:226-298). However, the public `PdfExporter::with_resolved_font_set` seam (lines 222-228) allows `Some(font) + None raw` → font DRAWS but ZERO advance → silent overprint, no diagnostic.

**Codify (route lessons-codification; extends OBS-P04-001):**
When a span has a drawable font but width-measurement cannot measure it, emit `tracing::warn` (non-silent) OR make `ResolvedFontSet` enforce font/raw/face_index pairing as a type invariant (`ResolvedFace { font, raw, face_index }` per slot) so divergence is unrepresentable.

This subsumes ADV-P05-MED-001's root cause: if `ResolvedFace` is introduced as the type invariant, `face_index` is co-located with `raw` and `font`, closing MED-001 structurally.

---

## What Is Sound — Do Not Re-Touch

- Pass-4 CRIT-001 + HIGH-001 remediations (load-bearing, verified above)
- `measure_text_width_pt` algorithm (real cmap→hmtx, f64 intermediate, upem==0 guard, missing-glyph skip, no panics)
- ttf-parser `=0.25.1` pin (single Cargo.lock version, transitive via fontdb 0.23.0)
- slide_pdf.rs docstrings (H1-NEW fully closed)
- Super/sub reduced size + parent-relative shift constants
- fontdb OS/2 query + non-silent `tracing::warn` degradation
- Zero production `unwrap`/`expect`/`println!` in slideforge-pdf/src
- C1-NEW face dispatch + C2-NEW distinctness (eval AC-001/AC-006 conversion + title-strip warning)
- PPTX/DOCX/HTML inline + layout Subtitle/TextRun threading (no regression)

---

## Convergence Trajectory

P2 (1C+2H+2M=5) → P3 (2C+1H+1OBS) → P4 (1C+1H+1OBS) → P5 (1MED+1OBS)

Severity decaying; zero CRIT/HIGH this pass.

---

## Verdict

```
CLEAN (strict): no
CLEAN (PR-merge): no
Streak after this pass: 0/3
```
