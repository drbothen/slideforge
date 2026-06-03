---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-041
title: "DOCX Core Serialization: report register + ooxmlsdk"
epic: EPIC-09
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.02.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-docx
target_module: slideforge-docx
subsystems: [SS-08]
depends_on:
  - STORY-026
  - STORY-035
  - STORY-036
blocks:
  - STORY-042
  - STORY-049
  - STORY-050
estimated_days: 3
---

# STORY-041: DOCX Core Serialization: report register + ooxmlsdk

## Subsystem Anchor Justification

SS-08 (DOCX Export) owns this story because it implements the DOCX exporter. The
ARCH-INDEX Subsystem Registry lists `slideforge-docx` under SS-08 as the effectful
shell crate for DOCX output.

## Dependency Anchor Justifications

- Depends on STORY-026 (Layout Core): The `LaidOutDeck` IR is the primary input.
  `LaidOutSlide.title`, `LaidOutSlide.frames`, and `LaidOutSlide.register_content` are
  consumed here.
- Depends on STORY-035 (Register Routing): `register_content` tagged with `Register::Report`,
  `Register::Notes`, `Register::Detail` is the routing infrastructure this story consumes.
  The DOCX exporter reads ONLY `Register::Report` from the slide body and
  `Register::Detail` for extended sections.
- Depends on STORY-036 (No Bleed Invariant): The `BleedChecker` utility is used in this
  story's tests. Un-ignoring AC-004 and AC-005 from STORY-036's bleed_tests.
- Blocks STORY-042 (Auto-Generated Sections): STORY-042 extends the DOCX structure built here.
- Blocks STORY-049/050: Integration tests require a working DOCX exporter.

## Summary

Implement the `slideforge-docx` crate as an `Exporter` plugin. The DOCX exporter
produces a Word-compatible `.docx` file where:

- `report` register content → narrative body paragraphs under per-slide headings
- `detail` register content → extended sections after the main body
- `notes` register content → explicitly excluded (never appears in `.docx` body)
- Visual slide content (titles, bullets) → represented as heading/summary, not duplicated

### DOCX ZIP Structure

```
[Content_Types].xml
_rels/.rels
word/document.xml              <!-- main body -->
word/_rels/document.xml.rels
word/styles.xml                <!-- paragraph + character styles -->
word/numbering.xml             <!-- list numbering definitions -->
word/settings.xml
word/theme/theme1.xml          <!-- brand color theme -->
word/media/image1.png          <!-- embedded images, if any -->
docProps/core.xml              <!-- dc:language from deck lang -->
docProps/app.xml
```

### document.xml Structure

The body content follows this ordering per BC-4.02.002 (auto-sections handled in STORY-042):

```xml
<w:body>
  <!-- Per-slide narrative body -->
  <w:p>  <!-- Heading 1: slide title -->
    <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
    <w:r><w:t>Slide Title</w:t></w:r>
  </w:p>
  <w:p>  <!-- Body paragraph: report register content -->
    <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
    <w:r><w:t>Report narrative text here...</w:t></w:r>
  </w:p>
  <!-- Repeat for each slide -->

  <!-- Extended sections: detail register content -->
  <w:p>
    <w:pPr><w:pStyle w:val="Heading2"/></w:pPr>
    <w:r><w:t>Appendix: Slide Title</w:t></w:r>
  </w:p>
  <w:p>
    <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
    <w:r><w:t>Detail content here...</w:t></w:r>
  </w:p>

  <w:sectPr/>  <!-- Section properties: page size, margins -->
</w:body>
```

### Inline Formatting: InlineSpan → Word Run Properties

| InlineSpan variant | Word XML |
|-------------------|---------|
| `Bold(spans)` | `<w:rPr><w:b/></w:rPr>` |
| `Italic(spans)` | `<w:rPr><w:i/></w:rPr>` |
| `Code(text)` | `<w:rPr><w:rFonts w:ascii="Courier New"/></w:rPr>` |
| `Link { text, url }` | `<w:hyperlink r:id="rId...">` with `<w:rPr><w:rStyle w:val="Hyperlink"/></w:rPr>` |
| `Strikethrough(spans)` | `<w:rPr><w:strike/></w:rPr>` |
| `Superscript(spans)` | `<w:rPr><w:vertAlign w:val="superscript"/></w:rPr>` |
| `Subscript(spans)` | `<w:rPr><w:vertAlign w:val="subscript"/></w:rPr>` |
| `Text(s)` | `<w:r><w:t xml:space="preserve">text</w:t></w:r>` |

Note: `xml:space="preserve"` is required on `<w:t>` when the text has leading or trailing
whitespace (otherwise Word collapses it).

### styles.xml

Provide at minimum these styles:
- `Heading1` (slide title level)
- `Heading2` (section title level)
- `Normal` (body paragraph)
- `Hyperlink` (character style for links)
- `CodeText` (character style for inline code, Courier New font)

Style definitions sourced from the brand when available; minimal valid stubs otherwise.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.02.001 | Serialize deck to .docx with report register as body paragraphs | AC-001 through AC-009 |

## Acceptance Criteria

### AC-001: Exporter plugin trait implemented
(traces to BC-4.02.001 postcondition 1 — a valid .docx file is written at the output path)

`slideforge-docx` implements `Exporter` via the plugin trait API. The `DocxExporter`
struct is registered via the plugin registry. Method signature matches `Exporter` trait.

### AC-002: Valid .docx ZIP produced with all required parts
(traces to BC-4.02.001 postcondition 2 — .docx passes OOXML schema validation)

A 1-slide deck produces a `.docx` ZIP containing `word/document.xml`, `word/styles.xml`,
`word/_rels/document.xml.rels`, `[Content_Types].xml`, `_rels/.rels`, `docProps/core.xml`.
A unit test enumerates ZIP entries and asserts all required paths present.

### AC-003: report register content in document.xml body
(traces to BC-4.02.001 postcondition 3 — report content as body paragraphs)

A slide with `report "Analysis follows."` produces a `<w:p>` with style `Normal`
containing a `<w:r><w:t>Analysis follows.</w:t></w:r>` in `word/document.xml`.
A unit test parses `document.xml` from the DOCX ZIP and asserts the text is present.

### AC-004: Slide title as Heading1 before report content
(traces to BC-4.02.001 postcondition 3 — report content under heading from slide title)

A slide with `title "Q1 Review"` and `report "Report text"` produces:
1. `<w:p style="Heading1">Q1 Review</w:p>` BEFORE the report paragraph
2. `<w:p style="Normal">Report text</w:p>`

A unit test verifies ordering: heading element appears before body paragraph in the XML.

### AC-005: notes register content absent from document.xml body
(traces to BC-4.02.001 invariant 1 — notes NEVER in DOCX body)

Un-ignore AC-004 from STORY-036's `bleed_tests.rs`. A deck with `notes "NOTES_DOCX_SENTINEL"`
produces a `document.xml` that does NOT contain "NOTES_DOCX_SENTINEL". Use
`BleedChecker::assert_absent_from_docx_body`.

### AC-006: report register content present (positive bleed test)
(traces to BC-4.02.001 postcondition 3 — report IS in DOCX body)

Un-ignore AC-005 from STORY-036's `bleed_tests.rs`. A deck with
`report "REPORT_DOCX_SENTINEL"` produces a `document.xml` that DOES contain
"REPORT_DOCX_SENTINEL". Use `BleedChecker::assert_present_in_docx_body`.

### AC-007: detail register content in extended section of document.xml
(traces to BC-4.02.001 postcondition 4 — detail in supplementary sections)

A slide with `detail "Technical appendix text"` produces a `<w:p style="Normal">`
containing "Technical appendix text" after the main narrative body sections. A unit
test parses `document.xml`, finds the position of the detail text relative to the
report text, and asserts detail appears after the last report paragraph.

### AC-008: Inline formatting mapped to Word run properties
(traces to BC-4.02.001 postcondition 7 — inline formatting preserved in DOCX)

A report field containing `"**Bold** and *italic*"` produces:
- A `<w:r>` with `<w:rPr><w:b/></w:rPr>` containing "Bold"
- A `<w:r>` containing " and "
- A `<w:r>` with `<w:rPr><w:i/></w:rPr>` containing "italic"

### AC-009: Slide with no report/detail content has heading + empty paragraph
(traces to BC-4.02.001 postcondition 6 — no content omitted silently)

A slide with only visual content (no report/detail register) produces:
- `<w:p style="Heading1">Slide Title</w:p>`
- `<w:p/>` (empty normal paragraph — not omitted)

This prevents layout issues in Word where headings without following content cause
spacing problems.

## Tasks

- [ ] **Task 1: Crate scaffold and Exporter trait impl**
  - Create `crates/slideforge-docx/` with `Cargo.toml`
  - Define `DocxExporter` struct
  - Implement `Exporter` trait

- [ ] **Task 2: ZIP assembler (reuse ZipAssembler pattern from slideforge-pptx)**
  - `DocxZipAssembler` — similar to PPTX version; deterministic entry ordering, epoch timestamps

- [ ] **Task 3: [Content_Types].xml for DOCX**
  - Required content types:
    - `word/document.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml`
    - `word/styles.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml`
    - `word/numbering.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml`
    - `word/settings.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml`

- [ ] **Task 4: styles.xml generator**
  - Generate styles: `Heading1`, `Heading2`, `Normal`, `Hyperlink`, `CodeText`
  - Source from brand when available (fonts, colors); minimal stubs otherwise

- [ ] **Task 5: Document body serializer**
  - Implement `DocumentBodySerializer` in `src/document_body.rs`
  - For each `LaidOutSlide`:
    - Emit `<w:p style="Heading1">` with slide title
    - Emit `<w:p style="Normal">` for each `Register::Report` content block
    - Emit empty `<w:p/>` if no report content (AC-009)
  - After all slides: emit extended sections from `Register::Detail` content blocks
  - Inline formatting: `InlineSpan → <w:rPr>` mapping

- [ ] **Task 6: Hyperlink relationship support**
  - For `InlineSpan::Link { text, url }`: create `word/_rels/document.xml.rels` entry
  - Use sequential `rId` values

- [ ] **Task 7: docProps/core.xml with dc:language**
  - `dc:language` from `LaidOutDeck.lang`

- [ ] **Task 8: Unit tests**
  - AC-002: ZIP structure
  - AC-003: report text in document.xml
  - AC-004: heading before report
  - AC-005: un-ignore bleed test (notes absent)
  - AC-006: un-ignore bleed test (report present)
  - AC-007: detail in extended section
  - AC-008: inline formatting (bold+italic)
  - AC-009: empty paragraph for visual-only slide

## Previous Story Intelligence

N/A — first story in EPIC-09. The pattern mirrors `slideforge-pptx` (STORY-037):
same `Exporter` trait, same ZIP assembly pattern, similar XML construction. The key
difference: DOCX uses `w:` namespace XML while PPTX uses `p:` namespace.

The `register_content` field from STORY-035 is the authoritative content source. This
exporter reads only `Register::Report` and `Register::Detail` — it explicitly excludes
`Register::Notes` from the document body.

## Architecture Compliance Rules

1. **`notes` register never in DOCX body (BC-4.02.001 invariant 1, DI-012)**:
   `DocumentBodySerializer` must check `register == Register::Report` or
   `register == Register::Detail` before emitting content. `Register::Notes` entries
   are skipped. BleedChecker verifies this at test time.
2. **Two-IR model (BC-4.02.001 invariant 3, DI-009)**: DOCX exporter reads from
   `LaidOutDeck` only. Does not call the evaluator or parser.
3. **ooxmlsdk for all OOXML construction**: No XML string concatenation.
4. **`xml:space="preserve"` on `<w:t>` with whitespace**: Required to prevent Word
   from collapsing leading/trailing spaces in run text.
5. **SS-08 effectful shell**: `slideforge-docx` is an effectful shell (writes files).
   The serialization computation is pure but file I/O is effectful.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | DOCX OOXML element construction |
| `zip` | `=4.2.0` | ZIP assembly (compatible with ooxmlsdk 0.6.1 dep) |
| `slideforge-types` | workspace | `LaidOutDeck`, `Register`, `InlineSpan` |
| `slideforge-plugin-api` | workspace | `Exporter` trait |
| `thiserror` | `=2.0.18` | `ExportError` enum |
| `tracing` | `=0.1.43` | Structured logging |
| `insta` | `=1.42.0` | Snapshot tests for document.xml |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-docx/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-docx/src/lib.rs` | Create | `DocxExporter`, `Exporter` impl |
| `crates/slideforge-docx/src/zip_assembler.rs` | Create | DOCX ZIP assembly |
| `crates/slideforge-docx/src/content_types.rs` | Create | DOCX `[Content_Types].xml` |
| `crates/slideforge-docx/src/document_body.rs` | Create | `DocumentBodySerializer` |
| `crates/slideforge-docx/src/styles.rs` | Create | `styles.xml` generator |
| `crates/slideforge-docx/src/error.rs` | Create | `ExportError` enum |
| `crates/slideforge-docx/src/tests/core_tests.rs` | Create | AC unit tests |
| `Cargo.toml` (workspace root) | Modify | Add `slideforge-docx` to workspace members |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,500 |
| BC-4.02.001 | ~2,000 |
| STORY-035 register types | ~800 |
| STORY-036 BleedChecker | ~500 |
| ooxmlsdk DOCX API | ~2,000 |
| Files to create | ~5,000 |
| **Total** | **~13,800** |

## Test Strategy

- **Snapshot tests**: `document.xml` for a 2-slide deck with report+detail content.
- **Unit tests**: All ACs (ZIP structure, report present, notes absent, detail in extended
  section, inline formatting, empty paragraph for visual-only slides).
- **Bleed tests**: Un-ignore STORY-036 AC-004 and AC-005.
- **Determinism test**: Build same deck twice; SHA-256 compare.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only notes register (no report/detail) | DOCX: heading + empty paragraph; notes text absent |
| EC-002 | report block with multiple paragraphs | Multiple `<w:p>` elements, one per paragraph |
| EC-003 | report content with hyperlink | `<w:hyperlink r:id="rId...">` with hyperlink char style |
| EC-004 | detail and report both on same slide | report in narrative section; detail in extended section |
| EC-005 | 50-slide deck | 50 heading sections; all report content present; file opens without error |

## Forbidden Dependencies

`slideforge-docx` must NOT depend on:
- `slideforge-eval` — upstream; no circular dep
- `slideforge-layout` — upstream
- `slideforge-pptx` — sibling exporter
- `slideforge-pdf` — sibling exporter
- `slideforge-html` — sibling exporter
- `slideforge-syntax` — upstream

Build fails if any sibling or upstream exporter crate appears in `slideforge-docx/Cargo.toml`.
