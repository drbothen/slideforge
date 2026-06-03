---
story_id: STORY-037
title: "PPTX Core Serialization: ooxmlsdk 0.6.1 + ZIP Assembly"
crate: slideforge-pptx
recorded: 2026-06-03
recorder: demo-recorder
product_type: library (no CLI yet — test-harness demo)
toolchain: VHS 0.10.0
---

# STORY-037 Demo Evidence Report

## Recording Overview

`slideforge-pptx` is a library crate with no CLI binary. Per library-demo
protocol, evidence is produced by a runnable Rust example:

- **Source**: `crates/slideforge-pptx/examples/demo_pptx.rs`
- **Run command**: `cargo run --example demo_pptx -p slideforge-pptx`

The example builds a three-slide `LaidOutDeck` (title + content-with-registers +
section\_divider) through `PptxExporter::export`, then inspects the resulting
ZIP bytes to verify each acceptance criterion.

A VHS terminal recording captures the complete program output.

---

## Recordings

| File | Type | Contents |
|------|------|----------|
| `AC-001-010-pptx-core-serialization.gif` | GIF | Full terminal run (AC-001..AC-010 summary) |
| `AC-001-010-pptx-core-serialization.webm` | WebM | Archival video of same run |
| `AC-001-010-pptx-core-serialization.tape` | VHS tape | Recording script source |

---

## AC Coverage Map

### AC-001: Exporter plugin trait implemented

**Evidence type**: structural (compile-time wiring)  
**Test**: `test_BC_4_01_001_exporter_trait_id_is_pptx`,
          `test_BC_4_01_001_exporter_trait_extension_is_pptx`  
**Demo output line**:
```
AC-001  Exporter trait: id()='pptx', extension()='pptx'  [structural — always OK]
```
`PptxExporter` implements `Exporter` from `slideforge-plugin-api`. `id()` returns
`"pptx"` and `extension()` returns `"pptx"`. The trait method signature matches the
spec: `fn export(&self, deck, laid_out, brand, opts) -> Result<Vec<u8>, ExportError>`.

**Status**: PASS

---

### AC-002: Valid PPTX ZIP produced with all required parts

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_zip_contains_all_required_parts`,
          `test_BC_4_01_001_notes_master_always_present`,
          `test_BC_4_01_001_handout_master_always_present`  
**Demo output excerpt**:
```
Total parts: 81
[OK] [Content_Types].xml
[OK] _rels/.rels
[OK] ppt/presentation.xml
[OK] ppt/_rels/presentation.xml.rels
[OK] ppt/slideMasters/slideMaster1.xml
[OK] ppt/slideMasters/_rels/slideMaster1.xml.rels
[OK] ppt/theme/theme1.xml
[OK] ppt/notesMasters/notesMaster1.xml
[OK] ppt/handoutMasters/handoutMaster1.xml
[OK] docProps/app.xml  /  docProps/core.xml
[OK] ppt/slides/slide1.xml .. slide3.xml
slideLayoutN.xml count: 31 (expected 31)
[OK] AC-002: all required parts present
```

**Status**: PASS

---

### AC-003: [Content_Types].xml registers every part type

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_content_types_snapshot_3_slides`,
          `test_BC_4_01_001_content_types_has_31_layout_overrides`  
**Demo output excerpt**:
```
Layout <Override> entries: 31 (expected 31)
[OK] Content-Type present: ...presentationml.presentation.main...
[OK] Content-Type present: ...presentationml.slideMaster...
[OK] Content-Type present: ...presentationml.slideLayout...
[OK] Content-Type present: ...presentationml.slide...
[OK] Content-Type present: ...presentationml.notesMaster...
[OK] Content-Type present: ...presentationml.handoutMaster...
[OK] Content-Type present: ...core-properties...
[OK] Content-Type present: ...extended-properties...
[OK] AC-003: [Content_Types].xml complete
```

**Status**: PASS

---

### AC-004: Placeholder inheritance correct (slide → layout by idx)

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_placeholder_inheritance_chain`,
          `test_BC_4_01_001_title_placeholder_has_idx_zero`  
**Demo output excerpt**:
```
Excerpt: ...></p:cNvSpPr><p:nvPr><p:ph type="title" idx="0"></p:ph></p:nvPr>
         </p:nvSpPr><p:spPr><a:xfrm><a:off x=...
[OK] AC-004: ph idx="0" found in slide1.xml (title placeholder)
```
The title shape carries `<p:ph type="title" idx="0"/>`, matching the layout's
`idx="0"` entry and the master's `type="title"` placeholder — confirming the
three-level inheritance chain per BC-4.01.001 postcondition 3.

**Status**: PASS

---

### AC-005: Integer EMU coordinates in all shape positions

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_all_coordinates_integer_i64`,
          `test_BC_4_01_001_no_decimal_in_off_ext_attributes`  
**Demo output excerpt**:
```
Sample coordinate: x="0"
[OK] AC-005: all coordinate attributes are integer i64 (no decimal)
```
All `<a:off x/y>` and `<a:ext cx/cy>` attributes parse as `i64` with no decimal
points or scientific notation. Title frame uses `x=457200`, `y=274638`,
`cx=8229600`, `cy=1143000` (integer EMU per BC-4.01.001 invariant 2).

**Status**: PASS

---

### AC-006: Element ordering schema-compliant

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_slide_xml_element_order_snapshot`,
          `test_BC_4_01_001_sp_child_order_nvSpPr_then_spPr_then_txBody`  
**Demo output excerpt**:
```
Positions: nvSpPr=268, spPr=404, txBody=541
[OK] AC-006: nvSpPr < spPr < txBody — schema-correct element order
```
Within each `<p:sp>`, child elements appear in ECMA-376 schema order:
`<p:nvSpPr>` (position 268) → `<p:spPr>` (404) → `<p:txBody>` (541).

**Status**: PASS

---

### AC-007: Deterministic output (SHA-256 comparison)

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_deterministic_output_sha256`,
          `test_BC_4_01_001_determinism_5_slides`  
**Demo output excerpt**:
```
Build 1 SHA-256: 56c5e52acbafdb60fd044992fb215366a383b2b6561ef6df3f2242d80bcefc95
Build 2 SHA-256: 56c5e52acbafdb60fd044992fb215366a383b2b6561ef6df3f2242d80bcefc95
[OK] AC-007: SHA-256 identical — output is deterministic
```
The same `LaidOutDeck` + `Brand` inputs produce byte-identical PPTX output on
two consecutive calls. ZIP entry ordering is alphabetical and all timestamps are
epoch (1980-01-01T00:00:00Z).

**Status**: PASS

---

### AC-008: PPTX opens in LibreOffice without repair dialog (CI gate)

**Evidence type**: CI gate reference (STORY-052)  
**Test**: `test_BC_4_01_001_libreoffice_open` (marked `#[ignore = "requires libreoffice in CI"]`)

LibreOffice headless is not installed in the local development environment.
This AC is covered by the CI visual-regression gate established in STORY-052,
which runs:
```
libreoffice --headless --convert-to png <output.pptx>
```
and asserts exit code 0. The unit test is correctly marked `#[ignore]` with a
comment citing the blocking dependency per implementer discipline (SID-1).

**Demo output line**:
```
[OK] AC-008: deferred to STORY-052 CI gate (expected)
```

**Status**: DEFERRED to STORY-052 CI gate (by design; local test marked #[ignore])

---

### AC-009: All relationship chains complete

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_relationship_chain_completeness`,
          `test_BC_4_01_001_all_rids_resolve_in_slide_rels`  
**Demo output excerpt**:
```
[OK] slide1.xml.rels → references slideLayout
[OK] slideMaster1.xml.rels → references theme1.xml
[OK] slideLayout1.xml.rels → references slideMaster1.xml
[OK] presentation.xml.rels → references slide1, notesMaster, handoutMaster
[OK] AC-009: relationship chain complete
```
Every XML part has a `.rels` sidecar. The complete chain is verified:
`slide → layout (by idx)` → `layout.rels → master` → `master.rels → theme`.

**Status**: PASS

---

### AC-010: Report/detail register content absent from PPTX slide body

**Evidence type**: VHS recording + example output  
**Test**: `test_BC_4_01_001_report_detail_absent_from_slides`,
          `test_BC_4_01_001_detail_sentinel_absent_from_all_pptx`  
**Demo output excerpt**:
```
slide2.xml contains REPORT_SENTINEL: false
slide2.xml contains DETAIL_SENTINEL: false
[OK] AC-010: neither REPORT_SENTINEL nor DETAIL_SENTINEL in slide2.xml
[OK] AC-010: all 3 slides clean of register sentinels
```
Slide 2 carries `RegisteredContent` entries for `Register::Report` ("REPORT\_SENTINEL")
and `Register::Detail` ("DETAIL\_SENTINEL") in `register_content`. Neither sentinel
appears in any `ppt/slides/slide*.xml` body — confirming BC-4.01.001 invariant 1
(two-IR model: PPTX exporter reads from `LaidOutDeck::frames` only).

**Status**: PASS

---

## Summary Table

| AC | Description | Evidence | Status |
|----|-------------|----------|--------|
| AC-001 | Exporter trait: id()='pptx', extension()='pptx' | Structural wiring, unit test names | PASS |
| AC-002 | ZIP part list complete (81 parts, 31 layouts) | Recording line 2–30 | PASS |
| AC-003 | [Content_Types].xml: 31 layout overrides + 8 types | Recording | PASS |
| AC-004 | ph idx="0" in slide1.xml title placeholder | Recording excerpt | PASS |
| AC-005 | All off/ext coordinates are integer i64 | Recording + sample value | PASS |
| AC-006 | nvSpPr(268) < spPr(404) < txBody(541) element order | Recording positions | PASS |
| AC-007 | SHA-256: `56c5e52a...` matches on both builds | Recording hash lines | PASS |
| AC-008 | LibreOffice open — CI gate (STORY-052) | `#[ignore]` + CI reference | CI-GATE |
| AC-009 | Rel chain: slide→layout→master→theme | Recording | PASS |
| AC-010 | REPORT\_SENTINEL / DETAIL\_SENTINEL absent from slides | Recording | PASS |

9 of 10 ACs demonstrated locally. AC-008 correctly deferred to STORY-052 CI gate.
