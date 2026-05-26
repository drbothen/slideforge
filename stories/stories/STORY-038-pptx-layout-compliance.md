---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-038
title: "PPTX Layout Compliance: Placeholder Inheritance + Slide IDs + Layouts"
epic: EPIC-08
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-4.01.005]
verification_properties: []
nfr_refs: [NFR-007, NFR-008, NFR-009, NFR-011, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-pptx
target_module: slideforge-pptx
subsystems: [SS-06]
depends_on:
  - STORY-037
  - STORY-023
blocks:
  - STORY-039
  - STORY-040
  - STORY-049
  - STORY-050
estimated_days: 3
---

# STORY-038: PPTX Layout Compliance: Placeholder Inheritance + Slide IDs + Layouts

## Subsystem Anchor Justification

SS-06 (PPTX Export) owns this story because it extends the PPTX exporter built in
STORY-037 with OOXML layout compliance rules. The ARCH-INDEX Subsystem Registry lists
`slideforge-pptx` under SS-06.

## Dependency Anchor Justifications

- Depends on STORY-037: All layout compliance work extends the `PptxExporter` crate
  created in STORY-037. Without the PPTX core, no layout compliance work can land.
- Depends on STORY-023: The 31-layout hierarchy comes from the `Brand` struct produced
  by brand synthesis. This story verifies those layouts are correctly embedded and
  referenced.
- Blocks STORY-039/040: Accessibility metadata and notes/sections build on top of the
  layout-compliant PPTX produced here.
- Blocks STORY-049/050: Integration tests require a layout-compliant PPTX.

## Summary

Extend `slideforge-pptx` with strict OOXML layout compliance: slide ID sequence starting
at 256, master ID = 2^31, all 31 slide layouts present in the PPTX even when not all
slide types appear in the deck, correct placeholder inheritance, and `clrMapOvr` for
dark-themed layouts.

This story directly implements BC-4.01.005 which was partially covered in STORY-037
but receives full coverage here with dedicated tests and the full 31-layout verification.

### Slide ID Assignment

```rust
// Start at 256 per ECMA-376 minimum (Spike S6 BUG-006)
const SLIDE_ID_START: u32 = 256;

// Master ID at 2^31 per ECMA-376 (same spike)
const MASTER_ID: u32 = 2_147_483_648;  // 2^31

fn assign_slide_ids(slide_count: usize) -> Vec<u32> {
    (SLIDE_ID_START..)
        .take(slide_count)
        .collect()
}
```

Generated `<p:sldIdLst>` in `presentation.xml`:
```xml
<p:sldIdLst>
  <p:sldId id="256" r:id="rId2"/>
  <p:sldId id="257" r:id="rId3"/>
  <!-- ... sequential -->
</p:sldIdLst>
```

Generated `<p:sldMasterIdLst>`:
```xml
<p:sldMasterIdLst>
  <p:sldMasterId id="2147483648" r:id="rId1"/>
</p:sldMasterIdLst>
```

### All 31 Layouts Present

Even if the deck uses only 5 of the 31 slide types, all 31 layout parts must be present
in `ppt/slideLayouts/`. This is required for template reuse and for renderer compatibility
(BC-4.01.005 invariant 3). The `Brand` struct from STORY-023 provides all 31 layout XMLs.

Layout file naming: `slideLayout1.xml` through `slideLayout31.xml`. The mapping from
slideforge slide type to layout index is:

| Layout Index | Slide Type |
|-------------|-----------|
| 1 | title_slide |
| 2 | title_content |
| 3 | two_column |
| 4 | bullets |
| 5 | executive_summary |
| 6 | key_insights |
| 7 | severity_cards |
| 8 | pro_con |
| 9 | timeline |
| 10 | team |
| 11 | bio |
| 12 | chart |
| 13 | table |
| 14 | image |
| 15 | quote |
| 16 | agenda |
| 17 | toc |
| 18 | section_divider |
| 19 | blank |
| 20 | announcement |
| 21 | comparison |
| 22 | roadmap |
| 23 | grid |
| 24 | metrics |
| 25 | closing |
| 26 | diagram |
| 27 | cover |
| 28 | stacked_content |
| 29 | split_content |
| 30 | full_bleed |
| 31 | sidebar |

The slide master references all 31 layouts via `<p:sldLayoutIdLst>` in `slideMaster1.xml`.

### clrMapOvr for Dark Layouts

Slides using dark-themed layouts (layouts with a dark background color defined in
the brand) must include `<p:clrMapOvr>`:

```xml
<p:sld>
  <p:cSld> ... </p:cSld>
  <p:clrMapOvr>
    <a:masterClrMapping/>
  </p:clrMapOvr>
</p:sld>
```

This tells PowerPoint to use the master's color map override for this slide, enabling
dark-on-light color inversion. The `Brand` struct carries a `dark_layout_indices: Vec<usize>`
field indicating which layout indices are dark-themed.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.005 | Slide IDs start at 256; master IDs at 2^31; all 31 layouts present | AC-001 through AC-008 |

## Acceptance Criteria

### AC-001: Slide IDs start at exactly 256, increment by 1
(traces to BC-4.01.005 invariant 1 — slide ID sequence starts at 256)

A 3-slide deck produces `<p:sldId id="256">`, `<p:sldId id="257">`, `<p:sldId id="258">`.
A unit test parses `presentation.xml` from the ZIP, extracts all `id` attributes from
`<p:sldId>` elements, and asserts the minimum is 256 and they are sequential with step 1.

### AC-002: Master ID is exactly 2147483648
(traces to BC-4.01.005 invariant 2 — master ID is exactly 2^31)

`<p:sldMasterId id="2147483648">` present in `presentation.xml`. A unit test parses
the master ID attribute and asserts it equals `2_147_483_648u32` (not a random value,
not `2147483649`).

### AC-003: Exactly 31 slideLayout parts in the PPTX ZIP
(traces to BC-4.01.005 postcondition 4 — 31 layout parts in ppt/slideLayouts/)

A unit test enumerates ZIP entries matching the pattern `ppt/slideLayouts/slideLayout*.xml`
and asserts `count == 31`. This test passes for ANY deck, including a 1-slide deck.

### AC-004: Each of the 31 slide types has a corresponding layout
(traces to BC-4.01.005 postcondition 5 — each slide type has a corresponding layout)

The `<p:sldLayoutIdLst>` in `slideMaster1.xml` has exactly 31 `<p:sldLayoutId>` entries,
one for each layout. A unit test parses `slideMaster1.xml` from the ZIP and asserts
`sldLayoutIdLst.len() == 31`.

### AC-005: Content_Types.xml registers all 31 layouts
(traces to BC-4.01.005 postcondition 6 — Content_Types registers all layouts)

The `[Content_Types].xml` has exactly 31 `<Override>` entries with
`ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"`.
A unit test counts these entries.

### AC-006: Slide using dark layout has clrMapOvr
(traces to BC-4.01.005 postcondition 1 — invariant: schema-compliant element ordering
covers dark layout handling per brand-architecture.md)

For a brand with `dark_layout_indices = [6, 18]` (severity_cards, section_divider),
a slide using layout index 6 has `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>`
in its `<p:sld>` element. A slide using layout index 1 (title_slide, not dark) does
NOT have `<p:clrMapOvr>`.

### AC-007: 257-slide deck — all slide IDs unique and ≥ 256
(traces to BC-4.01.005 edge case EC-003)

A 257-slide deck produces slide IDs 256 through 512. A unit test verifies all IDs are
unique and all ≥ 256. No overflow into invalid ranges.

### AC-008: Layout relationship chain complete in master rels file
(traces to BC-4.01.005 postcondition 5 — each layout referenced from master)

`ppt/slideMasters/_rels/slideMaster1.xml.rels` contains exactly 31 entries with
`Type=".../slideLayout"`. A unit test parses this `.rels` file and asserts count == 31.

## Tasks

- [ ] Implement `SlideIdAssigner` in `src/slide_ids.rs`:
  - `fn assign(count: usize) -> Vec<u32>` — returns [256, 257, ..., 255+count]
  - `const MASTER_ID: u32 = 2_147_483_648`
- [ ] Update `PresentationSerializer` to use `SlideIdAssigner`
- [ ] Implement `LayoutEmbedder` in `src/layout_embedder.rs`:
  - Write all 31 layout XMLs from `Brand.layout_xmls` to `ppt/slideLayouts/slideLayout{N}.xml`
  - Write `ppt/slideLayouts/_rels/slideLayout{N}.xml.rels` for each layout
  - Update `slideMaster1.xml` `<p:sldLayoutIdLst>` with 31 entries
  - Update `slideMaster1.xml.rels` with 31 layout relationships
- [ ] Implement `ClrMapOvrInjector` in `src/clrmapovr.rs`:
  - Accept `dark_layout_indices: &[usize]` from Brand
  - For slides whose layout index is in `dark_layout_indices`, inject `<p:clrMapOvr>` after `<p:cSld>`
- [ ] Update `ContentTypesBuilder` to include all 31 layout Overrides
- [ ] Write unit tests for AC-001 through AC-008
- [ ] Write snapshot test: `slideMaster1.xml` `<p:sldLayoutIdLst>` with 31 entries

## Previous Story Intelligence

STORY-037 laid the ZIP skeleton and wrote placeholder notes/handout masters. This story:
1. Fixes slide ID assignment (STORY-037 may have used placeholder IDs; this story locks them to 256+)
2. Adds all 31 layout files (STORY-037 may only have written the first layout)
3. Adds `clrMapOvr` to dark-layout slides

Coordinate type: `Brand.dark_layout_indices: Vec<usize>` — if this field was not added
to `Brand` in STORY-023, add it in this story with a default of `[]` (no dark layouts).

## Architecture Compliance Rules

1. **Slide IDs start at exactly 256 (BC-4.01.005 invariant 1)**: Not 257, not 1, not 0.
   Exactly 256. The `SlideIdAssigner::assign()` function is the single code path for ID
   assignment — no ad-hoc ID generation elsewhere.
2. **Master ID is exactly 2^31 (BC-4.01.005 invariant 2)**: The constant
   `MASTER_ID = 2_147_483_648u32` is defined once and used everywhere. No other values.
3. **All 31 layouts always present (BC-4.01.005 invariant 3)**: `LayoutEmbedder` always
   writes all 31 layouts. No filtering based on which slide types are in the deck.
4. **clrMapOvr element ordering (brand-architecture.md §Dark Layout Invariant)**:
   `<p:clrMapOvr>` appears immediately after `<p:cSld>`, before `<p:transition>` and
   `<p:timing>`. ooxmlsdk enforces this.

## Library & Framework Requirements

Same as STORY-037 — no new external dependencies:

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | OOXML element construction |
| `zip` | `=4.2.0` | ZIP assembly (compatible with ooxmlsdk 0.6.1 dep) |
| `slideforge-types` | workspace | `Brand` struct (layout XML access) |
| `insta` | `=1.42.0` | Snapshot tests |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/src/slide_ids.rs` | Create | `SlideIdAssigner`, `MASTER_ID` constant |
| `crates/slideforge-pptx/src/layout_embedder.rs` | Create | 31-layout embedding |
| `crates/slideforge-pptx/src/clrmapovr.rs` | Create | Dark-layout `clrMapOvr` injection |
| `crates/slideforge-pptx/src/presentation.rs` | Modify | Use `SlideIdAssigner` |
| `crates/slideforge-pptx/src/content_types.rs` | Modify | Add all 31 layout Overrides |
| `crates/slideforge-pptx/src/tests/layout_tests.rs` | Create | AC-001 through AC-008 tests |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,000 |
| BC-4.01.005 | ~1,500 |
| STORY-037 code reference | ~2,000 |
| STORY-023 Brand struct | ~1,000 |
| Test files | ~2,500 |
| **Total** | **~10,000** |

## Test Strategy

- **Unit tests**: Slide ID sequence correctness (1-slide, 3-slide, 257-slide cases);
  layout count (always 31); master ID (always 2^31); clrMapOvr present/absent per
  dark_layout_indices.
- **Snapshot tests**: `slideMaster1.xml` sldLayoutIdLst; `presentation.xml` sldIdLst.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 1-slide deck | Slide ID = 256; all 31 layouts present |
| EC-002 | Brand has no dark layouts (`dark_layout_indices = []`) | No `<p:clrMapOvr>` in any slide |
| EC-003 | 257 slides | IDs 256..512; no collision |
| EC-004 | All 31 slide types used | 31 layouts embedded; slide master references all |

## Forbidden Dependencies

Same as STORY-037 — no sibling exporter deps, no upstream crate deps.
