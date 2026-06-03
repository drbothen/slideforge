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
It also absorbs PR #52 follow-ups S1–S3 and the STORY-038 scope items confirmed in
ADR-015 Addendum A (full per-slide-type layout selection, placeholder-inheritance
compliance, end-to-end `clrMapOvr` integration test, and unmapped DSL keyword
resolution for the 9 Q2 keywords without a dedicated SF custom layout).

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
(BC-4.01.005 invariant 3). The `BrandTemplate` struct from `slideforge-brand` (built in
STORY-037) provides all 31 layouts as structured `SlideLayoutDef` entries, serialized via
`slideforge_brand::layout_xml::serialize_layout_to_xml(&template.layouts[i])`.

**Important:** STORY-037 established via ADR-015 that `Brand` carries structured data,
NOT raw XML strings. There are no `Brand.layout_xmls`, `Brand.master_xml`, or
`Brand.theme_xml` fields. All layout XML bytes come from
`serialize_layout_to_xml(&brand_template.layouts[i])`, master XML from
`serialize_master_to_xml(&brand_template)`, and theme XML from
`serialize_theme_to_xml(&brand_template)`. See ADR-015 §1–§3 for the canonical API.

Layout file naming: `slideLayout1.xml` through `slideLayout31.xml`. The mapping from
DSL `slide_type_keyword` to layout index is determined at runtime by the two-phase
`find_layout_index` lookup built in STORY-037 (ADR-015 §A.6 obligation #2):

1. **Phase 1 — `SlideLayoutDef.slide_type_keyword` match** (SF custom layouts, indices 12–31):
   The `slide_type_keyword: Option<Arc<str>>` field added to `SlideLayoutDef` in STORY-037
   holds the canonical DSL keyword for each SF custom layout.

2. **Phase 2 — `ooxml_type` match** (standard OOXML layouts, indices 1–11):
   Used for DSL keywords that correspond to standard OOXML layout types.

The canonical DSL keyword → layout index table (from ADR-015 §A.4):

#### Standard OOXML Layouts (matched by `ooxml_type`, indices 1–11)

| Layout Index (1-based) | DSL `slide_type_keyword` | OOXML type |
|------------------------|--------------------------|------------|
| 1 | `title` | `"title"` (SL-01) |
| 2 | `content` | `"obj"` (SL-02) |
| 3 | (standard layout, no SF keyword) | `"titleOnly"` (SL-03) |
| 4 | `two_column` | `"twoObj"` (SL-04) |
| 5 | (standard layout, no SF keyword) | `"tx"` (SL-05) |
| 6 | (standard layout, no SF keyword) | `"picTx"` (SL-06) |
| 7 | (blank slide) | `"blank"` (SL-07) |
| 8 | `table` | `"objTx"` (SL-08) |
| 9–11 | (standard layouts, no SF keyword) | various |

#### SF Custom Layouts (matched by `SlideLayoutDef.slide_type_keyword`, indices 12–31)

| Layout Index (1-based) | DSL `slide_type_keyword` | `SlideLayoutDef.name` | Dark? |
|------------------------|--------------------------|----------------------|-------|
| 12 | `section_divider` | `"SF Section Divider"` | YES |
| 13 | `stat_callout` | `"SF Stat Grid"` | no |
| 14 | `quote` | `"SF Quote"` | no |
| 15 | `vertical_timeline` | `"SF Timeline"` | no |
| 16 | `agenda` | `"SF Agenda"` | no |
| 17 | `toc` | `"SF TOC"` | no |
| 18 | `bio` | `"SF Bio"` | no |
| 19 | `team` | `"SF Team Grid"` | no |
| 20 | `enhanced_table` | `"SF Comparison Table"` | no |
| 21 | `image` | `"SF Full-Bleed Image"` | no |
| 22 | `end` | `"SF End Slide"` | YES |
| 23 | `content_stat` | `"SF Data"` | no |
| 24 | `diagram` | `"SF Diagram"` | no |
| 25 | `chart` | `"SF Chart"` | no |
| 26 | `highlight` | `"SF Map"` | no |
| 27 | `severity_cards` | `"SF Risk Register"` | no |
| 28 | `highlight_boxes` | `"SF Executive Summary"` | no |
| 29 | `stats_summary` | `"SF Two Column"` | no |
| 30 | `numbered_actions` | `"SF Methodology"` | no |
| 31 | (appendix/overflow) | `"SF Appendix"` | no |

**Dark layout indices** (for `clrMapOvr`): 12 (`section_divider`) and 22 (`end`).
Determined by `SlideLayoutDef.has_color_override`. Once `find_layout_index` returns the
correct index, the existing `is_dark_layout` logic in `lib.rs:196-199` requires no change.

**Unmapped DSL keywords** (`split_contrast`, `card_rows`, `horizontal_timeline`, `status`,
`progress_bar`, `metric_tree`, `formula`, `weighted_composite`, `grid`): These 9 DSL
keywords from Q2 decisions do not have a dedicated SF custom layout slot. STORY-038 must
decide for each keyword whether: (a) an existing SF layout is a semantic match and can
take ownership of that keyword via `slide_type_keyword`, or (b) a new SF layout slot is
needed, or (c) the generic "Title and Content" fallback (index 2) is permanently correct
for that keyword. See AC-009 for the resolution obligation. Every fallback to index 2
MUST emit `tracing::warn!` naming the unmatched keyword (no-silent-fallback rule).

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
dark-on-light color inversion. Dark status is determined by `SlideLayoutDef.has_color_override`
on the layout resolved for that slide — there is no separate `dark_layout_indices` list on
`Brand` or `BrandTemplate`. Layouts 12 (`section_divider`) and 22 (`end`) have
`has_color_override = true` per the canonical table in ADR-015 §A.4.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-4.01.005 | Slide IDs start at 256; master IDs at 2^31; all 31 layouts present | AC-001 through AC-012 |

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

### AC-006: Slide using dark layout has clrMapOvr (end-to-end integration test)
(traces to BC-4.01.005 postcondition 1 — dark-layout clrMapOvr; ADR-015 §A.7 item 1)

A deck containing a slide with `slide_type_keyword = "section_divider"` (layout index 12,
dark per `SlideLayoutDef.has_color_override`) flows end-to-end through `export_inner` and
produces a PPTX ZIP where the corresponding `ppt/slides/slideN.xml` contains
`<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` — verified by reading the ZIP entry
directly (`ZipArchive::read`). A `title` slide (layout index 1, not dark) in the same
deck does NOT have `<p:clrMapOvr>`.

This is the full end-to-end integration test deferred from STORY-037 per ADR-015 §A.7
item 1 (F-PASS2-C2). It requires the full 31-layout embedding from AC-003 and the
corrected `find_layout_index` from STORY-037 to be in place. If the STORY-037 obligation
(ADR-015 §A.6 item #2: `section_divider` maps to index 11 in 0-based) is not yet landed,
this AC must un-ignore the `#[ignore]`-tagged test from STORY-037 as its first step.

Note: The dark layout indices are 12 (`section_divider`) and 22 (`end`) per the canonical
table in ADR-015 §A.4, NOT indices 6 and 18 as written in earlier drafts of this story.

### AC-007: 257-slide deck — all slide IDs unique and ≥ 256
(traces to BC-4.01.005 edge case EC-003)

A 257-slide deck produces slide IDs 256 through 512. A unit test verifies all IDs are
unique and all ≥ 256. No overflow into invalid ranges.

### AC-008: Layout relationship chain complete in master rels file
(traces to BC-4.01.005 postcondition 5 — each layout referenced from master)

`ppt/slideMasters/_rels/slideMaster1.xml.rels` contains exactly 31 entries with
`Type=".../slideLayout"`. A unit test parses this `.rels` file and asserts count == 31.

### AC-009: Unmapped DSL keywords resolved — no silent fallback to Title Slide (layout 0)
(traces to BC-4.01.005 postcondition 5; ADR-015 §A.4 unmapped-keyword obligation)

The 9 DSL keywords from Q2 decisions that lack a dedicated SF custom layout
(`split_contrast`, `card_rows`, `horizontal_timeline`, `status`, `progress_bar`,
`metric_tree`, `formula`, `weighted_composite`, `grid`) must have an explicit resolution:
either (a) an existing SF layout gains a `slide_type_keyword` assignment for that keyword,
(b) a new SF layout is added to the 31-slot table (requiring BC-4.01.005 invariant 3 update),
or (c) the keyword is permanently mapped to layout index 2 ("Title and Content", `ooxml_type
= "obj"`) with a `tracing::warn!` naming the unmatched keyword and the fallback index.

A unit test must assert that none of these 9 keywords map to layout index 0 (Title Slide).
The decision for each keyword must be documented in a code comment on the fallback branch.

### AC-010: Full per-slide-type layout selection correctness
(traces to BC-4.01.005 postcondition 5; ADR-015 §A.2 STORY-038 scope confirmation)

For each of the 25 DSL keywords that DO have a canonical layout mapping (11 standard +
the 20 SF custom, minus keywords addressed by AC-009): a unit test asserts
`find_layout_index(keyword, &brand_template)` returns the expected 0-based index from
the canonical table in ADR-015 §A.4. Key assertions per ADR-015 §A.6 item #2:
- `"section_divider"` → index 11 (0-based, 1-based = 12)
- `"end"` → index 21 (0-based, 1-based = 22)
- `"title"` → index 0 (0-based, 1-based = 1)
- unknown keyword → index 1 (0-based, 1-based = 2) with `tracing::warn!` (NOT index 0)

### AC-011: Placeholder inheritance idx chain verified during serialization
(traces to BC-4.01.005 postcondition 5; ADR-015 §7 placeholder inheritance contract)

When serializing each `LaidOutSlide` shape, the exporter looks up the matching
`SlideLayoutDef` in `brand_template.layouts` via the slide's layout index and confirms
the layout contains a `LayoutPlaceholder` with the expected `idx`. If no matching layout
placeholder exists, a `tracing::warn!` is emitted and the `<p:ph>` element is omitted
(shape becomes a non-placeholder shape). A unit test verifies: (a) a valid idx match
emits a `<p:ph idx="N">` element, and (b) a missing idx emits no `<p:ph>` element and
produces a `tracing::warn!` log event.

### AC-012: S2 — Silent i32 clamp on slide size replaced with structured error
(traces to BC-4.01.005 invariant 1; PR-52 follow-up S2; no-silent-fallback rule)

`presentation.rs` slide-size EMU-to-i32 conversion must not silently clamp on
out-of-range input. Replace `i32::try_from(emu).unwrap_or(9_144_000)` /
`unwrap_or(5_143_500)` with an explicit `PptxError::InvalidEmu` return (or, if
the design choice is deliberate saturation, add a `tracing::warn!` and document the
invariant in a code comment). A unit test exercises the out-of-range path and asserts
the structured error is returned (or the warning is emitted, per chosen resolution).

## Follow-Ups Absorbed from PR #52

The following PR #52 reviewer suggestions (S1–S3) are non-blocking items targeted at
STORY-038 per the STATE.md follow-ups table. They are incorporated as tasks below.
S4 (`build_notes_handout_masters` rels error swallowing) is targeted at STORY-040.

| ID | Description | AC/Task |
|----|-------------|---------|
| S1 | `validate_emu` Err path (slide_serializer.rs:275) untested — add negative-width unit test | Task item below |
| S2 | Silent i32 slide-size clamp (presentation.rs:159-160) at odds with no-silent-fallback | AC-012 |
| S3 | Subtitle frames map to `PlaceholderValues::Title` (idx 0), not `subTitle` placeholder | Task item below |

## Tasks

- [ ] Implement `SlideIdAssigner` in `src/slide_ids.rs`:
  - `fn assign(count: usize) -> Vec<u32>` — returns [256, 257, ..., 255+count]
  - `const MASTER_ID: u32 = 2_147_483_648`
- [ ] Update `PresentationSerializer` to use `SlideIdAssigner`
- [ ] Implement `LayoutEmbedder` in `src/layout_embedder.rs`:
  - Call `serialize_layout_to_xml(&brand_template.layouts[i])` for each of the 31 entries
    and write to `ppt/slideLayouts/slideLayout{N}.xml` (NOT `Brand.layout_xmls` — that
    field does not exist; see ADR-015 §1 for the canonical API)
  - Write `ppt/slideLayouts/_rels/slideLayout{N}.xml.rels` for each layout
  - Update `slideMaster1.xml` `<p:sldLayoutIdLst>` with 31 entries
  - Update `slideMaster1.xml.rels` with 31 layout relationships
- [ ] Implement `ClrMapOvrInjector` in `src/clrmapovr.rs`:
  - Use `SlideLayoutDef.has_color_override` (populated by STORY-037 `find_layout_index`)
    to determine dark layouts — NOT a hardcoded `dark_layout_indices: &[usize]` list
  - For slides whose resolved layout index corresponds to a `has_color_override = true`
    `SlideLayoutDef`, inject `<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>` after
    `<p:cSld>` and before `<p:transition>`/`<p:timing>`
- [ ] Implement AC-009 keyword resolution: decide per unmapped Q2 keyword whether it
  gets (a) an existing SF layout `slide_type_keyword` assignment, (b) a new layout slot,
  or (c) a permanent fallback to index 2 with `tracing::warn!`; document the decision
  in a code comment on the fallback branch; add unit test asserting no mapping returns
  index 0 for non-title keywords
- [ ] Implement AC-010 layout selection correctness: verify `find_layout_index` returns
  correct 0-based indices for all 25 mapped DSL keywords per ADR-015 §A.4 table
- [ ] Implement AC-011 placeholder inheritance idx chain verification in slide serializer;
  add `tracing::warn!` on missing idx; add unit tests for both match and no-match paths
- [ ] Fix AC-012 / PR-52 S2: replace silent i32 clamp in `presentation.rs:159-160` with
  `PptxError::InvalidEmu` (or documented deliberate saturation with `tracing::warn!`);
  add unit test for out-of-range path
- [ ] Fix PR-52 S1: add unit test for `validate_emu` `Err` path in `slide_serializer.rs`
  — inject a frame with negative width/height, assert `PptxError::InvalidEmu` is returned
- [ ] Fix PR-52 S3: resolve `Subtitle` placeholder mapping — either map to
  `PlaceholderValues::SubTitle` (OOXML `subTitle` type) with `idx=1` on the layout, or
  add an explicit code comment confirming `Title` (idx=0) is intentional for this pipeline
  and explaining why; add a test that asserts the chosen behavior
- [ ] Update `ContentTypesBuilder` to include all 31 layout Overrides
- [ ] Write unit tests for AC-001 through AC-012
- [ ] Write snapshot test: `slideMaster1.xml` `<p:sldLayoutIdLst>` with 31 entries
- [ ] Un-ignore the `#[ignore]`-tagged STORY-037 dark-layout test
  (`test_f037_011_layout_index_is_wired_not_dead_code`) and strengthen it to assert
  `<p:clrMapOvr>` presence in the exported ZIP (per ADR-015 §A.6 item #5 / AC-006)

## Previous Story Intelligence

STORY-037 (merged PR #52, 2ebf184f) laid the ZIP skeleton, brand-rendering boundary
(ADR-015), and the following contracts that this story builds on:

1. **`serialize_layout_to_xml` / `serialize_master_to_xml` / `serialize_theme_to_xml`**
   are implemented in `slideforge-brand`. This story calls them — it does NOT reinvent
   layout XML generation or use any `Brand.layout_xmls` / `Brand.master_xml` raw fields
   (those fields do not exist on `Brand`; see ADR-015 §1–§3).
2. **`SlideLayoutDef.slide_type_keyword: Option<Arc<str>>`** was added to `SlideLayoutDef`
   in STORY-037 (ADR-015 §A.6 item #1). This story relies on it for `find_layout_index`.
3. **`find_layout_index`** was corrected in STORY-037 to use the two-phase lookup
   (ADR-015 §A.6 item #2). This story extends the mapping for the 9 unmapped keywords
   (AC-009) and adds full per-slide-type coverage tests (AC-010).
4. **Slide IDs** — STORY-037 set the foundation; this story enforces the exact sequence
   starting at 256 with dedicated `SlideIdAssigner` and unit tests.
5. **Dark layout determination** uses `SlideLayoutDef.has_color_override` (set in
   `generate_all_layouts` for layouts 12 and 22). There is no `Brand.dark_layout_indices`
   field — dark status is intrinsic to the `SlideLayoutDef`, not a separate list.

PR #52 follow-ups S1 (validate_emu Err path), S2 (silent i32 clamp), S3 (Subtitle
placeholder) are absorbed into this story's scope — see the "Follow-Ups Absorbed from
PR #52" section and AC-012 / Task items.

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
5. **Brand rendering boundary (ADR-015)**: Layout XML bytes come ONLY from
   `serialize_layout_to_xml(&brand_template.layouts[i])`. Master XML from
   `serialize_master_to_xml(&brand_template)`. Theme XML from
   `serialize_theme_to_xml(&brand_template)`. No inline XML string construction for
   these parts. `Brand` (in `slideforge-types`) carries structured data, NOT raw XML.
6. **No-silent-fallback on layout lookup (ADR-015 §A.4)**: Unknown DSL keywords must
   NOT silently fall back to layout index 0 (Title Slide). They fall back to index 1
   (0-based; "Title and Content") with a `tracing::warn!` naming the keyword.
7. **Forbidden dependencies (unchanged from STORY-037)**: `slideforge-pptx` MUST NOT
   call layout computation functions (e.g., `slideforge_layout::layout::run`). It MAY
   depend on `slideforge-layout` for `LaidOutDeck`, `LaidOutSlide`, `LaidOutFrame` types
   (ADR-015 §6 ruling). `slideforge-pptx` MUST depend on `slideforge-brand` for brand
   serialization APIs (ADR-015 §5).

## Library & Framework Requirements

Same as STORY-037 — no new external dependencies. ADR-015 §5–§6 established that
`slideforge-pptx` depends on both `slideforge-brand` and `slideforge-layout`; these
must be present in `Cargo.toml` (added in STORY-037).

| Library | Version | Purpose |
|---------|---------|---------|
| `ooxmlsdk` | `=0.6.1` | OOXML element construction |
| `zip` | `=4.2.0` | ZIP assembly (compatible with ooxmlsdk 0.6.1 dep) |
| `slideforge-brand` | workspace | `BrandTemplate`, `serialize_layout_to_xml`, `serialize_master_to_xml`, `serialize_theme_to_xml` (ADR-015 §1–§3) |
| `slideforge-layout` | workspace | `LaidOutDeck`, `LaidOutSlide`, `LaidOutFrame` input types (ADR-015 §6) |
| `slideforge-types` | workspace | `Brand` struct (structured brand data, NOT raw XML) |
| `insta` | `=1.42.0` | Snapshot tests |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-pptx/src/slide_ids.rs` | Create | `SlideIdAssigner`, `MASTER_ID` constant |
| `crates/slideforge-pptx/src/layout_embedder.rs` | Create | 31-layout embedding via `serialize_layout_to_xml` (ADR-015 §1) |
| `crates/slideforge-pptx/src/clrmapovr.rs` | Create | Dark-layout `clrMapOvr` injection driven by `SlideLayoutDef.has_color_override` |
| `crates/slideforge-pptx/src/presentation.rs` | Modify | Use `SlideIdAssigner`; fix silent i32 clamp (AC-012 / PR-52 S2) |
| `crates/slideforge-pptx/src/slide_serializer.rs` | Modify | Fix `validate_emu` Err path test (PR-52 S1); fix Subtitle placeholder (PR-52 S3); add placeholder idx chain verification (AC-011) |
| `crates/slideforge-pptx/src/content_types.rs` | Modify | Add all 31 layout Overrides |
| `crates/slideforge-pptx/src/tests/layout_tests.rs` | Create | AC-001 through AC-012 tests + PR-52 S1–S3 |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~4,500 |
| BC-4.01.005 | ~1,500 |
| ADR-015 + Addendum A | ~3,000 |
| STORY-037 code reference (`lib.rs`, `slide_serializer.rs`, `presentation.rs`) | ~3,000 |
| `slideforge-brand` layout/master/theme serializers | ~2,000 |
| Test files (AC-001 through AC-012) | ~3,500 |
| **Total** | **~17,500** |

Budget is within 20% of the 100k-token implementing agent context window. No split required.

## Test Strategy

- **Unit tests**: Slide ID sequence correctness (1-slide, 3-slide, 257-slide cases);
  layout count (always 31); master ID (always 2^31); `clrMapOvr` present for
  `section_divider`/`end` slides (driven by `has_color_override`), absent for all others;
  `find_layout_index` correctness for all 25 mapped + 9 unmapped keywords (AC-009/010);
  placeholder idx chain match and no-match paths (AC-011); `validate_emu` Err path (PR-52 S1);
  slide-size out-of-range path (AC-012 / PR-52 S2); Subtitle placeholder resolution (PR-52 S3).
- **End-to-end integration test**: Full export of a multi-slide deck (title + section_divider +
  content) with ZIP entry inspection for `<p:clrMapOvr>` (AC-006).
- **Snapshot tests**: `slideMaster1.xml` sldLayoutIdLst; `presentation.xml` sldIdLst.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 1-slide deck | Slide ID = 256; all 31 layouts present |
| EC-002 | Brand has no dark layouts (all `SlideLayoutDef.has_color_override = false`) | No `<p:clrMapOvr>` in any slide |
| EC-003 | 257 slides | IDs 256..512; no collision |
| EC-004 | All 31 slide types used | 31 layouts embedded; slide master references all |
| EC-005 | Slide with unmapped DSL keyword (`grid`, `status`, etc.) | Falls back to layout index 1 ("Title and Content"); `tracing::warn!` emitted naming keyword; NOT layout index 0 |
| EC-006 | `validate_emu` called with negative bbox dimensions | Returns `PptxError::InvalidEmu`; does not panic |
| EC-007 | Slide-size EMU outside i32 range | Returns `PptxError::InvalidEmu` (or documented saturation with logged warning); does not silently clamp |

## Forbidden Dependencies

Same as STORY-037 — no sibling exporter deps, no upstream crate deps.
