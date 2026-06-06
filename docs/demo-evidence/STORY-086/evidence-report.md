# STORY-086 Demo Evidence Report

**Story:** STORY-086 — Stage 2b: Post-Eval Field-to-Block Threading Pass + TextTag Routing (AltText::Unspecified)
**Crates:** `slideforge-eval`, `slideforge-types`, `slideforge-layout`, `slideforge-validate`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge`
**Branch:** `feature/STORY-086`
**Recorded:** 2026-06-06
**Recording tool:** VHS (terminal capture of `cargo nextest` tests against real pipeline output)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-019-023-text-routing.gif` | GIF (PR embed) | AC-001, AC-003, AC-019, AC-023 |
| `AC-001-019-023-text-routing.webm` | WEBM (archival) | AC-001, AC-003, AC-019, AC-023 |
| `AC-001-019-023-text-routing.tape` | VHS script | AC-001, AC-003, AC-019, AC-023 |
| `AC-002-pdf-content.gif` | GIF (PR embed) | AC-002, AC-018 |
| `AC-002-pdf-content.webm` | WEBM (archival) | AC-002, AC-018 |
| `AC-002-pdf-content.tape` | VHS script | AC-002, AC-018 |
| `AC-004-005-006-alt-text-a11y.gif` | GIF (PR embed) | AC-004, AC-005, AC-006, AC-007 |
| `AC-004-005-006-alt-text-a11y.webm` | WEBM (archival) | AC-004, AC-005, AC-006, AC-007 |
| `AC-004-005-006-alt-text-a11y.tape` | VHS script | AC-004, AC-005, AC-006, AC-007 |

---

## Coverage Map

### AC-001 — Title text routes to PPTX title placeholder, NOT a generic body shape

**Demonstrated in recording:** `AC-001-019-023-text-routing` — Section 1
**Evidence:** `cargo nextest run -p slideforge -E 'test(ac001_pptx_title)'` runs:
- `test_bc_1_16_001_ac001_pptx_title_text_run_is_nonempty` — asserts `<a:t>My Title</a:t>` in PPTX slide XML
- `test_bc_4_01_001_ac001_pptx_title_in_title_placeholder_not_body` — co-location assertion: "My Title" MUST appear inside the `<p:sp>` carrying `<p:ph type="title"/>`, and MUST NOT appear in any other shape

Both tests PASS. The co-location test would fail if title text were placed in a body shape or if TextTag routing were absent.

**Result:** PASS

---

### AC-002 — Content slide body text visible in PDF

**Demonstrated in recording:** `AC-002-pdf-content` — Section 1
**Evidence:** `test_bc_1_16_001_ac002_pdf_contains_body_text_strings` builds `story-086-content-slide.sf`
(title slide + content slide with `title "Heading"` and `body "Body paragraph text"`) via the PDF exporter
and asserts both strings appear in the raw PDF byte stream. Stage 2b threads these fields into
`ContentBlock::Text(TextTag::Body)` entries which flow through layout → `FrameContent::Body` →
`tag_engine.rs` `/ActualText` emission.

**Result:** PASS

---

### AC-003 — Title routes to DOCX Heading1 via TextTag (NOT positional fallback)

**Demonstrated in recording:** `AC-001-019-023-text-routing` — Section 4
**Evidence:** `cargo nextest run -p slideforge -E 'test(ac003)'` runs:
- `test_bc_1_16_001_ac003_docx_heading1_run_carries_title_text` — "Report Title" in `<w:t>` node
- `test_bc_4_02_001_ac003_docx_title_in_heading1_with_pstyle` — co-location assertion: the SAME `<w:p>` block must contain BOTH `<w:pStyle w:val="Heading1"/>` AND "Report Title"

The co-location test verifies tag-driven routing (not the removed positional fallback).

**Result:** PASS

---

### AC-004 — strict + chart WITH alt → build Ok, PPTX placeholder carries descr

**Demonstrated in recording:** `AC-004-005-006-alt-text-a11y` — Section 1
**Evidence:** `test_bc_1_16_001_ac004_strict_chart_with_alt_build_ok_and_pptx_carries_descr` builds
`story-086-chart-with-alt.sf` (`chart_type "bar"`, `alt "Bar chart showing Q3 revenue by region"`)
with `strict: true` and asserts: (a) build returns `Ok`, (b) PPTX slide XML contains the alt string.
Stage 2b resolves the alt field to `AltText::Provided(...)` → `thread_media_alt_into_frames` threads it
into the frame → `AltTextEmbedder` emits the `descr` attribute.

**Result:** PASS (success path)

---

### AC-005 — strict + chart WITHOUT alt → Err(ValidationFailed) with exactly one E-A11-001

**Demonstrated in recording:** `AC-004-005-006-alt-text-a11y` — Section 2
**Evidence:** `test_bc_1_16_001_ac005_strict_chart_no_alt_returns_validation_error` builds
`story-086-chart-no-alt.sf` (chart, no alt field) with `strict: true` and asserts:
(a) build returns `Err`, (b) error variant is `BuildError::ValidationFailed`, (c) diagnostics
contain exactly 1 `E-A11-001` code. Stage 2b produces `ContentBlock::Chart { alt: None }` →
`thread_media_alt_into_frames` maps `None` → `AltText::Unspecified` → `validate_post_layout`
fires `E-A11-001`.

**Result:** PASS (error path)

---

### AC-006 — decorative:true on a chart → strict Ok, empty descr / PDF Artifact

**Demonstrated in recording:** `AC-004-005-006-alt-text-a11y` — Section 3
**Evidence:** `test_bc_1_16_001_ac006_decorative_chart_strict_ok_no_e_a11_001` builds
`story-086-chart-decorative.sf` (`decorative: true`, no alt string) with `strict: true` and
asserts build returns `Ok`. Stage 2b calls `resolve_alt(decorative=true, alt=None)` →
`AltText::Decorative` → `validate_post_layout` sees Decorative → no `E-A11-001` emitted.

**Result:** PASS (success path for decorative opt-out)

---

### AC-007 — Bullets via @var binding produce ContentBlock::Bullets (unit test)

**Demonstrated in recording:** `AC-004-005-006-alt-text-a11y` — Section 4
**Evidence:** `test_bc_1_16_001_ac007_value_list_produces_content_block_bullets` in
`crates/slideforge-eval/tests/field_to_block_unit.rs` directly constructs a `Value::List`
(bypassing DSL parsing, per AC-007 NOTE) and calls `thread_fields_to_blocks`, asserting
the result contains `ContentBlock::Bullets` with 3 items. The E2E `.sf` fixture remains
`#[ignore]` pending STORY-088 (DSL list-literal parser).

**Result:** PASS (unit path; E2E path deferred to STORY-088 per spec)

---

### AC-018 — Wave 4 Gate 3 re-pass: 3-slide deck builds cleanly under strict=true

**Demonstrated in recording:** `AC-002-pdf-content` — Section 2
**Evidence:** `cargo nextest run -p slideforge -E 'test(ac018)'` runs three tests covering
`story-086-wave4-gate3.sf` (title slide + content slide + chart slide with alt) across all
three output formats:
- `test_bc_5_02_001_ac018_wave4_gate3_repass_pptx_strict_ok_nonempty` — PPTX, strict, title text "Annual Review" present
- `test_bc_5_02_001_ac018_wave4_gate3_repass_pdf_strict_ok_nonempty` — PDF, strict, title text present
- `test_bc_5_02_001_ac018_wave4_gate3_repass_docx_strict_ok_nonempty` — DOCX, strict, Heading1 present

All three PASS. This is the primary Wave 4 Gate 3 re-pass confirmation — the gate that was
unconditionally failing before Stage 2b (due to `Slide.blocks = vec![]` and `AltText::Decorative`
structural placeholders triggering false-positive E-A11-001).

**Result:** PASS

---

### AC-019 — Body text routes to PPTX body placeholder, NOT title placeholder

**Demonstrated in recording:** `AC-001-019-023-text-routing` — Section 2
**Evidence:** `test_bc_4_01_001_ac019_body_text_in_body_placeholder_not_title` parses `slide2.xml`
from `story-086-content-slide.sf` into individual `<p:sp>` blocks and performs co-location assertions:
(a) title shape with `<p:ph type="title"/>` contains "Heading" but NOT "Body paragraph text"
(b) a distinct non-title shape contains "Body paragraph text" but NOT "Heading"
(c) the two shapes are distinct `<p:sp>` elements

**Result:** PASS

---

### AC-020 — DOCX body text routes to Normal paragraph, NOT Heading1

**Not directly shown in recordings** (demonstrated through AC-018 DOCX test which covers
the full pipeline including body routing). The test `test_bc_4_02_001_ac020_docx_body_not_in_heading1_paragraph`
passes as part of the STORY-086 test suite. It verifies at the `<w:p>` block level that the
`<w:p>` containing "Body paragraph text" does NOT carry `<w:pStyle w:val="Heading1"/>`.

**Result:** PASS (covered by full suite run)

---

### AC-021 — PPTX subtitle routes to subtitle placeholder (or body fallback)

**Not directly in a dedicated recording.** The test `test_bc_4_01_001_ac021_pptx_subtitle_in_placeholder`
passes — it verifies that a `slide title:` block with both `title "Main Title"` and `subtitle "Subtitle text"`
produces PPTX XML containing both strings.

**Result:** PASS (covered by full suite run)

---

### AC-022 — DOCX subtitle routes to Heading2 paragraph

**Not directly in a dedicated recording.** The test `test_bc_4_02_001_ac022_docx_subtitle_in_heading2`
passes — it verifies that `word/document.xml` contains both "Chapter Subtitle" text and
`<w:pStyle w:val="Heading2"/>` for a slide with `subtitle "Chapter Subtitle"`.

**Result:** PASS (covered by full suite run)

---

### AC-023 — TextTag::Title is tag-driven not position-driven (invariant verification)

**Demonstrated in recording:** `AC-001-019-023-text-routing` — Section 3
**Evidence:** `cargo nextest run -p slideforge-layout -E 'test(ac023)'` runs two layout unit tests:
- `test_bc_4_01_001_ac023_reversed_blocks_title_gets_title_region_body_gets_body_region` — constructs `Slide.blocks` with `[ContentBlock::Text(Body), ContentBlock::Text(Title)]` in reversed order and asserts layout still places Title content in the title frame
- `test_bc_4_01_001_ac023_texttag_title_is_tag_driven_not_position_driven` — explicit invariant test confirming no position-based routing fires

Both PASS. These tests would fail if `layout::run` used `blocks[0]` as title detection.

**Result:** PASS

---

## CI Gate Confirmation

All CI gates ran clean before recording (LOCAL adversarial convergence: 3/3 CLEAN):

| Gate | Command | Result |
|------|---------|--------|
| STORY-086 E2E tests | `cargo nextest run -p slideforge -E 'test(story_086)'` | CLEAN (15/15 PASS) |
| Field-to-block unit tests | `cargo nextest run -p slideforge-eval -E 'test(field_to_block)'` | CLEAN (7/7 PASS) |
| AC-007 Value::List unit test | `cargo nextest run -p slideforge-eval -E 'test(ac007_value_list)'` | CLEAN (1/1 PASS) |
| Layout AC-023 tag-over-position tests | `cargo nextest run -p slideforge-layout -E 'test(ac023)'` | CLEAN (2/2 PASS) |
| Alt text threading integration test | `cargo nextest run -p slideforge-layout -E 'test(chart_alt_none_maps_to_unspecified)'` | CLEAN |

---

## ACs Not Given Dedicated Recordings

| AC | Reason | Evidence |
|----|--------|---------|
| AC-008 | Unit test coverage (empty title → no block) — not user-visible behavior requiring own recording | Covered in field_to_block unit suite |
| AC-009 | Purity / idempotency test — not visual behavior | Covered in field_to_block unit suite |
| AC-010 | Canonical block ordering unit test | Covered in field_to_block unit suite |
| AC-011 | Shape exclusion unit test | Covered in field_to_block unit suite |
| AC-012 | AltText::Unspecified sibling-site sweep — compile-time correctness | Verified by `cargo build --workspace` (0 non_exhaustive_patterns warnings) |
| AC-013 | regions.rs structural placeholder unit test | Covered in slideforge-layout unit suite |
| AC-014 | thread_media_alt_into_frames fallback unit test | Covered in slideforge-layout unit suite |
| AC-015 | validate_post_layout match arms unit test | Covered in slideforge-validate unit suite |
| AC-016 | Stale comment corrections — not behavioral | Verified by reviewer grep |
| AC-017 | build_inner pipeline wiring — structural | Verified by AC-018 E2E test (pipeline runs Stage 2b) |
| AC-020 | DOCX body→Normal routing — covered via AC-018 DOCX path and dedicated passing test | PASS |
| AC-021 | PPTX subtitle presence — covered via full suite | PASS |
| AC-022 | DOCX Heading2 routing — covered via full suite | PASS |
