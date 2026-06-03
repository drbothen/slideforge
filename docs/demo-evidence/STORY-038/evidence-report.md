# Demo Evidence Report — STORY-038

**Story:** PPTX Layout Compliance: Placeholder Inheritance + Slide IDs + Layouts  
**Story ID:** STORY-038  
**BC:** BC-4.01.005  
**Crate:** `slideforge-pptx` (library crate)  
**Recorded:** 2026-06-03  
**Convergence status:** CONVERGED — adversary 3/3 clean passes

---

## Recording Artifacts

| File | Format | Purpose |
|------|--------|---------|
| `AC-001-012-layout-compliance.gif` | GIF | PR-embeddable animation of all AC checks |
| `AC-001-012-layout-compliance.webm` | WebM | Archival-quality recording |
| `AC-001-012-layout-compliance.tape` | VHS tape source | Reproduces both recordings |
| `demo_layout_output.log` | Plain text | Full stdout capture with ANSI stripped |

**VHS command (to reproduce):**

```
cargo build --example demo_layout -p slideforge-pptx
vhs docs/demo-evidence/STORY-038/AC-001-012-layout-compliance.tape
```

**Demo source:** `crates/slideforge-pptx/examples/demo_layout.rs`  
**Build command:** `cargo build --example demo_layout -p slideforge-pptx`  
**Run command:** `./target/debug/examples/demo_layout 2>/dev/null`

---

## Demo Deck

The example builds a 4-slide `LaidOutDeck` with a synthesized `Brand`:

| Slide | `slide_type_keyword` | Layout index (0-based) | Dark? | Purpose |
|-------|---------------------|----------------------|-------|---------|
| 1 | `title` | 0 | No | AC-001, AC-002, AC-011 (title ph) |
| 2 | `content` | 1 | No | AC-006 (no clrMapOvr), AC-011 (body ph) |
| 3 | `section_divider` | 11 | YES | AC-006 (clrMapOvr present), AC-010 |
| 4 | `blank` | 6 | No | AC-010 (layout mapping correctness) |

---

## Acceptance Criteria Coverage

### AC-001 — Slide IDs start at exactly 256, increment by 1

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — parses `presentation.xml` from the exported ZIP, extracts all `<p:sldId id="...">` attributes, and asserts minimum is `256` (= `SLIDE_ID_START`) and all are sequential.  
**Example output:**
```
Parsed slide IDs from presentation.xml: [256, 257, 258, 259]
[PASS] AC-001: first slide ID = 256 (= SLIDE_ID_START = 256)
[PASS] AC-001: IDs are sequential with step 1 (count=4)
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac001_slide_ids_exact_sequence_3_slides`  
**Constant:** `slideforge_pptx::slide_ids::SLIDE_ID_START = 256`

---

### AC-002 — Master ID is exactly 2147483648 (2^31)

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — checks `presentation.xml` contains `id="2147483648"` in the `<p:sldMasterId>` element.  
**Example output:**
```
[PASS] AC-002: <p:sldMasterId id="2147483648"> present (= 2^31)
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac002_master_id_exact_2147483648`  
**Constant:** `slideforge_pptx::slide_ids::MASTER_ID = 2_147_483_648`

---

### AC-003 — Exactly 31 slideLayout parts in the PPTX ZIP

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — counts ZIP entries matching `ppt/slideLayouts/slideLayoutN.xml` (not `_rels`), asserts count == 31.  
**Example output:**
```
slideLayout XML parts: 31 (expected 31)
[PASS] AC-003: exactly 31 slideLayout parts
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac003_zip_contains_exactly_31_layout_parts`,  
`tests/layout_tests.rs::test_BC_4_01_005_ac003_ec001_one_slide_deck_still_has_31_layouts`

---

### AC-004 — slideMaster `<p:sldLayoutIdLst>` has 31 entries

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — parses `slideMaster1.xml` from ZIP and counts `<p:sldLayoutId>` occurrences.  
**Example output:**
```
<p:sldLayoutId> entries in slideMaster1.xml: 31 (expected 31)
[PASS] AC-004: <p:sldLayoutIdLst> has 31 entries
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac004_slide_master_sldlayoutidlst_has_31_entries`

---

### AC-005 — `[Content_Types].xml` has exactly 31 layout Override entries

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — counts occurrences of `presentationml.slideLayout+xml` content-type string in `[Content_Types].xml`.  
**Example output:**
```
slideLayout Override count in [Content_Types].xml: 31 (expected 31)
[PASS] AC-005: [Content_Types].xml has 31 layout Override entries
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac005_content_types_has_exactly_31_layout_overrides`

---

### AC-006 — Dark layout slide has `<p:clrMapOvr>`; light layout slide does not

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — reads `slide1.xml` (title, light) and `slide3.xml` (section_divider, dark) from the exported ZIP; asserts clrMapOvr absent from slide1 and present in slide3.  
**Example output:**
```
[PASS] AC-006: slide1 (title, light) does NOT have <p:clrMapOvr>
Excerpt from slide3.xml: ...d><p:clrMapOvr><a:masterClrMapping /></p:clrMapOvr></p:sld>...
[PASS] AC-006: slide3 (section_divider, dark) HAS <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
```
**Unit tests:**  
- `tests/layout_tests.rs::test_BC_4_01_005_ac006_section_divider_slide_has_clrmapovr_in_zip`  
- `tests/layout_tests.rs::test_BC_4_01_005_ac006_title_slide_does_not_have_clrmapovr`  
- `tests/layout_tests.rs::test_BC_4_01_005_ac006_end_slide_has_clrmapovr_in_zip`  
**Module:** `crates/slideforge-pptx/src/clrmapovr.rs` (`ClrMapOvrInjector::needs_clr_map_ovr`)

---

### AC-007 — 257-slide deck: all slide IDs unique and >= 256

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — calls `SlideIdAssigner::assign(257)` directly, verifies the result equals `(256..=512).collect::<Vec<u32>>()` and all IDs are unique.  
**Example output:**
```
257-slide ID range: 256..=512 (expected 256..=512)
All IDs unique: true
[PASS] AC-007: 257-slide deck IDs = 256..=512, all unique, all >= 256
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac007_ec003_257_slides_unique_ids_256_to_512`  
**Note:** Full 257-slide export is exercised via `SlideIdAssigner` in isolation (avoids 257×ZIP build overhead in demo).

---

### AC-008 — slideMaster `.rels` has 31 layout relationships

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — counts occurrences of `slideLayouts/slideLayout` in `ppt/slideMasters/_rels/slideMaster1.xml.rels` (each Relationship entry contains this substring exactly once in its Target path).  
**Example output:**
```
Layout Relationship entries in slideMaster1.xml.rels: 31 (expected 31)
[PASS] AC-008: slideMaster rels has 31 layout Relationship entries
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac008_master_rels_has_31_layout_relationships`

---

### AC-009 — Unmapped Q2 DSL keywords → index 1, not index 0

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — lists all 9 unmapped Q2 keywords with their confirmed fallback assignment to index 1 (Title and Content), and cross-references the unit test suite.  
**The 9 keywords:** `split_contrast`, `card_rows`, `horizontal_timeline`, `status`, `progress_bar`, `metric_tree`, `formula`, `weighted_composite`, `grid`  
**Unit tests (9 tests):** `tests/layout_tests.rs::test_BC_4_01_005_ac009_*_maps_to_index_1_not_0`  
**Rule:** ADR-015 §A.4 no-silent-fallback; index 0 (Title Slide) is NOT the generic fallback.

---

### AC-010 — Per-slide-type layout selection correctness

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — prints the canonical keyword → 0-based index mapping table for 10 representative keywords (full suite in unit tests).  
**Example output:**
```
title                → index  0  (Title Slide)
content              → index  1  (Title and Content)
two_column           → index  3  (Two Objects)
table                → index  7  (Object with Caption)
blank                → index  6  (Blank)
section_divider      → index 11  (SF Section Divider (dark))
end                  → index 21  (SF End Slide (dark))
stat_callout         → index 12  (SF Stat Grid)
quote                → index 13  (SF Quote)
chart                → index 24  (SF Chart)
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac010_*` (~20 tests covering all 25 mapped keywords)  
**Source:** `crates/slideforge-pptx/src/lib.rs::find_layout_index` (two-phase lookup, ADR-015 §A.2)

---

### AC-011 — Placeholder inheritance idx chain verified during serialization

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — reads `slide1.xml` and `slide2.xml` from the exported ZIP and checks:
- `slide1.xml` contains `<p:ph type="title">` (Title frame → idx chain wired)
- `slide2.xml` contains `<p:ph type="body">` (Body frame → idx chain wired)

**Example output:**
```
slide1.xml placeholder excerpt: ...<p:ph type="title" idx="0"></p:ph></p:nvPr>...
[PASS] AC-011: title frame emits <p:ph type="title"> (idx chain present)
[PASS] AC-011: body frame on content slide emits <p:ph type="body">
[NOTE] AC-011: missing-idx warn path tested in tests/layout_tests.rs (test_BC_4_01_005_ac011_*)
```
**Unit tests:**  
- `tests/layout_tests.rs::test_BC_4_01_005_ac011_valid_ph_idx_emits_ph_element`  
- `tests/layout_tests.rs::test_BC_4_01_005_ac011_missing_ph_idx_omits_ph_element`

---

### AC-012 / S2 — Out-of-range EMU → `PptxError::InvalidEmu` (no silent clamp)

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — builds a `LaidOutDeck` with `page_size.width = Emu(i64::MAX)` (far beyond `i32::MAX`) and asserts the exporter returns an error rather than silently clamping.  
**Example output:**
```
Out-of-range EMU export error: export rendering error: invalid EMU coordinate in slide 0,
  frame 0: slide size width EMU 9223372036854775807 exceeds i32::MAX (2147483647);
  cannot represent as OOXML cx attribute
[PASS] AC-012: out-of-range EMU page size returns error (not silent clamp)
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_ac012_slide_size_out_of_range_emu_returns_err`  
**Source:** `crates/slideforge-pptx/src/presentation.rs` — `i32::try_from(emu).map_err(|_| PptxError::InvalidEmu {...})`

---

## PR-52 Follow-Up Coverage

### S1 — `validate_emu` Err path (negative-width/height)

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — builds a 1-slide deck with `Frame.bbox.width = Emu(-1)` (negative), asserts the exporter returns an error with a message describing the negative dimension.  
**Example output:**
```
Negative-width EMU error: export rendering error: invalid EMU coordinate in slide 0,
  frame 0: negative size: width=-1 height=1143000
[PASS] S1: negative-width frame returns export error (validate_emu Err path)
```
**Unit tests:**  
- `tests/layout_tests.rs::test_BC_4_01_005_s1_validate_emu_negative_width_returns_err`  
- `tests/layout_tests.rs::test_BC_4_01_005_s1_validate_emu_negative_height_returns_err`

---

### S3 — Subtitle frame emits `type="subTitle"` `idx=1`

**Status:** PASS  
**Demonstrated by:** `demo_layout` example — builds a dedicated 1-slide deck with a `FrameContent::Subtitle` frame, exports it, and asserts `slide1.xml` contains `type="subTitle"` and `idx="1"` in the `<p:ph>` element.  
**Example output:**
```
Subtitle slide ph excerpt: ...<p:ph type="subTitle" idx="1"></p:ph></p:nvPr>...
[PASS] S3: Subtitle frame emits type="subTitle" in <p:ph>
[PASS] S3: Subtitle frame emits idx="1" in <p:ph>
```
**Unit tests:** `tests/layout_tests.rs::test_BC_4_01_005_s3_subtitle_frame_emits_subtitle_placeholder`  
**Source:** `crates/slideforge-pptx/src/slide_serializer.rs` — `FrameContent::Subtitle` match arm

---

## LibreOffice Open-Without-Repair (Deferred to CI Gate)

The "open-without-repair" test (verifying the generated PPTX opens cleanly in LibreOffice) requires LibreOffice headless, which is not available in local demo environments.

This coverage is provided by:
- **STORY-052** CI visual-regression gate: `libreoffice --headless --convert-to png <file>` on every PR
- Test marker: tests tagged `#[ignore = "requires libreoffice in CI"]` in `tests/layout_tests.rs`

This is the documented deferral per the story spec ("AC requiring LibreOffice → note it's covered by the CI visual-regression gate (STORY-052)").

---

## Summary Table

| AC / Item | Status | Artifact | Unit Test(s) |
|-----------|--------|----------|-------------|
| AC-001 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac001_*` |
| AC-002 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac002_*` |
| AC-003 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac003_*` |
| AC-004 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac004_*` |
| AC-005 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac005_*` |
| AC-006 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac006_*` (3 tests) |
| AC-007 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac007_*` |
| AC-008 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac008_*` |
| AC-009 | PASS | `demo_layout` output + unit tests | `test_BC_4_01_005_ac009_*` (9 tests) |
| AC-010 | PASS | `demo_layout` output + unit tests | `test_BC_4_01_005_ac010_*` (~20 tests) |
| AC-011 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac011_*` (2 tests) |
| AC-012 / S2 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_ac012_*` |
| S1 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_s1_*` (2 tests) |
| S3 | PASS | `demo_layout` output + GIF/WebM | `test_BC_4_01_005_s3_*` |
| LibreOffice | DEFERRED | STORY-052 CI gate | `#[ignore]` test in layout_tests.rs |

**All 12 ACs and both follow-up items (S1, S3) are covered.**  
**Exit code 0 confirmed:** `cargo run --example demo_layout -p slideforge-pptx 2>/dev/null; echo $?` → `0`
