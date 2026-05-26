---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-040
title: "PPTX: Speaker Notes + Slide Sections + notesMaster1.xml"
epic: EPIC-08
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.01.003, BC-4.01.006]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pptx
target_module: slideforge-pptx
subsystems: [SS-06]
depends_on:
  - STORY-038
  - STORY-035
blocks:
  - STORY-049
  - STORY-050
estimated_days: 2
---

# STORY-040: PPTX: Speaker Notes + Slide Sections + notesMaster1.xml

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns this story because it extends the PPTX exporter with speaker
notes, slide sections, and the required notesMaster/handoutMaster structural parts.
`slideforge-pptx` is the sole crate involved.

## Dependency Anchor Justifications

- Depends on STORY-038 (Layout Compliance): Speaker notes (`notesSlide` parts) must
  reference the `notesMaster1.xml` which exists after STORY-037/038. The slide XML
  structure established there is extended here.
- Depends on STORY-035 (Register Routing): Notes content comes from
  `LaidOutSlide.register_content` entries tagged as `Register::Notes`. Without STORY-035,
  notes content is unavailable.
- Blocks STORY-049/050: Integration tests require speaker notes to be present for
  full-pipeline verification.

## Summary

This story completes the PPTX speaker notes and structural master requirements:

1. **Speaker notes**: For each slide with non-empty `Register::Notes` content, produce
   a `ppt/notesSlides/notesSlide{N}.xml` part referencing `notesMaster1.xml`.
2. **notesMaster1.xml**: Replace the empty stub from STORY-037 with a valid notes master
   that the notesSlide parts can reference. Styled from the brand if available; minimal
   valid otherwise.
3. **Slide sections**: When the deck contains `section "Name":` blocks, produce
   `<p:sectionLst>` in `presentation.xml` grouping slides by their section membership.

### notesSlide XML Structure

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
         xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:grpSpPr/>
      <!-- Slide image placeholder (sp type="sldImg") -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Slide Image Placeholder 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="sldImg"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
      </p:sp>
      <!-- Notes text placeholder -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Notes Placeholder 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r><a:t>NOTES_CONTENT_HERE</a:t></a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:notes>
```

The `notesSlide` part also needs a `.rels` file:
```
ppt/notesSlides/_rels/notesSlide1.xml.rels
```
containing relationships to: (1) the slide it belongs to
(`http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide`) and (2)
the notes master.

### Slide Sections in presentation.xml

When `section "Background":` groups slides 1-3 and `section "Analysis":` groups slides
4-6, the `<p:sectionLst>` element is produced:

```xml
<p:presentationPr>
  <p:sectionLst>
    <p:section name="Background" id="{GUID}">
      <p:sldIdLst>
        <p:sldId id="256"/>
        <p:sldId id="257"/>
        <p:sldId id="258"/>
      </p:sldIdLst>
    </p:section>
    <p:section name="Analysis" id="{GUID}">
      <p:sldIdLst>
        <p:sldId id="259"/>
        <p:sldId id="260"/>
        <p:sldId id="261"/>
      </p:sldIdLst>
    </p:section>
  </p:sectionLst>
</p:presentationPr>
```

Section GUIDs are generated deterministically from the section name using
`uuid::Builder::from_sha1_str(section_name)` or similar seeded approach — NOT
`Uuid::new_v4()` (which is random and breaks determinism).

### notesMaster1.xml

Replace the empty stub from STORY-037 with a minimal valid notes master:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:grpSpPr/>
      <!-- date, footer, page number, slide image, body placeholders -->
    </p:spTree>
  </p:cSld>
  <p:clrMap ... />   <!-- inherits from theme -->
</p:notesMaster>
```

If the brand provides a notes master from the extracted `.pptx` template, use it verbatim.
Otherwise, generate a minimal valid one with the required placeholder types.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.003 | PPTX contains speaker notes, master/layout/theme system, slide sections | AC-001 through AC-006 |
| BC-4.01.006 | PPTX contains notesMaster1.xml and handoutMaster1.xml even if empty | AC-007, AC-008 |

## Acceptance Criteria

### AC-001: notesSlide part created for each slide with non-empty notes register
(traces to BC-4.01.003 postcondition 1 — every slide with notes has a notesSlide part)

A 3-slide deck where slides 1 and 3 have `notes "..."` content produces:
- `ppt/notesSlides/notesSlide1.xml` (for slide 1)
- `ppt/notesSlides/notesSlide3.xml` (for slide 3)
- NO `ppt/notesSlides/notesSlide2.xml` (slide 2 has no notes)

A unit test enumerates ZIP entries matching `ppt/notesSlides/notesSlide*.xml` and
asserts count == 2.

### AC-002: Notes content in notesSlide body placeholder
(traces to BC-4.01.003 postcondition 6 — notes visible in PowerPoint Notes pane)

The notes text from `Register::Notes` content appears in the `<p:txBody>` of the body
placeholder (`<p:ph type="body" idx="1">`) in the `notesSlide` XML. A unit test parses
`notesSlide1.xml` and asserts the text content of the body placeholder equals the notes
text from the deck.

### AC-003: Notes content NOT in slide body
(traces to BC-4.01.003 invariant 1 — speaker notes never in slide body content)

The notes text from `Register::Notes` does NOT appear in any `ppt/slides/slide*.xml`
file. Use `BleedChecker::assert_absent_from_pptx_slides` from STORY-036 to verify.

### AC-004: Slide sections produce sectionLst in presentation.xml
(traces to BC-4.01.003 postcondition 5 — sectionLst groups slides by section)

A deck with `section "Background":` (slides 1-2) and `section "Analysis":` (slides 3-4)
produces `<p:sectionLst>` in `presentation.xml` with two `<p:section>` elements. The
section names match the DSL declarations. A unit test parses `presentation.xml` from the
ZIP and asserts `sectionLst.sections.len() == 2` and section names are correct.

### AC-005: Deck with no sections has no sectionLst
(traces to BC-4.01.003 edge case EC-002 — no sections → no sectionLst)

A deck with no `section:` blocks produces `presentation.xml` without a `<p:sectionLst>`
element. A unit test parses `presentation.xml` and asserts `sectionLst` is absent.

### AC-006: XML-escaped section names
(traces to BC-4.01.003 edge case EC-004 — section name with special chars)

A section named `"Background & Overview"` produces
`<p:section name="Background &amp; Overview">` in the presentation XML. The XML is
well-formed (parseable without error).

### AC-007: notesMaster1.xml always present (even with no notes)
(traces to BC-4.01.006 invariant 1 — notesMaster always emitted)

A deck with zero `notes` fields still produces `ppt/notesMasters/notesMaster1.xml` in
the PPTX ZIP. A unit test enumerates ZIP entries and asserts the path is present.

### AC-008: handoutMaster1.xml always present
(traces to BC-4.01.006 postcondition 2 — handoutMaster always emitted)

Same assertion for `ppt/handoutMasters/handoutMaster1.xml`. Always present regardless
of deck content.

## Tasks

- [ ] Implement `NotesSlideSerializer` in `src/notes_slide.rs`:
  - Accept `slide_idx`, `notes_content: &[RegisteredContent]` (filtered to `Register::Notes`)
  - Generate `notesSlide{N}.xml` with notes text in body placeholder
  - Only called when notes content is non-empty for a slide
- [ ] Implement `NotesMasterSerializer` in `src/notes_master.rs`:
  - Generate minimal valid `notesMaster1.xml` (or embed brand's notes master if provided)
  - Generate `handoutMaster1.xml` (minimal valid stub)
- [ ] Update `ZipAssembler` to:
  - Write `notesSlide{N}.xml` for each slide with notes
  - Write `ppt/notesSlides/_rels/notesSlide{N}.xml.rels` for each notes slide
  - Update `[Content_Types].xml` with notesSlide content type overrides
- [ ] Implement `SectionListBuilder` in `src/sections.rs`:
  - Accept `Vec<(section_name: &str, slide_ids: Vec<u32>)>` from `LaidOutDeck`
  - Generate `<p:sectionLst>` XML
  - Deterministic GUID generation from section names (not random UUIDs)
- [ ] Update `PresentationSerializer` to include `<p:sectionLst>` when sections exist
- [ ] Write unit tests for all ACs

## Previous Story Intelligence

STORY-037 produced empty stub notesMaster and handoutMaster files. This story replaces
the stubs with real content and adds speaker notes. The key change:

1. `PptxExporter::export()` now calls `NotesSlideSerializer` for each slide with notes
2. `[Content_Types].xml` gets additional `<Override>` entries for each notesSlide part
3. `presentation.xml.rels` gets a `notesMaster` relationship (may already exist from STORY-037)

The `Register::Notes` content is available from `LaidOutSlide.register_content` (STORY-035).
Filter entries by `register == Register::Notes` and serialize their `content: Vec<InlineSpan>`.

## Architecture Compliance Rules

1. **Speaker notes in notesSlide only (BC-4.01.003 invariant 1, DI-012)**:
   Notes register content NEVER appears in `ppt/slides/slide*.xml`. Only in
   `ppt/notesSlides/notesSlide*.xml`. `BleedChecker` verifies this.
2. **notesMaster always present (BC-4.01.006 invariant 1)**: The `NotesMasterSerializer`
   is always called — no conditional. Even for decks with zero notes.
3. **Deterministic section GUIDs**: GUIDs derived from section names using a seeded
   hash, not `Uuid::new_v4()`. Same section name → same GUID → byte-identical output.
4. **Section data from DSL (BC-4.01.003 invariant 3)**: No section data is fabricated.
   Only sections declared in the DSL with `section "Name":` blocks are included.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | notesSlide XML construction |
| `zip` | `=4.2.0` | ZIP assembly (compatible with ooxmlsdk 0.6.1 dep) |
| `slideforge-types` | workspace | `Register::Notes`, `LaidOutDeck.sections` |
| `sha2` | `=0.10.9` | Seeded GUID generation for section IDs (deterministic) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/src/notes_slide.rs` | Create | `NotesSlideSerializer` |
| `crates/slideforge-pptx/src/notes_master.rs` | Create | `NotesMasterSerializer`, handout master |
| `crates/slideforge-pptx/src/sections.rs` | Create | `SectionListBuilder` |
| `crates/slideforge-pptx/src/presentation.rs` | Modify | Include `<p:sectionLst>` when sections exist |
| `crates/slideforge-pptx/src/zip_assembler.rs` | Modify | Write notesSlide parts and their rels |
| `crates/slideforge-pptx/src/content_types.rs` | Modify | Add notesSlide content type overrides |
| `crates/slideforge-pptx/src/tests/notes_tests.rs` | Create | AC-001 through AC-008 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-4.01.003 | ~1,500 |
| BC-4.01.006 | ~1,000 |
| STORY-037/038 code reference | ~1,500 |
| STORY-035 register types | ~800 |
| Test files | ~2,000 |
| **Total** | **~9,300** |

## Test Strategy

- **Unit tests**: All eight ACs including notes count equality, notes text in body
  placeholder, notes absent from slide body (BleedChecker), sections in presentation.xml,
  notesMaster/handoutMaster always present.
- **Snapshot test**: `notesSlide1.xml` for a slide with rich notes content (bold, links).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with empty notes register | No notesSlide part for that slide |
| EC-002 | Deck with no section blocks | No `<p:sectionLst>` in presentation.xml |
| EC-003 | Notes content with `{{ }}` interpolation | Evaluated before embedding (STORY-035 resolves it) |
| EC-004 | Section name with XML special characters | XML-escaped in `<p:section name>` attribute |
| EC-005 | Two sections each containing one slide | Both sections in sectionLst; each with one sldId |

## Forbidden Dependencies

Same as STORY-037 — no upstream crate deps, no sibling exporter deps.
