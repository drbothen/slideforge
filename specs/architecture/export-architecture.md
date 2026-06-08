---
document_type: architecture-section
section: export-architecture
version: "1.1"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
modified:
  - "2026-06-07: v1.1 — Added authoritative 'PPTX Slide Sections (sectionLst extension)' subsection.
     Specifies p14 namespace URI, p:ext URI GUID, full element nesting, GUID derivation algorithm,
     element ordering constraint, and ooxmlsdk-vs-raw-XML implementation decision.
     Reconciles spec-vs-OOXML-reality defect found at STORY-082 start (bare p:sectionLst in story
     XML snippet is wrong; correct form is p14:sectionLst under p:extLst/p:ext). Story-writer
     updates STORY-082 XML snippet from this record."
traces_to: ARCH-INDEX.md
---

# Export Architecture

## Shared Input Contract

All exporters consume the same two IRs via the `Exporter` trait:
- `Deck` — semantic information (register assignments, slide types, alt text)
- `LaidOutDeck` — geometric information (EMU coordinates, text flows, reading order)
- `Brand` — theme colors, fonts, layout templates

## PPTX Exporter (SS-06, ADR-001)

Library: `ooxmlsdk = "=0.6.1"`. Spike S1 confirmed 55/57 capability checks PASS.

Two workarounds are required (implemented in `slideforge-pptx`):

**W1 — Table serialization:** `GraphicData` has no typed `a_table` field. Tables
are serialized to XML string via `table.to_xml_bytes()` then placed in
`GraphicData.xml_children`. Encapsulated in `table_builder.rs`.

**W2 — Content_Types Default entries:** ooxmlsdk emits only `<Override>` entries.
ECMA-376 requires `<Default>` entries for `.rels` and `.xml` extensions.
Post-processed in `opc_postprocess.rs` (~50 lines): parse ZIP in-memory, inject
Default entries, repackage.

Key implementation notes (S1 API findings):
- All slide shapes, pictures, graphic frames go via `ShapeTreeChoice` enum variants
- Navigate parts via relationships — never hard-code part paths (they are auto-numbered)
- Import PML and DML types in separate `use` blocks to avoid namespace collision

Visual parity gate: SSIM ≥ 0.99 AND PSNR ≥ 35dB per slide vs LibreOffice renders
(ADR-002, S6). CI pipeline: PPTX → PDF via LibreOffice Still 25.8.7 → PNG via
ImageMagick 300 DPI → scikit-image comparison.

## PDF Exporter (SS-07, ADR-003)

Library stack: `pdf-writer = "=0.14"` + `krilla = "=0.6"` + `SlideTagEngine` (custom).

The SlideTagEngine is the critical component: it builds the `/StructTreeRoot`
required for PDF/UA-1. It MUST be designed with a decoupled draw/tag interface
(Feasibility Note 1):

**Draw layer:** krilla handles path fills, strokes, glyph placement, image/SVG
embedding. This layer is stable regardless of tagging approach.

**Tag layer (primary):** krilla's experimental tagged PDF hooks assign `/MCID`
values and link to structure tree elements. Available in krilla v0.6.0.

**Tag layer (fallback):** If krilla's experimental tagging fails PDF/UA-1 validation
after Phase 6 hardening (≤5 story-days effort), fall back to direct `pdf-writer`
structure tree construction. The draw layer (krilla) remains unchanged. The fallback
trigger: `veraPDF --flavour ua1` exits non-zero on a full test fixture deck.

**Fallback-of-fallback:** Typst template approach (generate `.typ` source from
LaidOutDeck via Typst's `place()` function). Escalates to human decision.

Coordinate mapping: krilla's `Surface` is top-left, Y-down (surface.rs:44, geom.rs:145).
krilla applies the PDF Y-axis flip internally via `page_root_transform`
(`Transform::from_row(1,0,0,-1,0,h)`, page.rs:262-263). The draw path passes IR Y
coordinates directly as `emu_to_pt(ir_y)` — no exporter-applied Y-flip.
The pure function `ir_y_to_pdf_y` (formula: `page_height_pt - (ir_y_pt + element_height_pt)`)
is retained as a documented pure function and VP-006 Kani proof target, but it is NOT
called on the krilla draw path. See BC-4.03.005 v1.2 and DIR-044-001.

Accessibility gate: `veraPDF --flavour ua1` in CI via Docker sidecar (`verapdf/rest:1.26.0`).

## DOCX Exporter (SS-08)

OOXML DOCX serialization via `ooxmlsdk`. The `report` register content (DI-012)
maps to document sections. Shares the OOXML element-ordering knowledge from PPTX.
Brand template provides the DOCX template via `BrandProvider::synthesize_docx()`.

## HTML/Preview Exporter (SS-09, ADR-008)

The web preview is an embedded `axum` server with WebSocket push. SVG-based
canvas rendering is required (not `<canvas>`) per S3 WCAG constraint:
a bare `<canvas>` is opaque to axe-core and screen readers (WCAG 1.1.1).

Slides render as SVG elements with ARIA attributes. The Node.js footprint is
test-harness only (`@axe-core/playwright` in `crates/slideforge-preview/tests/`).
The production binary has zero Node.js dependency.

## PPTX Slide Sections (sectionLst extension)

### Authoritative Element Structure

PowerPoint slide sections are stored in the **Microsoft PowerPoint 2010 extension
namespace** (`p14`), not as a bare `p:sectionLst` in the core PresentationML namespace.
The bare form `<p:sectionLst>` does not exist in the ECMA-376 schema for
`p:presentation`; PowerPoint 365 would not recognize it as a section list. The element
is absent from ECMA-376 Part 1 core and appears only as a transitional extension per
ECMA-376 Part 4 / [MS-OE376].

**Sources:** ECMA-376 Part 4 (PresentationML transitional schema); Microsoft [MS-OE376]
implementation notes at `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/`;
python-pptx issue #257 (open-source reference implementation cross-check).

### Namespace and Extension URI

| Item | Value |
|------|-------|
| p14 namespace prefix | `p14` |
| p14 namespace URI | `http://schemas.microsoft.com/office/powerpoint/2010/main` |
| `p:ext` URI attribute (fixed GUID) | `{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}` |

The `p:ext uri` GUID is a fixed constant identifying the sectionLst extension slot.
It is NOT a per-deck generated value; it must be this exact string in every
`presentation.xml` that contains slide sections.

### Full Element Nesting

```xml
<p:presentation
    xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
    xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"
    ...>
  <!-- ... p:sldMasterIdLst, p:notesMasterIdLst, p:handoutMasterIdLst,
           p:sldIdLst, p:sldSz, p:notesSz, p:defaultTextStyle ... -->
  <p:extLst>
    <p:ext uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}">
      <p14:sectionLst
          xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">
        <p14:section name="Background" id="{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}">
          <p14:sldIdLst>
            <p14:sldId id="256"/>
            <p14:sldId id="257"/>
          </p14:sldIdLst>
        </p14:section>
        <p14:section name="Analysis" id="{YYYYYYYY-YYYY-YYYY-YYYY-YYYYYYYYYYYY}">
          <p14:sldIdLst>
            <p14:sldId id="258"/>
            <p14:sldId id="259"/>
          </p14:sldIdLst>
        </p14:section>
      </p14:sectionLst>
    </p:ext>
  </p:extLst>
</p:presentation>
```

Notes on the nesting:
- `p14:sldId/@id` values reference the integer slide IDs from `p:sldIdLst/p:sldId/@id`
  (starting at 256 per BC-4.01.005). They are the same integers — no indirection layer.
- The `xmlns:p14` declaration may appear on `p:presentation` (preferred, cleaner) or
  on `p14:sectionLst` itself (both are schema-valid). Emit it on `p:presentation` when
  using raw-XML construction so namespace resolution is unambiguous.

### Section `id` GUID Format

Each `p14:section/@id` is a section-identity GUID:
- Format: brace-wrapped, hyphen-separated, uppercase hex —
  `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (8-4-4-4-12 groups)
- ECMA-376 does not normatively mandate uppercase; Microsoft Office generates uppercase.
  slideforge MUST generate uppercase to maximize PowerPoint round-trip fidelity.
- The `id` attribute is NOT the same as `p:ext uri`. It identifies the specific section
  instance within the deck and must be unique per section.

### Deterministic GUID Derivation

Section GUIDs are derived deterministically from the section name using SHA-256:

```
raw = sha2::Sha256::digest(section_name.as_bytes())   // 32 bytes
// Truncate to 16 bytes (UUID-sized)
uuid_bytes[0..16] = raw[0..16]
// Set version nibble (byte 6, high nibble) = 5 (UUID v5 convention)
uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x50
// Set variant bits (byte 8, high 2 bits) = 10 (RFC 4122 variant)
uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80
// Format as uppercase braced GUID string:
// {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
```

Algorithm properties:
- **Deterministic:** same section name → same GUID → byte-identical builds (AC-009).
- **Reproducible across platforms:** sha2 0.10.9 is pure Rust, no OS dependency.
- **UUID v5-like:** version nibble = 5, variant = RFC 4122. Not strictly UUID v5 (no
  namespace prefix is hashed), but structurally valid as a GUID.
- **`Uuid::new_v4()` is FORBIDDEN** (produces non-deterministic output; violates
  reproducible build invariant and BC-4.01.003 invariant 3).
- **Duplicate section names** produce the same GUID (by construction). This is acceptable
  per BC-4.01.003 edge case EC-011: both sections are emitted with the same GUID;
  PowerPoint's behavior with duplicate section GUIDs is undefined but non-crashing.
  A validation warning is emitted; no error is raised.

### Element Ordering Constraint

`p:extLst` is the **last child element** in `p:presentation`'s content model
(CT_Presentation in ECMA-376 is a `sequence`; `p:extLst` is the terminal optional
element). It must appear after `p:sldIdLst`, `p:sldSz`, `p:notesSz`,
`p:defaultTextStyle`, and all other `p:presentation` children. Inserting `p:extLst`
earlier in the sequence produces a schema-invalid document that PowerPoint may reject
or silently corrupt.

For `SectionListBuilder`, this ordering is enforced by injecting the `p:extLst` block
as the final XML before `</p:presentation>` in the post-processing step (see
implementation decision below).

### ooxmlsdk-vs-Raw-XML Implementation Decision

**Decision: raw XML construction via `quick-xml` + string injection into `presentation.xml`
bytes, consistent with the established W1/W2 post-processing pattern in this crate.**

Rationale:

1. **`ooxmlsdk` 0.6.1 has no typed support for p14.** The crate generates Rust types from
   ECMA-376 Part 1 / .NET SDK metadata. The `p14` namespace
   (`http://schemas.microsoft.com/office/powerpoint/2010/main`) is not in the generated
   type set. There are no `P14SectionList`, `P14Section`, etc. structs. Confirmed by
   inspection of `lib.rs/crates/ooxmlsdk` (no `p14_*` types; no `presentation_2010` module).

2. **`ooxmlsdk`'s `p:ext` has no generic "any XML" slot.** The `p:ext` children are
   strongly typed to known extensions only. There is no `xml_children: Vec<AnyElement>`
   or `raw_xml: String` escape field. Unknown extension elements are silently dropped on
   deserialization and cannot be injected on serialization through the typed API.

3. **Post-processing is the established pattern in this crate.** Workaround W2
   (`opc_postprocess.rs`: inject `<Default>` entries into `[Content_Types].xml` after
   ooxmlsdk serializes) is the direct precedent. W1 (table serialization via
   `GraphicData.xml_children`) shows raw bytes injection at the element level.
   `SectionListBuilder` follows the same pattern: ooxmlsdk builds the base
   `p:presentation` bytes, then `SectionListBuilder` appends the `p:extLst` block.

4. **`quick-xml` is already a workspace dep (=0.36.0)** used in `slideforge-pptx` dev
   dependencies and across the workspace. It is the correct tool for the post-processing
   step. Adding it as a production dep to `slideforge-pptx` for this story is
   justified and minimal.

**Implementation sketch for `SectionListBuilder`:**

```rust
// sections.rs
// 1. Accept presentation_xml_bytes: Vec<u8> + sections: &[SlideSectionEntry]
// 2. If sections is empty, return bytes unchanged (no p:extLst injection).
// 3. Build p:extLst XML string using format! / quick-xml Writer:
//    - xmlns:p14 on p14:sectionLst element
//    - fixed uri="{BB962C8B-B8C3-4F9C-9F0B-04B162FE9A02}"
//    - one p14:section per SlideSectionEntry with deterministic GUID
//    - XML-escape section names (existing xml_escape.rs utility)
// 4. Locate closing </p:presentation> tag in the bytes (always the final tag).
// 5. Insert p:extLst XML immediately before </p:presentation>.
// 6. Return the modified bytes.
// Also: inject xmlns:p14 declaration into the <p:presentation ...> opening tag
//       so namespace is declared at the root element (preferred form).
```

**Why not fork ooxmlsdk to add p14 types?** Forking ooxmlsdk is a multi-day effort,
adds a vendor maintenance burden, and introduces supply-chain risk for a single
extension element. The post-processing approach delivers the correct output with
~100 lines of straightforward code. If ooxmlsdk gains typed p14 support in a future
version, migration to the typed API is a clean swap.

### p14 Namespace Declaration Placement

Add `xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"` to the
`<p:presentation ...>` opening tag during the injection step (not only on
`<p14:sectionLst>`). This is the canonical form in Office-generated files and avoids
namespace declarations deep in the tree that some parsers handle poorly.

When `SectionListBuilder` injects the `p:extLst`, it must also patch the
`<p:presentation` opening tag to include the `p14` namespace declaration.

## Format Matrix

| Format | Library | Phase | Key Constraint | Gate |
|--------|---------|-------|---------------|------|
| PPTX | ooxmlsdk 0.6.1 | 3 | SSIM ≥ 0.99, PSNR ≥ 35dB | Visual regression CI |
| DOCX | ooxmlsdk 0.6.1 | 3 | Multi-renderer round-trip | Snapshot test |
| PDF | pdf-writer + krilla | 4 | PDF/UA-1 compliant | veraPDF CI gate |
| HTML | axum + SVG | 3-4 | WCAG AA zero violations | axe-core CI gate |
| Preview | axum + WebSocket | 3 | SVG canvas (not canvas element) | axe-core CI gate |
