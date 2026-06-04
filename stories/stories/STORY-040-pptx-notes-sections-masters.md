---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-040
title: "PPTX: Speaker Notes + notesMaster1.xml + handoutMaster1.xml"
epic: EPIC-08
wave: 4
points: 3
priority: P0
tdd_mode: strict
status: merged
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
estimated_days: 1
---

# STORY-040: PPTX: Speaker Notes + notesMaster1.xml + handoutMaster1.xml

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns this story because it extends the PPTX exporter with speaker
notes and the required notesMaster/handoutMaster structural parts.
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
3. **handoutMaster1.xml**: Always-present companion to notesMaster — minimal valid stub.

Slide sections (`<p:sectionLst>`) were split out to STORY-082 (slide-grouping follow-up)
per human-authorized scope split 2026-06-04. That feature requires a new DSL construct
(`section "Name":` slide-grouping blocks, distinct from `section IDENT:` document-structure
blocks) plus a `slide_sections` field on `LaidOutDeck` and an eval-stage membership mapping
that do not exist yet.

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
| BC-4.01.003 | PPTX speaker notes and master/layout/theme system | AC-001, AC-002, AC-003 |
| BC-4.01.006 | PPTX contains notesMaster1.xml and handoutMaster1.xml even if empty | AC-004, AC-005 |

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

### AC-004: notesMaster1.xml always present (even with no notes)
(traces to BC-4.01.006 invariant 1 — notesMaster always emitted)

A deck with zero `notes` fields still produces `ppt/notesMasters/notesMaster1.xml` in
the PPTX ZIP. A unit test enumerates ZIP entries and asserts the path is present.

### AC-005: handoutMaster1.xml always present
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
- [ ] Write unit tests for all five ACs

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
3. **No sectionLst in this story**: `SectionListBuilder` and `<p:sectionLst>` in
   `presentation.xml` are STRICTLY out of scope. Any code touching `sections.rs` or
   `LaidOutDeck.slide_sections` belongs to STORY-082 (slide-grouping follow-up).

## Forbidden Dependencies

Same as STORY-037: no upstream crate deps beyond `slideforge-types` and `slideforge-plugin-api`,
no sibling exporter deps (no `slideforge-docx`, `slideforge-pdf`, `slideforge-html`). The
`sha2` crate is NOT needed here — that dependency belongs to STORY-082 (section GUID generation).
If `sha2` appears in `slideforge-pptx/Cargo.toml` as a result of this story, it is a defect.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | notesSlide XML construction |
| `zip` | `=4.2.0` | ZIP assembly (compatible with ooxmlsdk 0.6.1 dep) |
| `slideforge-types` | workspace | `Register::Notes`, `LaidOutDeck` |

Note: `sha2` (seeded GUID generation) is explicitly NOT in this story's dependencies.
It belongs to STORY-082 only.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/src/notes_slide.rs` | Create | `NotesSlideSerializer` |
| `crates/slideforge-pptx/src/notes_master.rs` | Create | `NotesMasterSerializer`, handout master |
| `crates/slideforge-pptx/src/zip_assembler.rs` | Modify | Write notesSlide parts and their rels |
| `crates/slideforge-pptx/src/content_types.rs` | Modify | Add notesSlide content type overrides |
| `crates/slideforge-pptx/src/tests/notes_tests.rs` | Create | AC-001 through AC-005 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,000 |
| BC-4.01.003 (Half A postconditions only) | ~800 |
| BC-4.01.006 | ~1,000 |
| STORY-037/038 code reference | ~1,500 |
| STORY-035 register types | ~800 |
| Test files | ~1,200 |
| **Total** | **~7,300** |

## Test Strategy

- **Unit tests**: All five ACs including notes count equality, notes text in body
  placeholder, notes absent from slide body (BleedChecker), notesMaster/handoutMaster
  always present.
- **Snapshot test**: `notesSlide1.xml` for a slide with rich notes content (bold, links).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with empty notes register | No notesSlide part for that slide |
| EC-003 | Notes content with `{{ }}` interpolation | Evaluated before embedding (STORY-035 resolves it) |
