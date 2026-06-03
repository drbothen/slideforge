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
