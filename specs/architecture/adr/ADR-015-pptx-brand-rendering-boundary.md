---
document_type: adr
adr_id: ADR-015
title: "PPTX brand rendering boundary: slideforge-brand owns layout/master/theme XML; slideforge-pptx calls brand APIs"
status: accepted
date: 2026-06-03
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
story_context: "STORY-037 adversarial review finding — Task 7 spec ambiguity resolved"
---

# ADR-015: PPTX Brand Rendering Boundary

## Context

STORY-037 (PPTX Core Serialization) Task 7 contained the instruction:

> "Import `Brand.master_xml`, `Brand.layout_xmls[0..31]`, `Brand.theme_xml` verbatim."

This instruction is incorrect. The `Brand` type in `slideforge-types` (v1.0 at
`crates/slideforge-types/src/brand.rs:63`) carries **structured data** — colors,
fonts, and `Vec<LayoutDefinition>` — not raw OOXML strings. No `master_xml`,
`layout_xmls`, or `theme_xml` fields exist on `Brand`.

The actual rendering-ready struct is `BrandTemplate` in `slideforge-brand`
(`crates/slideforge-brand/src/template.rs:323`), which carries:

- `layouts: Vec<SlideLayoutDef>` — structured layout definitions (31 entries for
  synthesized brands), serializable to OOXML via `serialize_layout_to_xml`
- `colors: [ColorSlot; 12]` — 12 OOXML scheme color slots for theme XML rendering
- `fonts: BrandFonts` — heading/body font names for theme XML rendering
- `notes_master_stub: Vec<u8>` — pre-serialized `notesMaster1.xml` bytes
- `handout_master_stub: Vec<u8>` — pre-serialized `handoutMaster1.xml` bytes
- `master_ids: MasterIds` — OOXML-mandated master/layout/slide ID constants
- `content_types_layout_entries: Arc<str>` — pre-generated `[Content_Types].xml`
  layout Override fragment

The implementer responded to the missing raw-XML fields by using empty
`SlideMaster::default()` / `SlideLayout::default()` shells and a hardcoded theme.
The adversary correctly flagged this as CRITICAL spec-drift: brand chrome dropped,
placeholder inheritance structurally broken, master schema-invalid.

Separately, the story's Forbidden Dependencies list bans `slideforge-pptx →
slideforge-layout`, yet `LaidOutDeck` (the exporter's primary input type) resides
in `slideforge-layout::types`, not `slideforge-types`. This was also identified as
an error requiring resolution.

## Decision

### 1. Layout parts — existing brand API is the correct call site

`slideforge-pptx` MUST generate the 31 `slideLayoutN.xml` parts by calling:

```rust
slideforge_brand::layout_xml::serialize_layout_to_xml(&brand_template.layouts[i])
```

This is the canonical API. It takes a `&SlideLayoutDef` and returns `Vec<u8>` — a
complete, schema-correct `slideLayoutN.xml` document. It is already implemented,
tested, and publicly exported from `slideforge-brand`.

- File: `crates/slideforge-brand/src/layout_xml.rs:89`
- Public via: `slideforge_brand::layout_xml::serialize_layout_to_xml`

The exporter iterates `brand_template.layouts` (guaranteed 31 entries for synthesized
brands; invariant documented on `BrandTemplate::layouts`) and writes
`ppt/slideLayouts/slideLayout{N}.xml` for N in 1..=31.

### 2. Master part — new `serialize_master_to_xml` function added to slideforge-brand

No master OOXML renderer exists in `slideforge-brand` today. The decision is:

**(a) Add `serialize_master_to_xml` to `slideforge-brand`** — not (b) generate in
`slideforge-pptx`, not (c) a new story.

Rationale: The master XML renderer requires the same `BrandTemplate` data that
`serialize_layout_to_xml` already uses (color slots, fonts, layout IDs). Placing the
renderer in `slideforge-brand` maintains the invariant that all OOXML XML generation
from brand data lives in one crate. The pptx exporter remains a pure consumer of
pre-rendered bytes — the same pattern as layouts and notes/handout master stubs. This
is consistent with the design documented in `crates/slideforge-brand/src/lib.rs:34`:
"Layout XML serialization is performed lazily by the PPTX exporter (STORY-037), which
calls `layout_xml::serialize_layout_to_xml` per layout." Master XML serialization
follows the same lazy pattern.

**Scope of master XML:** `slideMaster1.xml` must contain:
- `<p:sldLayoutIdLst>` with one `<p:sldLayoutId id="...">` per layout
  (IDs from `MasterIds::layout_id_start` through `layout_id_start + 30`)
- `<a:clrMap>` with all 12 color map tokens in ECMA-376 order
- `<p:txStyles>` with placeholder text style defaults (body, title, other)
- One `<p:sp>` per master placeholder type (`title`, `body`, `dt`, `ftr`, `sldNum`)
  with canonical position/size geometry
- `<p:hf>` header/footer flags (from `BrandTemplate::footer_flags`)

The new function signature:

```rust
/// Serialize `BrandTemplate` data to a complete `slideMaster1.xml` document.
pub fn serialize_master_to_xml(template: &BrandTemplate) -> Vec<u8>
```

This function is to be implemented in `crates/slideforge-brand/src/layout_xml.rs`
(alongside `serialize_layout_to_xml`), or in a new `crates/slideforge-brand/src/master_xml.rs`
module if size warrants separation.

**This work is within STORY-037's scope** (see §8 below — scope verdict).

### 3. Theme part — new `serialize_theme_to_xml` function added to slideforge-brand

`slideforge-brand` has `parse_theme_colors` and `parse_theme_fonts` (read paths) but no
write path. The decision is:

**(a) Add `serialize_theme_to_xml` to `slideforge-brand`** — same rationale as #2.

The theme XML renderer requires `BrandTemplate::colors` (all 12 color slots) and
`BrandTemplate::fonts` (heading/body typeface names), both already present on
`BrandTemplate`. The generated `theme1.xml` must conform to ECMA-376 §20.1.6.9
(`<a:theme>`) with a `<a:fmtScheme>`, `<a:fontScheme>`, and `<a:clrScheme>` containing
all 12 slots in the order mandated by `COLOR_SLOT_NAMES`.

Function signature:

```rust
/// Serialize `BrandTemplate` color and font data to a complete `theme1.xml` document.
pub fn serialize_theme_to_xml(template: &BrandTemplate) -> Vec<u8>
```

The `slideforge-pptx` exporter calls this once per deck and writes the result to
`ppt/theme/theme1.xml`.

### 4. notesMaster1.xml / handoutMaster1.xml

Both are already solved. `BrandTemplate::notes_master_stub` and
`BrandTemplate::handout_master_stub` hold pre-serialized `Vec<u8>` bytes populated by
`BrandSynthesizer::synthesize` from the constants:

- `slideforge_brand::layout_xml::NOTES_MASTER_STUB` — `crates/slideforge-brand/src/layout_xml.rs:40`
- `slideforge_brand::layout_xml::HANDOUT_MASTER_STUB` — `crates/slideforge-brand/src/layout_xml.rs:52`

`slideforge-pptx` writes `brand_template.notes_master_stub` to
`ppt/notesMasters/notesMaster1.xml` and `brand_template.handout_master_stub` to
`ppt/handoutMasters/handoutMaster1.xml`. No new API required.

### 5. Dependency graph — `slideforge-pptx → slideforge-brand` is correct and required

`slideforge-pptx` MUST depend on `slideforge-brand` to call:
- `slideforge_brand::layout_xml::serialize_layout_to_xml`
- `slideforge_brand::layout_xml::serialize_master_to_xml` (new, per decision 2)
- `slideforge_brand::layout_xml::serialize_theme_to_xml` (new, per decision 3)
- `slideforge_brand::layout_xml::NOTES_MASTER_STUB` / `HANDOUT_MASTER_STUB` (via
  `BrandTemplate` fields — no direct import needed)
- `slideforge_brand::template::BrandTemplate` as the brand input type

This dependency is NOT present in `dependency-graph.md` and is added by this ADR (see
dependency-graph update below).

No cycle risk: `slideforge-brand/src/lib.rs:56` explicitly forbids `slideforge-brand`
from depending on `slideforge-pptx` (Architecture Compliance Rule 5, STORY-022).
`slideforge-brand` also does not depend on `slideforge-layout` or `slideforge-eval`.
The new edge `pptx → brand` is acyclic.

**`Cargo.toml` change required:** Add to `crates/slideforge-pptx/Cargo.toml`:
```toml
slideforge-brand = { workspace = true }
```

### 6. `LaidOutDeck` lives in `slideforge-layout` — Forbidden Dependencies ruling

`LaidOutDeck` is defined in `crates/slideforge-layout/src/types.rs:122`. It is NOT in
`slideforge-types`. This means the STORY-037 Forbidden Dependencies list as written
(`slideforge-pptx` must NOT depend on `slideforge-layout`) creates a logical contradiction:
the exporter's primary input type lives in the crate it is forbidden to depend on.

**Ruling:** The Forbidden Dependencies prohibition on `slideforge-pptx →
slideforge-layout` is REVOKED for the `LaidOutDeck` / `LaidOutSlide` / `LaidOutFrame`
types specifically. `slideforge-pptx` MAY depend on `slideforge-layout` for the
purpose of consuming `LaidOutDeck` and related IR types.

**Rationale:** The prohibition's original intent was to prevent the exporter from
calling layout logic (which would break the Two-IR model invariant: exporters consume
`LaidOutDeck` but do not re-run layout). That intent is preserved — the exporter calls
zero layout computation functions. It merely receives the `LaidOutDeck` output type.
There is no design alternative: `LaidOutDeck` is defined in `slideforge-layout` and
cannot be moved to `slideforge-types` without breaking the purity boundary (it carries
`LayoutWarning` variants that reference layout-specific error types).

**The prohibition REMAINS in effect for:** calling `slideforge_layout::layout::run` or
any other layout computation function from within `slideforge-pptx`. The exporter
receives a `&LaidOutDeck` as a parameter — it never constructs one.

**`Cargo.toml` change required:** Add to `crates/slideforge-pptx/Cargo.toml`:
```toml
slideforge-layout = { workspace = true }
```

**Story Forbidden Dependencies list correction:** STORY-037's prohibition
"slideforge-pptx must NOT depend on slideforge-layout" must be amended to read:
"slideforge-pptx must NOT call slideforge-layout computation functions (e.g.,
`layout::run`). It MAY depend on slideforge-layout for `LaidOutDeck`, `LaidOutSlide`,
and associated IR types."

### 7. Placeholder inheritance contract

For placeholder inheritance to function correctly, three conditions must hold:

1. **Layout XMLs must contain matching `<p:ph idx="...">` elements.** The layouts
   produced by `serialize_layout_to_xml` already do this:
   `write_placeholder` at `crates/slideforge-brand/src/layout_xml.rs:251` emits a
   `<p:ph type="..." idx="...">` for every `LayoutPlaceholder` in the layout definition.
   Every non-Blank layout is tested to have at least one title-type placeholder
   (test at `crates/slideforge-brand/src/layouts.rs:1037`).

2. **Master XML must contain matching `<p:ph type="...">` elements.** The new
   `serialize_master_to_xml` function (decision 2) must produce master placeholder
   shapes for each of: `type="title"`, `type="body"`, `type="dt"`, `type="ftr"`,
   `type="sldNum"`. Slide/layout placeholders that reference a type that is absent
   from the master will silently inherit nothing — producing unstyled output in
   PowerPoint/Keynote.

3. **slideforge-pptx must verify the idx chain during serialization.** When serializing
   each `LaidOutSlide` shape, the exporter MUST:
   - Look up the slide's layout index (from `LaidOutSlide`) to find the matching
     `SlideLayoutDef` in `brand_template.layouts`
   - Confirm the layout contains a `LayoutPlaceholder` with the expected `idx`
   - If no matching layout placeholder exists, emit a `tracing::warn!` and fall back
     to omitting the `<p:ph>` element (shape becomes a non-placeholder shape)
   - This verification is a runtime check, not a hard error — a missing layout
     placeholder is a brand authoring issue, not a fatal export failure

The geometric chain is: slide `<p:spPr><a:xfrm>` provides explicit EMU coordinates
(from `LaidOutFrame.position` and `.size`). The layout placeholder provides fallback
geometry if the slide omits `<a:xfrm>`. The master provides style defaults only.
Because `slideforge-layout` always populates explicit EMU coordinates on every
`LaidOutFrame`, the slide XML will always have explicit `<a:xfrm>` — layout geometry
fallback is not exercised in the current pipeline. Nonetheless the placeholder `idx`
chain must be structurally correct for applications that inspect it (accessibility
tools, screen readers, template editing).

### 8. Scope verdict — in STORY-037

The work required to fix the brand-rendering gap is:

1. Add `serialize_master_to_xml(&BrandTemplate) -> Vec<u8>` to `slideforge-brand`
2. Add `serialize_theme_to_xml(&BrandTemplate) -> Vec<u8>` to `slideforge-brand`
3. Add `slideforge-brand` and `slideforge-layout` to `slideforge-pptx/Cargo.toml`
4. Update `slideforge-pptx` Task 7 implementation to call brand APIs instead of using
   default shells

Items 1–2 are self-contained pure functions with no I/O, following the exact same
pattern as the existing `serialize_layout_to_xml`. They each require approximately
100–150 lines of `quick_xml` code plus tests, closely paralleling the existing layout
serializer. Item 3 is a two-line `Cargo.toml` change. Item 4 is the replacement of
the incorrect default-shell approach.

**Verdict: all four items fit within STORY-037's 13-point scope.** The story's
existing Task 7 ("Master + layout + theme embedding") and Task 9 ("Write unit tests")
already budget for this work. The incorrect "verbatim" language was a spec error, not
a scope addition. No new story is required.

**If the implementer determines that the master XML complexity (txStyles, font
theme embedding, 5 master placeholder types) is substantially larger than estimated,
they must surface this to the orchestrator before deferring** — not silently defer
to a default shell.

## Consequences

### Positive

- Brand chrome is fully preserved in generated PPTX (colors, fonts, layout geometry)
- Placeholder inheritance is structurally correct across all 31 layouts
- The master and theme XMLs are generated from the same `BrandTemplate` data already
  used by the layout serializer — no new data sources required
- No raw XML string fields need to be added to `Brand` or `BrandTemplate`
- The purity boundary is preserved: all brand-to-XML serialization lives in
  `slideforge-brand`; `slideforge-pptx` consumes bytes, not brand logic

### Negative

- `slideforge-pptx` gains a dependency on `slideforge-brand` (previously absent from
  `Cargo.toml`) and on `slideforge-layout` (previously prohibited)
- The `serialize_master_to_xml` and `serialize_theme_to_xml` functions must be written
  in STORY-037 scope, increasing implementation effort slightly

### Neutral

- The STORY-037 Forbidden Dependencies list requires a targeted amendment (the
  `slideforge-layout` prohibition is narrowed, not removed)
- `LaidOutDeck` stays in `slideforge-layout`; no type migration required

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-03 | Route master/theme XML generation to slideforge-brand | Purity boundary consistency; brand data lives in brand crate |
| 2026-06-03 | Revoke pptx→layout prohibition for IR types only | LaidOutDeck is in slideforge-layout; prohibition was structurally self-contradictory |
| 2026-06-03 | Classify as in-STORY-037 scope | Items 1–4 fit Task 7 + Task 9 budget; no new story warranted |
| 2026-06-03 | Addendum A — layout-selection scope boundary and mapping mechanism | Adversary pass 2 findings F-PASS2-C1/C2/M1/M2/H1; see §Addendum A below |

---

## Addendum A — Layout Selection Scope Boundary and Mapping Mechanism

**Date:** 2026-06-03
**Triggered by:** STORY-037 adversary pass 2 findings F-PASS2-C1 (CRITICAL), F-PASS2-C2
(CRITICAL), F-PASS2-H1 (HIGH), F-PASS2-M1 (MEDIUM), F-PASS2-M2 (MEDIUM).
**Binding on:** STORY-037 implementer (current), STORY-038 implementer (next).

### A.1 Root Cause of the Vocabulary Mismatch

Three distinct vocabularies exist for slide type identification:

1. **DSL slide-type keywords** (`LaidOutSlide.slide_type_keyword`): lowercase
   snake_case as declared in Q2 decisions — `title`, `content`, `two_column`,
   `stat_callout`, `section_divider` (hypothetical), `end`, etc. These are the
   authoritative user-visible names.

2. **`SlideLayoutDef.name`** (display names in `slideforge-brand`): human-readable
   strings like `"Title Slide"`, `"SF Section Divider"`, `"SF End Slide"`. These are
   OOXML presentation names, not DSL keywords.

3. **STORY-038 mapping table** (§"All 31 Layouts Present"): uses `title_slide`,
   `section_divider`, etc. — a third vocabulary, neither DSL nor display-name.
   **This table is stale and incorrect.** It was written before the DSL keyword list
   was finalized in Q2 and does not match vocabulary #1 or #2 above.

`find_layout_index` at `crates/slideforge-pptx/src/lib.rs:254` attempts to match
vocabulary #1 against vocabulary #2. They never match because `"section_divider"
!= "SF Section Divider"`, `"content" != "Title and Content"`, etc. The result is
that every slide silently falls back to layout index 0 (Title Slide layout).

### A.2 Scope Boundary Ruling

**STORY-037 scope (Core Serialization):**
- Must NOT ship `find_layout_index` with a lookup that is structurally guaranteed
  to fail (current state).
- Must implement a **minimal correct mapping** using `ooxml_type` for the 11
  standard layouts and an explicit keyword-to-layout-index function for the 20 custom
  SF layouts — sufficient to make the dark-layout wiring (F-PASS2-C2) exercisable
  through the real export path, not a bypassed unit test.
- Specifically: `SlideLayoutDef` MUST gain a `slide_type_keyword: Option<Arc<str>>`
  field, and `generate_all_layouts` MUST populate it with the canonical Q2 DSL
  keywords. This field is the **source of truth** for keyword-to-layout lookup.
  See §A.4 for the canonical mapping table.
- The `brand_adapter::brand_template_from_brand` function that silently discards
  `Brand.layouts` (F-PASS2-M1) is a STORY-037 defect if `Brand.layouts` carries
  non-empty layout data; the adapter MUST either use `Brand.layouts` as a source
  of overrides or document why the synthesized layouts take precedence with a
  `tracing::debug!`. It MUST NOT silently lose data without a documented invariant.
  See §A.5.
- The master XML element order (F-PASS2-H1) is a STORY-037 serializer defect and
  MUST be fixed in STORY-037 scope. See §A.3.

**STORY-038 scope (Layout Compliance):**
- Full per-slide-type layout selection correctness is CONFIRMED as STORY-038
  responsibility when it requires coordinating with `Brand.dark_layout_indices`
  or with the full 31-layout embedding pass.
- The STORY-038 mapping table (§"All 31 Layouts Present") MUST be replaced with
  the canonical keyword-to-index table from §A.4.
- Full `clrMapOvr` integration test (a slide with `slide_type_keyword` matching a
  known-dark layout keyword flowing end-to-end through `export_inner` and producing
  `<p:clrMapOvr>` in the output ZIP) belongs to STORY-038 AC-006, not STORY-037.
  F-PASS2-C2 adversary finding is therefore a **legitimate cross-story deferral**
  from STORY-037 to STORY-038, SUBJECT to §A.2 STORY-037 obligations being met
  (i.e., the lookup must not be structurally broken before STORY-038 runs).

### A.3 Master XML Element Order (F-PASS2-H1) — STORY-037 Non-Deferrable Fix

ECMA-376 §19.3.1.42 (`CT_SlideMaster`) specifies the sequence model for
`<p:sldMaster>` children. The correct order is:

```
cSld, clrMap, sldLayoutIdLst, hf, txStyles
```

The current implementation in `crates/slideforge-brand/src/layout_xml.rs:696-700`
emits `txStyles` BEFORE `hf`:

```rust
// <p:txStyles> — heading and body font definitions
write_master_tx_styles(&mut writer, template);

// <p:hf> — footer visibility flags
write_master_hf(&mut writer, &template.footer_flags);
```

This is schema-invalid. PowerPoint/Keynote process children in sequence-model
order; a wrong-ordered `<p:txStyles>` either causes a repair dialog or silently
drops txStyles content.

**Fix required in STORY-037:** Swap the two calls so `hf` is emitted before
`txStyles`. Also update the `serialize_master_to_xml` docstring at line 602-603
to reflect the corrected order:

```
/// - `<p:hf>` footer visibility flags from `template.footer_flags`.
/// - `<p:txStyles>` with heading/body font names from `template.fonts`.
```

The fix is a two-line swap plus a docstring update. No new API, no new test (existing
master XML tests will catch the ordering implicitly once snapshot tests are added;
an explicit element-order assertion must be added as part of this fix to satisfy
TD-VSDD-059 — paper-fix detection requires a load-bearing test).

### A.4 Canonical DSL Keyword → Layout Index Mapping

**Owner: STORY-037** adds `slide_type_keyword: Option<Arc<str>>` to `SlideLayoutDef`.
**Owner: STORY-038** replaces its stale mapping table with this table.

`SlideLayoutDef.slide_type_keyword` is `None` for standard OOXML layouts (indices 1–11)
that have no direct DSL slide-type keyword equivalent (they are referenced by OOXML
type, not by user keyword). It is `Some(keyword)` for every SF custom layout.

For the 11 standard layouts, `find_layout_index` matches by `ooxml_type`:

| DSL keyword | OOXML type match | Layout index (1-based) |
|-------------|-----------------|----------------------|
| `title` | `"title"` (ooxml_type SL-01) | 1 |
| `content` | `"obj"` (ooxml_type SL-02) | 2 |
| `two_column` | `"twoObj"` (ooxml_type SL-04) | 4 |
| `table` | `"objTx"` (ooxml_type SL-08) | 8 |
| (blank slide) | `"blank"` (ooxml_type SL-07) | 7 |

For the 20 SF custom layouts, `slide_type_keyword` is set directly:

| DSL keyword | `SlideLayoutDef.name` | Layout index (1-based) |
|-------------|----------------------|----------------------|
| `section_divider` | `"SF Section Divider"` (dark) | 12 |
| `stat_callout` | `"SF Stat Grid"` | 13 |
| `quote` | `"SF Quote"` | 14 |
| `vertical_timeline` | `"SF Timeline"` | 15 |
| `agenda` | `"SF Agenda"` | 16 |
| `toc` | `"SF TOC"` | 17 |
| `bio` | `"SF Bio"` | 18 |
| `team` | `"SF Team Grid"` | 19 |
| `enhanced_table` | `"SF Comparison Table"` | 20 |
| `image` | `"SF Full-Bleed Image"` | 21 |
| `end` | `"SF End Slide"` (dark) | 22 |
| `content_stat` | `"SF Data"` | 23 |
| `diagram` | `"SF Diagram"` | 24 |
| `chart` | `"SF Chart"` | 25 |
| `highlight` | `"SF Map"` | 26 |
| `severity_cards` | `"SF Risk Register"` | 27 |
| `highlight_boxes` | `"SF Executive Summary"` | 28 |
| `stats_summary` | `"SF Two Column"` | 29 |
| `numbered_actions` | `"SF Methodology"` | 30 |
| (appendix/overflow) | `"SF Appendix"` | 31 |

**Note:** Several DSL keywords from Q2 (`split_contrast`, `card_rows`,
`horizontal_timeline`, `status`, `progress_bar`, `metric_tree`, `formula`,
`weighted_composite`, `grid`) do not have a dedicated custom layout slot in the
current 20 SF custom layouts. The correct fallback for these is layout index 2
("Title and Content", `ooxml_type = "obj"`) — a generic content layout, NOT layout
index 0 (Title Slide). This fallback MUST emit a `tracing::warn!` naming the
unmatched keyword and the fallback index, satisfying the no-silent-fallback rule.

The architecture decision is: **STORY-038 assigns any DSL keywords that currently
lack a dedicated SF custom layout to the generic "Title and Content" layout (index 2)
with a logged warning.** If a future story adds new SF layouts for those types, it
adds the `slide_type_keyword` entries and removes the fallback warning.

The dark layout indices (for `clrMapOvr`) are: 12 (`section_divider`) and 22 (`end`).
These are fully determined by `SlideLayoutDef.has_color_override`, which is already
set correctly. Once `find_layout_index` returns the correct index, the existing
`is_dark_layout` logic in `lib.rs:196-199` requires no change.

### A.5 `brand_template_from_brand` and `Brand.layouts` (F-PASS2-M1)

`Brand` in `slideforge-types` carries `layouts: Vec<LayoutDefinition>` (or equivalent).
`brand_template_from_brand` in `crates/slideforge-pptx/src/brand_adapter.rs` builds a
`BrandTemplate` by calling `generate_all_layouts` from scratch, ignoring whatever is
in `Brand.layouts`.

**Ruling:** This is NOT a silent data-loss defect if — and only if — `Brand.layouts`
is always empty at the point `export` is called (i.e., the synthesizer fills
`BrandTemplate.layouts` not `Brand.layouts`). The implementer MUST add an assertion or
`debug_assert!` at the top of `brand_template_from_brand`:

```rust
debug_assert!(
    brand.layouts.is_empty(),
    "brand_template_from_brand: Brand.layouts is non-empty ({} entries) \
     but brand_adapter synthesizes layouts from scratch; non-empty Brand.layouts \
     will be ignored. If this is intentional, remove this assertion and document why.",
    brand.layouts.len()
);
```

If `Brand.layouts` is already a `Vec` with 31 populated entries at export time (i.e.,
the synthesizer puts layout data in `Brand`, not `BrandTemplate`), then
`brand_template_from_brand` is discarding those entries and is a STORY-037 defect
requiring a fix in scope. The implementer must verify which case applies and act
accordingly. This check MUST be added in STORY-037 scope.

### A.6 Summary of STORY-037 Non-Deferrable Obligations

The following items are in STORY-037 scope and MUST NOT be deferred to STORY-038:

1. **Add `slide_type_keyword: Option<Arc<str>>` field to `SlideLayoutDef`** in
   `crates/slideforge-brand/src/layouts.rs`. Populate with the canonical DSL keyword
   for each SF custom layout per the table in §A.4.

2. **Replace `find_layout_index` lookup logic** in
   `crates/slideforge-pptx/src/lib.rs:254`. The new implementation must:
   - First match by `SlideLayoutDef.slide_type_keyword` (for SF custom layouts).
   - Second match by `ooxml_type` string for the ~5 standard OOXML layouts that
     correspond to DSL keywords.
   - Fall back to layout index 1 (0-based: index 1 = "Title and Content") on no match,
     with a `tracing::warn!` that names the unmatched keyword. Layout index 0 ("Title
     Slide") must NOT be the fallback for non-title content slides.
   - The function must have a unit test that asserts: (a) `"section_divider"` maps to
     index 11 (0-based), (b) `"end"` maps to index 21 (0-based), (c) `"title"` maps
     to index 0 (0-based), (d) an unknown keyword maps to index 1 (0-based) with a
     warning (not index 0).

3. **Fix master XML element order** in `serialize_master_to_xml` in
   `crates/slideforge-brand/src/layout_xml.rs:696-700`. Swap `hf` before `txStyles`.
   Add an element-order assertion test in the `#[cfg(test)] mod tests` block.

4. **Add `Brand.layouts` assertion** in `brand_template_from_brand` per §A.5.

5. **Remove or fix the paper-fix dark-layout test** in
   `crates/slideforge-pptx/src/tests/core_tests.rs:1554-1586`
   (`test_f037_011_layout_index_is_wired_not_dead_code`). This test currently only
   verifies "does not panic" — it does NOT verify `<p:clrMapOvr>` is present in the
   output. It MUST be strengthened to assert that a `section_divider` slide in the
   exported ZIP contains `<p:clrMapOvr>`. If this requires additional fixture work
   that cannot complete in STORY-037 scope, the test must be marked `#[ignore]`
   with a comment citing `STORY-038 AC-006` as the story that will un-ignore it —
   the existing "does not panic" assertion is NOT a substitute for the load-bearing
   contract (TD-VSDD-059).

### A.7 Items Deferred to STORY-038

The following items from the adversary's F-PASS2 findings are CONFIRMED as
cross-story deferrals from STORY-037 to STORY-038. They become STORY-038 input
obligations, not open defects when STORY-037 ships:

1. **F-PASS2-C2 (full `clrMapOvr` end-to-end integration test):** A deck with a
   `section_divider` slide must produce a PPTX ZIP where `slide1.xml` contains
   `<p:clrMapOvr>`, verified via `ZipArchive::read`. This requires the full 31-layout
   embedding (STORY-038 AC-003) and the corrected `find_layout_index` from obligation
   #2 above working together. It cannot be tested end-to-end until STORY-038 embeds all
   31 layouts. STORY-038 AC-006 covers this.

2. **STORY-038 mapping table replacement:** The stale layout-index mapping table in
   `STORY-038-pptx-layout-compliance.md` §"All 31 Layouts Present" MUST be replaced
   with the canonical table from §A.4 before STORY-038 implementation begins.
   The story-writer must make this update when STORY-037 ships.

3. **Unmapped DSL keywords (split_contrast, card_rows, etc.):** The definitive
   resolution of which SF custom layout each receives, or whether new layouts are
   needed, is a STORY-038 scope item. STORY-037 uses the generic "Title and Content"
   fallback with a warning for all unmatched keywords.

### A.8 No Dependency Graph Changes Required

The dependency graph (`dependency-graph.md`) already records `slideforge-pptx →
slideforge-brand` and `slideforge-pptx → slideforge-layout` as of the original
ADR-015 (2026-06-03). No new edges are introduced by Addendum A. The addition of
`slide_type_keyword` to `SlideLayoutDef` is an additive, non-breaking change within
`slideforge-brand` that does not alter any crate dependency edge.
