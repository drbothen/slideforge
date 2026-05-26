---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-042
title: "DOCX: Auto-Generated Document Sections"
epic: EPIC-09
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-docx
target_module: slideforge-docx
subsystems: [SS-08]
depends_on:
  - STORY-041
  - STORY-027
blocks:
  - STORY-049
  - STORY-050
estimated_days: 2
---

# STORY-042: DOCX: Auto-Generated Document Sections

## Subsystem Anchor Justification

SS-08 (DOCX Export) owns this story because it extends the DOCX serializer built in
STORY-041 with auto-generated and manually-authored document sections. The ARCH-INDEX
Subsystem Registry lists `slideforge-docx` under SS-08.

## Dependency Anchor Justifications

- Depends on STORY-041 (DOCX Core Serialization): The DOCX exporter structure,
  `DocxZipAssembler`, and `DocumentBodySerializer` established in STORY-041 are extended
  here. STORY-041's document body provides the narrative foundation; this story adds
  the auto-generated and manually-authored sections that follow it.
- Depends on STORY-027 (Layout: Document Section Generation): STORY-027 defines how
  the layout engine computes `AutoSection` candidates from slide data (e.g., which
  `severity_cards` slides contribute to `risk_register`). `LaidOutDeck.auto_sections`
  and `LaidOutDeck.manual_sections` carry this data. STORY-042 serializes those
  structures to DOCX XML.
- Blocks STORY-049/050: Integration tests verify auto-sections in the final PPTX+DOCX output.

## Summary

Extend `slideforge-docx` with auto-generated and manually-authored document sections.
Auto-generated sections are declarative rules managed by the `SectionType` plugin:
`executive_summary` is generated from `takeaway` fields across all slides;
`risk_register` is generated from `severity_cards` slides.

### Document Structure After This Story

```
word/document.xml (extended):

[Intro / TOC placeholder]
[Per-slide narrative body: STORY-041]
  Heading 1: Slide 1 Title
    Report paragraph 1
    Report paragraph 2
  Heading 1: Slide 2 Title
    Report paragraph

[Auto-generated sections: THIS STORY]
  Heading 1: Executive Summary
    - Takeaway from slide 1
    - Takeaway from slide 2

  Heading 1: Risk Register
    [Table: severity_cards content]
    | Risk | Severity | Description |
    |------|----------|-------------|
    | R001 | HIGH     | ...         |

[Manually-authored sections: THIS STORY]
  Heading 1: Methodology
    [section methodology: content]
```

### Section Ordering Rule

Per BC-4.02.002 postcondition 5:
1. Intro
2. Per-slide narrative body (STORY-041)
3. Auto-generated sections (this story)
4. Manually-authored sections (this story)
5. Approval section (if present)

### Auto-Generation Rules

| Section Name | Source Slide Type | Aggregation |
|-------------|------------------|-------------|
| `executive_summary` | Any slide with non-empty `takeaway` field | Bullet list of all `takeaway` values |
| `risk_register` | `slide severity_cards:` | Table: one row per slide, columns: title/severity/description |
| (extensible via SectionType plugin) | | |

The rules are managed by the `SectionType` plugin (BC-4.02.002 invariant 1). The DOCX
exporter reads `LaidOutDeck.auto_sections: Vec<AutoSection>` — it does NOT hardcode
section logic. Each `AutoSection` carries its source data already extracted by the
layout stage (STORY-027).

### Risk Register Table XML

```xml
<w:tbl>
  <w:tblPr>
    <w:tblStyle w:val="TableGrid"/>
    <w:tblW w:w="0" w:type="auto"/>
  </w:tblPr>
  <!-- Header row -->
  <w:tr>
    <w:tc><w:p><w:pPr><w:pStyle w:val="TableHeader"/></w:pPr><w:r><w:t>Risk</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:pPr><w:pStyle w:val="TableHeader"/></w:pPr><w:r><w:t>Severity</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:pPr><w:pStyle w:val="TableHeader"/></w:pPr><w:r><w:t>Description</w:t></w:r></w:p></w:tc>
  </w:tr>
  <!-- Data rows: one per severity_cards slide -->
  <w:tr>
    <w:tc><w:p><w:r><w:t>R001</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>HIGH</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>Infrastructure outage risk</w:t></w:r></w:p></w:tc>
  </w:tr>
</w:tbl>
```

Add styles `TableGrid` (table style) and `TableHeader` (header cell style) to `styles.xml`.

### Manual Section Handling

A `section methodology:` block in the DSL produces a `LaidOutManualSection` with:
- `name: "methodology"`
- `content: Vec<ContentBlock>` (the authored content)
- `position: SectionPosition` (declared position, or `After` = after auto-sections)

Manual sections are serialized as:
```xml
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Methodology</w:t></w:r></w:p>
<!-- content paragraphs -->
```

BC-4.02.002 invariant 3: manual sections appear at their DECLARED position. They are NOT
merged with auto-generated sections of the same name.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.02.002 | Auto-generated document sections present in DOCX from slide data | AC-001 through AC-008 |

## Acceptance Criteria

### AC-001: executive_summary section from takeaway fields
(traces to BC-4.02.002 postcondition 2 — executive_summary contains takeaway content)

A 2-slide deck where slide 1 has `takeaway "Key finding: X"` and slide 2 has
`takeaway "Key finding: Y"` produces a DOCX with:
1. A `Heading 1` paragraph "Executive Summary"
2. Two bullet paragraphs: "Key finding: X" and "Key finding: Y"

A unit test parses `document.xml` and asserts both takeaway strings appear under the
"Executive Summary" heading.

### AC-002: risk_register table from severity_cards slides
(traces to BC-4.02.002 postcondition 3 — risk_register from severity_cards in tabular format)

A deck with 3 `slide severity_cards:` blocks (HIGH, MED, LOW) produces a DOCX with:
1. A `Heading 1` paragraph "Risk Register"
2. A `<w:tbl>` with 1 header row + 3 data rows

A unit test parses `document.xml`, finds the `<w:tbl>` in the risk_register section,
and asserts row count == 4 (header + 3 data rows).

### AC-003: manually-authored section at declared position
(traces to BC-4.02.002 postcondition 4 — manual sections at declared position)

A deck with `section methodology:` block produces a DOCX with a "Methodology" `Heading 1`
section containing the authored content. A unit test parses `document.xml` and asserts
the "Methodology" heading and its content are present.

### AC-004: section ordering (narrative → auto → manual)
(traces to BC-4.02.002 postcondition 5 — section ordering follows the rule)

A deck with narrative body slides, a `severity_cards` slide (triggers auto risk_register),
and a `section methodology:` block (manual) produces a DOCX where the order in
`document.xml` is:
1. Per-slide narrative headings (from STORY-041)
2. "Risk Register" (auto-generated)
3. "Methodology" (manually-authored)

A unit test parses `document.xml`, extracts all `Heading1` element positions, and
asserts their order.

### AC-005: auto-section absent when no source slides
(traces to BC-4.02.002 invariant 2 — empty sections not emitted)

A deck with no `severity_cards` slides produces a DOCX with NO "Risk Register" section.
A unit test parses `document.xml` and asserts "Risk Register" is absent.

### AC-006: executive_summary absent when all takeaway fields empty
(traces to BC-4.02.002 edge case EC-004 — empty source data → section not emitted)

A deck where all slides have empty `takeaway` fields (or no `takeaway` fields) produces
no "Executive Summary" section. A unit test asserts "Executive Summary" is absent from
`document.xml`.

### AC-007: manual section NOT merged with auto-generated section
(traces to BC-4.02.002 invariant 3 — manual at declared position, not merged)

A deck with both `section methodology:` (manual) and a `slide methodology_chart:` that
would auto-generate a methodology section produces TWO separate sections — the manual
one at its declared position, not merged. A unit test counts "Methodology" heading
occurrences and asserts count == 1 (only the manually-declared one, since there is no
auto-methodology rule by default).

### AC-008: 20 severity_cards slides → 20-row table
(traces to BC-4.02.002 edge case EC-005 — no row limit)

A deck with 20 `severity_cards` slides produces a risk_register table with 20 data rows.
A unit test counts `<w:tr>` elements in the risk_register table and asserts count == 21
(header + 20 data rows).

## Tasks

- [ ] Implement `AutoSectionSerializer` in `src/auto_sections.rs`:
  - `executive_summary(items: &[Arc<str>]) -> Vec<XmlElement>` — bullet list
  - `risk_register(rows: &[SeverityCardRow]) -> Vec<XmlElement>` — table
  - Dispatch on `AutoSection.section_type` (string matching "executive_summary", "risk_register", etc.)
- [ ] Implement `ManualSectionSerializer` in `src/manual_sections.rs`:
  - Accept `LaidOutManualSection { name, content, position }`
  - Emit `Heading1` + content paragraphs
- [ ] Implement `SectionOrderer` in `src/section_order.rs`:
  - Collect: narrative body from STORY-041, auto-sections, manual sections
  - Sort by: `SectionPosition` (narrative < auto < manual, unless manual declares earlier position)
  - Return ordered list of XML blocks for `DocumentBodySerializer`
- [ ] Update `styles.xml` to include `TableGrid` and `TableHeader` styles
- [ ] Update `DocumentBodySerializer` (from STORY-041) to call `SectionOrderer`
- [ ] Write unit tests for all ACs

## Previous Story Intelligence

STORY-041 provides the `DocumentBodySerializer` that generates per-slide narrative body
paragraphs. This story extends it with a post-narrative section block:

1. `SectionOrderer` collects all sections (narrative, auto, manual)
2. Orders them by `SectionPosition`
3. Returns a flat list of XML elements for the body serializer to emit

The `LaidOutDeck.auto_sections` and `LaidOutDeck.manual_sections` fields are populated
by STORY-027 (layout stage). If STORY-027 has not yet added these fields to `LaidOutDeck`,
this story adds them as `Vec<AutoSection>` and `Vec<LaidOutManualSection>` with empty
defaults.

## Architecture Compliance Rules

1. **Section rules managed by SectionType plugin (BC-4.02.002 invariant 1, DI-008)**:
   `slideforge-docx` reads `LaidOutDeck.auto_sections` (computed by the layout engine
   using the SectionType plugin). It does NOT hardcode `executive_summary` vs
   `risk_register` logic — it dispatches on section type names. This makes it extensible.
2. **No empty auto-sections (BC-4.02.002 invariant 2)**: `AutoSectionSerializer` skips
   sections where `items.is_empty()`.
3. **Manual sections at declared position (BC-4.02.002 invariant 3)**: `SectionOrderer`
   respects `LaidOutManualSection.position` — manual sections are NOT forcibly placed
   after all auto-sections if they declare an earlier position.
4. **ooxmlsdk for table XML**: `<w:tbl>`, `<w:tr>`, `<w:tc>` elements constructed via
   ooxmlsdk typed builders. No string concatenation.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | `<w:tbl>` and section XML |
| `slideforge-types` | workspace | `AutoSection`, `LaidOutManualSection`, `SectionPosition` |
| `zip` | `=4.2.0` | DOCX ZIP (compatible with ooxmlsdk 0.6.1 dep) |
| `insta` | `=1.42.0` | Snapshot tests for section XML |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-docx/src/auto_sections.rs` | Create | `AutoSectionSerializer` |
| `crates/slideforge-docx/src/manual_sections.rs` | Create | `ManualSectionSerializer` |
| `crates/slideforge-docx/src/section_order.rs` | Create | `SectionOrderer` |
| `crates/slideforge-docx/src/document_body.rs` | Modify | Call `SectionOrderer` after narrative |
| `crates/slideforge-docx/src/styles.rs` | Modify | Add `TableGrid`, `TableHeader` styles |
| `crates/slideforge-docx/src/tests/section_tests.rs` | Create | AC-001 through AC-008 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-4.02.002 | ~1,500 |
| STORY-041 code reference | ~1,500 |
| STORY-027 types reference | ~800 |
| ooxmlsdk table API | ~1,000 |
| Test files | ~2,500 |
| **Total** | **~9,800** |

## Test Strategy

- **Unit tests**: All eight ACs (executive_summary bullets, risk_register table,
  manual section, ordering, empty-source-no-section, 20-row table).
- **Snapshot test**: `document.xml` for a deck with all section types (narrative +
  executive_summary + risk_register + methodology) to catch ordering regressions.
- **Edge case tests**: Empty takeaway all slides → no executive_summary;
  no severity_cards → no risk_register.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No `severity_cards` slides | No risk_register section in DOCX |
| EC-002 | Multiple `severity_cards` slides | All rows in single risk_register table |
| EC-003 | `takeaway` field empty on all slides | No executive_summary section |
| EC-004 | Manual `section methodology:` only | One methodology section; no auto-section duplication |
| EC-005 | 20 severity_cards slides | 20-row table; no row limit enforced |

## Forbidden Dependencies

Same as STORY-041 — no sibling exporter deps, no upstream crate deps.
