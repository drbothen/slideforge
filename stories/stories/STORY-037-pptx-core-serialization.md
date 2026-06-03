---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-037
title: "PPTX Core Serialization: ooxmlsdk 0.6.1 + ZIP Assembly"
epic: EPIC-08
wave: 4
points: 13
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.01.001]
verification_properties: [VP-013]
nfr_refs: [NFR-005, NFR-007, NFR-008, NFR-009, NFR-011, NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
crate: slideforge-pptx
target_module: slideforge-pptx
subsystems: [SS-06]
depends_on:
  - STORY-023
  - STORY-026
  - STORY-034
  - STORY-035
  - STORY-036
blocks:
  - STORY-038
  - STORY-039
  - STORY-040
  - STORY-049
  - STORY-050
estimated_days: 5
---

# STORY-037: PPTX Core Serialization: ooxmlsdk 0.6.1 + ZIP Assembly

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns this story's entire scope per the ARCH-INDEX Subsystem Registry.
`slideforge-pptx` is the primary crate for SS-06. This story is the foundation of the
PPTX exporter: without it, no PPTX can be produced and no downstream PPTX stories
(038, 039, 040) can proceed.

## Dependency Anchor Justifications

- Depends on STORY-023 (Brand Synthesis): The PPTX exporter requires a `Brand` struct
  with a complete 31-layout hierarchy. The brand provides theme colors, fonts, master XML,
  and layout XMLs that are embedded verbatim in the PPTX ZIP.
- Depends on STORY-026 (Layout Core): The `LaidOutDeck` IR is this story's primary input.
  Without a complete `LaidOutDeck`, no serialization is possible.
- Depends on STORY-034 (SVG Normalization): Chart and diagram SVGs embedded in PPTX must
  be `NormalizedDiagramSvg` (no foreignObject, absolute dims). STORY-034's output type
  is required here.
- Depends on STORY-035 (Register Routing): `LaidOutSlide.register_content` must exist.
  Notes content is extracted from it for `notesSlide` parts (STORY-040 will add notes,
  but the routing must exist now so the PPTX exporter ignores report/detail correctly).
- Depends on STORY-036 (No Bleed Invariant): The `BleedChecker` utility is used in this
  story's tests to un-ignore the bleed invariant tests for PPTX.
- Blocks STORY-038/039/040: They extend the PPTX output built here.
- Blocks STORY-049/050: Plugin registry assembly and E2E tests depend on a working PPTX exporter.

## Summary

Implement the `slideforge-pptx` crate as an `Exporter` plugin. This is the largest
story in the project at 13 points because it establishes the full PPTX ZIP structure,
all required parts, the ooxmlsdk 0.6.1 element construction pipeline, and complete
placeholder inheritance. Subsequent PPTX stories (038, 039, 040) are extensions, not
foundations.

### What This Story Delivers

1. Complete PPTX ZIP archive with all required parts
2. Correct `[Content_Types].xml` for every part type
3. All relationship chains (presentation.xml.rels, slide.xml.rels, etc.)
4. Slide serialization: title/content/speaker shapes from `LaidOutDeck`
5. Placeholder inheritance: slide → layout (by `idx`) → master (by `type`)
6. EMU coordinate serialization (integer only, no `f64`)
7. Deterministic output (same input → byte-identical output)
8. `Exporter` plugin trait implementation

### PPTX ZIP Structure

The produced ZIP must contain exactly these parts (minimum for a valid PPTX):

```
[Content_Types].xml
_rels/.rels
ppt/presentation.xml
ppt/_rels/presentation.xml.rels
ppt/slideLayouts/slideLayout1.xml        (x31, for each of 31 layouts)
ppt/slideLayouts/_rels/slideLayout1.xml.rels
ppt/slideMasters/slideMaster1.xml
ppt/slideMasters/_rels/slideMaster1.xml.rels
ppt/slides/slide1.xml                    (one per LaidOutSlide)
ppt/slides/_rels/slide1.xml.rels
ppt/theme/theme1.xml
ppt/notesMasters/notesMaster1.xml        (required even if empty — BC-4.01.006)
ppt/notesMasters/_rels/notesMaster1.xml.rels
ppt/handoutMasters/handoutMaster1.xml    (required even if empty — BC-4.01.006)
ppt/handoutMasters/_rels/handoutMaster1.xml.rels
ppt/media/image1.png                     (chart/diagram SVG or rasterized media, if any)
docProps/app.xml
docProps/core.xml
```

### ooxmlsdk 0.6.1 API Usage

Use `ooxmlsdk` for all OOXML element construction. Do NOT build XML by string
concatenation. The ooxmlsdk crate organizes OOXML types under the `schemas`
module with namespace-derived paths. Use type aliases for ergonomics:

```rust
// The actual module paths are namespace-derived. Use aliases:
use ooxmlsdk::schemas::schemas_openxmlformats_org_presentationml_2006_main as pml;
use ooxmlsdk::schemas::schemas_openxmlformats_org_drawingml_2006_main as dml;

// Key PresentationML types (accessed via pml::):
// - Presentation, SlideIdList, SlideIdListEntry
// - Slide, CommonSlideData (cSld), ShapeTree
// - Shape, NonVisualShapeProperties, NonVisualDrawingProperties (cNvPr)
// - PlaceholderShape (ph), ShapeProperties (spPr), TextBody (txBody)

// Key DrawingML types (accessed via dml::):
// - Transform2D (xfrm), Offset (off), Extents (ext)
// - Paragraph, Run, RunProperties
// - SolidFill, SchemeColor

// Parts module for reading/writing package parts:
use ooxmlsdk::parts;
```

**IMPORTANT:** The exact struct/enum names in ooxmlsdk 0.6.1 are generated from
OOXML schema metadata and use CT_* naming conventions (e.g., `CT_Shape`,
`CT_Presentation`). Consult `docs.rs/ooxmlsdk/0.6.1` for exact type names.
The serializers module provides XML serialization for each type.

All position/size values are integer EMUs. No `f64`. Use `i64` for EMU values.

### Element Ordering (Schema-Significant)

OOXML element ordering is schema-significant. Child elements must appear in ECMA-376
schema order. Violations cause "repair required" in some renderers. Critical orderings:

**`<p:sp>` (Shape) element order:**
```xml
<p:sp>
  <p:nvSpPr>          <!-- 1. Non-visual shape properties -->
    <p:cNvPr .../>    <!-- id, name, descr (alt text) -->
    <p:cNvSpPr/>
    <p:nvPr>
      <p:ph type="..." idx="..."/>   <!-- placeholder type/idx -->
    </p:nvPr>
  </p:nvSpPr>
  <p:spPr>            <!-- 2. Shape properties -->
    <a:xfrm>          <!-- position/size (EMU) -->
      <a:off x="..." y="..."/>
      <a:ext cx="..." cy="..."/>
    </a:xfrm>
    <a:prstGeom prst="rect"/>
  </p:spPr>
  <p:txBody>          <!-- 3. Text body (LAST) -->
    <a:bodyPr/>
    <a:lstStyle/>
    <a:p>
      <a:r><a:t>text</a:t></a:r>
    </a:p>
  </p:txBody>
</p:sp>
```

**`<p:sld>` (Slide) element order:**
```xml
<p:sld>
  <p:cSld>            <!-- 1. Common slide data -->
    <p:spTree>        <!-- Shape tree (all shapes) -->
      <p:grpSpPr/>
      <p:sp> ... </p:sp>   <!-- shapes in order -->
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr>       <!-- 2. Color map override (for dark layouts) -->
    <a:masterClrMapping/>
  </p:clrMapOvr>
  <p:transition/>     <!-- 3. Transition (omit if none) -->
  <p:timing/>         <!-- 4. Timing (omit if none) -->
</p:sld>
```

### Placeholder Inheritance

Placeholder inheritance is the most complex aspect of PPTX. Follow this protocol exactly:

1. **Slide → Layout** matching: by `idx` attribute on `<p:ph>`.
   - `idx=0` → title placeholder
   - `idx=1` → content/body placeholder
   - `idx=11` → date/time
   - `idx=12` → footer
   - `idx=13` → slide number
2. **Layout → Master** matching: by `type` attribute on `<p:ph>`.
   - `type="title"` → master title placeholder
   - `type="body"` → master body placeholder
   - `type="dt"`, `type="ftr"`, `type="sldNum"` → master utility placeholders
3. A slide shape with `<p:ph idx="0">` inherits geometry from the layout's `<p:ph idx="0">`.
   The slide may override position/size; if the slide's `<p:spPr>` has no `<a:xfrm>`,
   the layout's geometry is used.

### [Content_Types].xml

Every part type must be registered. Required content types for the minimum PPTX:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <!-- ... one per layout (31 total) ... -->
  <Override PartName="/ppt/slides/slide1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <!-- ... one per slide ... -->
  <Override PartName="/ppt/theme/theme1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/ppt/notesMasters/notesMaster1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"/>
  <Override PartName="/ppt/handoutMasters/handoutMaster1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml"/>
  <Override PartName="/docProps/core.xml"
    ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml"
    ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>
```

### Determinism

Produce byte-identical ZIP output for the same input (excluding ZIP timestamps). This
requires:
1. Deterministic entry ordering in the ZIP (alphabetical by part name)
2. No `SystemTime::now()` in the ZIP metadata (use epoch 1980-01-01 00:00:00 for all entries)
3. No HashMap iteration order dependence — use `BTreeMap` for any map-to-XML serialization

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.001 | Serialize LaidOutDeck to valid .pptx with correct placeholder inheritance | AC-001 through AC-010 |

## Acceptance Criteria

### AC-001: Exporter plugin trait implemented
(traces to BC-4.01.001 precondition 4 — LaidOutDeck IR types implement Hash+Eq+Clone)

`slideforge-pptx` implements `Exporter` via the plugin trait API from `slideforge-plugin-api`.
The `PptxExporter` struct is registered via the plugin registry. No bypass of the trait
API. Method signature:
```rust
fn export(&self, deck: &LaidOutDeck, brand: &Brand, output: &mut dyn Write) -> Result<(), ExportError>;
```

### AC-002: Valid PPTX ZIP produced with all required parts
(traces to BC-4.01.001 postcondition 2 — passes OOXML schema validation)

A 1-slide deck with title "Hello World" produces a PPTX ZIP that contains all parts
listed in the "PPTX ZIP Structure" section above. Every listed part is present. A unit
test enumerates ZIP entries and asserts the presence of each required path.

### AC-003: [Content_Types].xml registers every part type
(traces to BC-4.01.001 postcondition 7 — Content_Types registers every part)

The produced `[Content_Types].xml` has `<Override>` entries for every `ppt/slides/slide*.xml`,
every `ppt/slideLayouts/slideLayout*.xml`, `ppt/slideMasters/slideMaster1.xml`,
`ppt/theme/theme1.xml`, `ppt/notesMasters/notesMaster1.xml`,
`ppt/handoutMasters/handoutMaster1.xml`. A snapshot test captures the full Content_Types
XML for a 3-slide deck and is used for regression detection.

### AC-004: Placeholder inheritance correct (slide → layout by idx, layout → master by type)
(traces to BC-4.01.001 postcondition 3 — placeholder inheritance is correct)

A slide with a `title` field produces a `<p:sp>` with `<p:ph idx="0">`. The corresponding
layout has a `<p:ph idx="0">` that matches. The master has a `<p:ph type="title">` that the
layout's `<p:ph type="title">` inherits from. A unit test parses the slide XML and verifies
the idx/type chain is consistent.

### AC-005: Integer EMU coordinates in all shape positions
(traces to BC-4.01.001 precondition 5 — all coordinates integer EMUs)

All `<a:off x="..." y="...">` and `<a:ext cx="..." cy="...">` attributes contain integer
values (no decimal points, no scientific notation). A test parses all shape XML in a
3-slide PPTX and asserts all coordinate attribute values parse as valid `i64` without
remainder.

### AC-006: Element ordering schema-compliant
(traces to BC-4.01.001 postcondition 2 — passes OOXML schema validation)

The ooxmlsdk serialization naturally produces schema-correct element ordering. A snapshot
test captures the full `slide1.xml` for a title slide and confirms the child order:
`<p:nvSpPr>` → `<p:spPr>` → `<p:txBody>` for each `<p:sp>`.

### AC-007: Deterministic output
(traces to BC-4.01.001 invariant 4 — same input → byte-identical output)

Building the same `LaidOutDeck` + `Brand` twice produces byte-identical PPTX output.
A test builds a 5-slide deck twice, computes SHA-256 of each output, and asserts equality.
ZIP timestamps must be set to epoch (1980-01-01 00:00:00) for all entries.

### AC-008: PPTX opens in LibreOffice without repair dialog (CI gate)
(traces to BC-4.01.001 postcondition 4 — openable in LibreOffice without error)

The CI visual regression job (STORY-052) runs `libreoffice --headless --convert-to png`
on the produced PPTX and asserts exit code 0. A non-zero exit code means LibreOffice
could not open the file. This test is `#[ignore = "requires libreoffice in CI"]` in
unit tests but is a required CI gate.

### AC-009: All relationship chains complete
(traces to BC-4.01.001 postcondition 7 — complete relationship references)

Every XML part has a corresponding `.rels` file. Every `r:id` reference in any XML file
resolves to an entry in the corresponding `.rels` file. A unit test verifies: for each
slide, `ppt/slides/_rels/slide1.xml.rels` contains the layout relationship; the layout's
`.rels` file contains the master relationship; the master's `.rels` file contains the
theme relationship.

### AC-010: Report/detail register content absent from PPTX slide body
(traces to BC-4.01.001 invariant 1 — PPTX exporter reads from LaidOutDeck only)

Un-ignore AC-001 and AC-002 from STORY-036's `bleed_tests.rs`. A deck with
`report "REPORT_SENTINEL"` and `detail "DETAIL_SENTINEL"` produces a PPTX where neither
sentinel appears in any `ppt/slides/slide*.xml`. Use `BleedChecker::assert_absent_from_pptx_slides`.

## Tasks

- [ ] **Task 1: Crate scaffold and Exporter trait impl**
  - Create `crates/slideforge-pptx/` with `Cargo.toml` (deps: ooxmlsdk=0.6.1, zip=4.2.0, slideforge-types, slideforge-plugin-api)
  - Define `PptxExporter` struct in `src/lib.rs`
  - Implement `Exporter` trait: `fn export(&self, deck, brand, output) -> Result<(), ExportError>`

- [ ] **Task 2: ZIP assembler**
  - Implement `ZipAssembler` in `src/zip_assembler.rs`
  - Write parts to in-memory `zip::ZipWriter<Cursor<Vec<u8>>>` (zip 4.x API)
  - Use epoch timestamps for all ZIP entries (zip 4.x: use `DateTime::default()` or explicit 1980-01-01)
  - Sort entries alphabetically for determinism

- [ ] **Task 3: `[Content_Types].xml` generator**
  - Implement `ContentTypesBuilder` in `src/content_types.rs`
  - Build `[Content_Types].xml` with all required Default and Override entries
  - Entries computed from the set of parts actually written

- [ ] **Task 4: Relationship file generator**
  - Implement `RelsBuilder` in `src/rels.rs`
  - Generate `_rels/.rels`, `ppt/_rels/presentation.xml.rels`, per-slide `.rels`, per-layout `.rels`
  - All `r:id` values use sequential `rId1`, `rId2`, ... per `.rels` file

- [ ] **Task 5: Presentation.xml + slide list**
  - Implement `PresentationSerializer` in `src/presentation.rs` using ooxmlsdk
  - Generate `<p:sldIdLst>` with slide IDs starting at 256 (BC-4.01.005)
  - Generate `<p:sldMasterIdLst>` with master ID = 2^31 (2147483648)
  - Include `<p:sectionLst>` if sections present (deferred to STORY-040; stub empty list)

- [ ] **Task 6: Slide serializer**
  - Implement `SlideSerializer` in `src/slide_serializer.rs` using ooxmlsdk
  - For each `LaidOutSlide`, generate `<p:sld>` XML
  - Map `FrameContent` variants to PPTX shapes: title→`<p:ph idx="0">`, content→`<p:ph idx="1">`
  - EMU coordinates from `LaidOutFrame.position` and `.size`
  - Inline text: map `InlineSpan` variants to `<a:r>` + `<a:rPr>` (bold, italic, code)

- [ ] **Task 7: Master + layout + theme embedding (per ADR-015)**
  - Generate 31 layout XML parts by calling
    `slideforge_brand::layout_xml::serialize_layout_to_xml(&template.layouts[i])`
    for each `i` in `0..template.layouts.len()` (guaranteed 31 entries for synthesized brands).
    Write results to `ppt/slideLayouts/slideLayout{N}.xml` for N in 1..=31.
  - Generate master XML by calling `slideforge_brand::layout_xml::serialize_master_to_xml(template)`.
    Write result to `ppt/slideMasters/slideMaster1.xml`.
  - Generate theme XML by calling `slideforge_brand::layout_xml::serialize_theme_to_xml(template)`.
    Write result to `ppt/theme/theme1.xml`.
  - Write notes/handout master stubs from `template.notes_master_stub` and
    `template.handout_master_stub` to `ppt/notesMasters/notesMaster1.xml` and
    `ppt/handoutMasters/handoutMaster1.xml` respectively.
  - Generate `.rels` for master and each layout.
  - Do NOT use `SlideMaster::default()` / `SlideLayout::default()` shells or hardcoded theme XML.
  - Cite: ADR-015 §1 (layout API), §2 (master API), §3 (theme API), §4 (stubs).

- [ ] **Task 8: notesMaster + handoutMaster (empty stubs)**
  - Write minimal valid `notesMaster1.xml` and `handoutMaster1.xml`
  - Register both in `[Content_Types].xml`
  - Add to `presentation.xml.rels`
  - Full notes content is added in STORY-040; these stubs satisfy BC-4.01.006

- [ ] **Task 9: Write unit tests**
  - AC-002: ZIP contains all required parts (enumerate ZIP entries)
  - AC-003: `[Content_Types].xml` snapshot test (3-slide deck)
  - AC-004: Placeholder inheritance chain verified
  - AC-005: All coordinates are integer `i64` (no `.` in attribute values)
  - AC-006: `slide1.xml` snapshot test (element ordering)
  - AC-007: Determinism test (build twice, assert SHA-256 equality)
  - AC-009: Relationship chain completeness test
  - AC-010: Un-ignore STORY-036 bleed tests for PPTX (report/detail absent from slides)

- [ ] **Task 10: docProps/core.xml and docProps/app.xml**
  - `core.xml`: minimal required fields (dc:creator="slideforge", dc:language from deck lang, dcterms:created epoch)
  - `app.xml`: minimal valid (`<AppVersion>`, `<Company>`)
  - Note: full `dc:language` embedding for BC-4.01.004 / BC-5.01.005 is in STORY-039

## Previous Story Intelligence

N/A — first story in EPIC-08. This is the foundational PPTX story. All prior art:

- The `LaidOutDeck` structure from STORY-026 provides the input IR.
- The `Brand` struct from STORY-023 provides master/layout/theme XML. The PPTX exporter
  embeds brand XML verbatim — it does NOT re-parse or re-generate it.
- The `NormalizedDiagramSvg` type from STORY-034 is the only SVG type accepted for
  diagram embedding.
- The `Register` and `RegisteredContent` types from STORY-035 are present on `LaidOutSlide`.
  This story only excludes report/detail from slide body — it does NOT add notes to
  notesSlide (that is STORY-040).

Spike S1 (ooxmlsdk PPTX coverage) confirmed ooxmlsdk 0.6.1 covers all required element
types. Spike S6 (multi-renderer parity) identified BUG-002 (missing notesMaster causes
repair dialog) and BUG-006 (slide ID < 256 causes corruption in some renderers). Both
are addressed here.

## Architecture Compliance Rules

1. **ooxmlsdk for all OOXML construction (ADR-001)**: No XML string concatenation.
   Every OOXML element is constructed via ooxmlsdk typed builders.
2. **Integer EMU coordinates only (BC-4.01.001 invariant 2, DI-010)**: No `f64` in the
   PPTX output path. All `<a:off>`, `<a:ext>` values are `i64`.
3. **Two-IR model (BC-4.01.001 invariant 1, DI-009)**: PPTX exporter reads from
   `LaidOutDeck` only — it does not modify `Deck` or call the evaluator.
4. **Element ordering schema-compliant (R4 finding)**: `<p:nvSpPr>` → `<p:spPr>` →
   `<p:txBody>` within `<p:sp>`. ooxmlsdk enforces this automatically.
5. **Placeholder inheritance by idx/type (BC-4.01.001 postcondition 3)**: Slide
   placeholders use `idx` matching to layout; layout uses `type` matching to master.
6. **notesMaster1.xml always present (BC-4.01.006 invariant 1)**: No conditional logic
   omits these files. They are always written.
7. **clrMapOvr for dark layouts (brand-architecture.md §Dark Layout Invariant)**: Slides
   using dark-themed layouts include `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>`.
   The `clrMapOvr` element must appear after `<p:cSld>` and before `<p:transition>`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | All OOXML element construction (ADR-001) |
| `zip` | `=4.2.0` | ZIP archive assembly (must be compatible with ooxmlsdk's zip dep) |
| `slideforge-types` | workspace | `Register`, `InlineSpan`, and shared types |
| `slideforge-plugin-api` | workspace | `Exporter` trait |
| `slideforge-brand` | workspace | `BrandTemplate`, `serialize_layout_to_xml`, `serialize_master_to_xml`, `serialize_theme_to_xml` (per ADR-015) |
| `slideforge-layout` | workspace (IR types only) | `LaidOutDeck`, `LaidOutSlide`, `LaidOutFrame` — no layout-computation calls (per ADR-015 §6) |
| `thiserror` | `=2.0.18` | `ExportError` enum |
| `tracing` | `=0.1.43` | Structured logging of serialization stages |
| `insta` | `=1.42.0` | Snapshot tests for XML output |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/Cargo.toml` | Create | Crate manifest with pinned deps |
| `crates/slideforge-pptx/src/lib.rs` | Create | `PptxExporter`, `Exporter` impl |
| `crates/slideforge-pptx/src/zip_assembler.rs` | Create | ZIP writer with deterministic ordering |
| `crates/slideforge-pptx/src/content_types.rs` | Create | `[Content_Types].xml` generation |
| `crates/slideforge-pptx/src/rels.rs` | Create | Relationship file generation |
| `crates/slideforge-pptx/src/presentation.rs` | Create | `presentation.xml` serialization |
| `crates/slideforge-pptx/src/slide_serializer.rs` | Create | Per-slide `slide*.xml` serialization |
| `crates/slideforge-pptx/src/error.rs` | Create | `ExportError` enum using `thiserror` |
| `crates/slideforge-pptx/src/tests/core_tests.rs` | Create | AC unit tests |
| `Cargo.toml` (workspace root) | Modify | Add `slideforge-pptx` to workspace members |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~5,000 |
| BC-4.01.001 | ~2,000 |
| STORY-026 LaidOutDeck types | ~2,000 |
| STORY-023 Brand struct reference | ~1,500 |
| ooxmlsdk 0.6.1 API reference | ~3,000 |
| zip 4.2.0 API | ~500 |
| Files to create (10 files) | ~8,000 |
| **Total** | **~22,000** |

At 22,000 tokens this is approximately 10-12% of a 200k-token context window — within
the 20-30% guideline. The token estimate is higher than average because this story creates
many new files.

## Test Strategy

- **Snapshot tests (most critical)**: `slide1.xml`, `presentation.xml`,
  `[Content_Types].xml` for a 3-slide deck. Snapshots are reviewed for correct structure
  and OOXML compliance before commit.
- **ZIP structure test**: Enumerate ZIP entries; assert all required paths present.
- **Relationship chain test**: Verify every `r:id` in presentation.xml resolves in
  presentation.xml.rels; every slide `r:id` resolves in its `.rels` file.
- **Determinism test**: Build twice, SHA-256 compare.
- **Bleed invariant test**: Un-ignore STORY-036 AC-001/AC-002 for PPTX.
- **LibreOffice CI test**: `#[ignore]` in unit tests; required CI gate in STORY-052.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no visual elements (only register content) | Valid empty slide body; notes/report/detail routed correctly |
| EC-002 | Deck with 1 slide | Single slide in ZIP; all 31 layouts still present (BC-4.01.005 invariant 3) |
| EC-003 | Chart SVG embedded | `ppt/media/image1.svg` added; relationship from slide; `[Content_Types].xml` has SVG Override |
| EC-004 | Output directory does not exist | Directory created recursively; output written |
| EC-005 | Brand has dark-themed layout | `<p:clrMapOvr>` present in slides using that layout |
| EC-006 | Build same LaidOutDeck twice | Byte-identical output; ZIP timestamps epoch |

## Forbidden Dependencies

`slideforge-pptx` must NOT depend on:
- `slideforge-eval` — evaluator is upstream; no circular dependency
- `slideforge-syntax` — parser is upstream
- `slideforge-docx` — sibling exporter; no cross-exporter deps
- `slideforge-pdf` — sibling exporter
- `slideforge-html` — sibling exporter
- Any crate not in the approved dependency list in `architecture/dependency-graph.md`

**Amended per ADR-015:**
- `slideforge-pptx` must NOT call `slideforge-layout` layout-computation functions
  (e.g., `layout::run`), but MAY depend on `slideforge-layout` for `LaidOutDeck`,
  `LaidOutSlide`, and associated IR types (per ADR-015 §6).

**Approved dependencies (per ADR-015):**
- `slideforge-brand` — APPROVED; `slideforge-pptx` MUST call
  `slideforge_brand::layout_xml::serialize_layout_to_xml`,
  `serialize_master_to_xml`, and `serialize_theme_to_xml` (per ADR-015 §1–3 and §5).

**ooxmlsdk 0.6.1 verification (as of 2026-05-25):** Confirmed published on crates.io
with MSRV 1.88.0 (matches project toolchain). Dependencies: `quick-xml ^0.38.0`,
`thiserror ^2.0.12`, `zip ^4.2.0`. The crate provides generated OOXML types under
`ooxmlsdk::schemas::*` with namespace-derived module names, plus `ooxmlsdk::parts::*`
for package part read/write, and `ooxmlsdk::serializers::*` / `ooxmlsdk::deserializers::*`
for XML serialization. No `[patch]` or `path` dependency needed.
