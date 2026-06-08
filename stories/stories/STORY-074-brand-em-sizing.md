---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-074
title: "Brand-aware Em conversion (font_size_emu)"
epic: EPIC-07
wave: 5
points: 3
priority: P2
tdd_mode: strict
status: draft
crate: slideforge-types
target_module: slideforge-types + slideforge-layout
subsystems: [SS-05]
behavioral_contracts: [BC-3.04.001]
verification_properties: [VP-037]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-028]
blocks: []
estimated_days: 1
version: "1.6"
---

# STORY-074: Brand-aware Em conversion (font_size_emu)

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns this story because the em→EMU conversion is a
layout-stage concern: `ShapeUnit::Em` values are resolved to integer EMU at
`layout::run()` time using the brand's body font size as the reference `1em`
unit. The `BrandFonts.font_size_emu` field that carries this value lives in
`slideforge-types` (SS-01 type definitions) but is threaded through to
`layout_shapes()` in SS-05.

## Dependency Anchor Justifications

- Depends on STORY-028: `layout_shapes()`, `DEFAULT_EM_IN_EMU`, `ShapeUnit::Em`,
  and the deferred comment at `layout.rs:208` ("STORY-NNN-brand-em-sizing") are
  all established by STORY-028. This story closes that deferral by adding
  `font_size_emu: i64` to `BrandFonts` and threading the real value through
  `layout::run` → `layout_shapes`.
- Wave 5: Assigned to Wave 5 per orchestrator dispatch (frontmatter `wave: 5`).
  STORY-028 merged in Wave 4 (depends_on satisfied). Exporter stories
  (STORY-037+) that consume `LaidOutDeck` with shape frames using em units are
  also in Wave 5, and this story unblocks them within the wave.

## Summary

STORY-028 deferred brand-aware em conversion because `BrandFonts` in
`slideforge-types` had no `font_size_emu` field. A compile-time constant
`DEFAULT_EM_IN_EMU = 457_200` (0.5 inch at 36pt body font) was used as a safe
placeholder (historically at `layout.rs:205–211` in the STORY-028 delivery;
the STORY-074-CLOSED comment now lives at `layout.rs:262–266` after this
story's implementation).

BC-3.04.001 Postcondition 2 mandates brand-aware em resolution:
> "1em → `em_milliems * brand_font_size_emu / 1_000`"

The constant satisfies the formula structurally but is not brand-aware. This
story closes the structural gap:

1. **Add `font_size_emu: i64` to `BrandFonts`** in `slideforge-types/src/brand.rs`
   (or wherever `BrandFonts` is defined). Default: `457_200` (36pt body font =
   0.5 inch at 914_400 EMU/inch, matching the current constant). Field must derive `Debug + Clone + PartialEq +
   Eq + Hash` (comemo compatibility per DI-010).

2. **Thread `brand.fonts.font_size_emu` through `layout::run`** to `layout_shapes()`
   replacing `DEFAULT_EM_IN_EMU`. The `layout::run` signature receives `&Brand`,
   which includes `BrandFonts`. No signature change to `run` itself — just use the
   real field instead of the constant.

3. **Remove `DEFAULT_EM_IN_EMU` from `shapes.rs`** once it is no longer referenced.
   Replace the deferred comment (historically at `layout.rs:205–211` in the
   STORY-028 delivery) with a STORY-074-CLOSED comment. In the delivered
   implementation this closed comment lives at `layout.rs:262–266`.

4. **Add unit test**: a brand with a non-default `font_size_emu` (e.g., 609_600 for
   48pt) produces correct EMU from `ShapeUnit::Em(1000)` (i.e., `609_600`), not
   `457_200`. Test must fail before the fix and pass after.

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.04.001 | shape: Block Declares Custom Shape with type/position/fill/text/alt | v1.6 | AC-001 |

## Acceptance Criteria

### AC-001: Brand font_size_emu drives em→EMU conversion
(traces to BC-3.04.001 Postcondition 2 — brand-aware em resolution)

Given a `BrandFonts { font_size_emu: 609_600, .. }` (48pt body font), a shape
position of `ShapeUnit::Em(1000)` (1em) resolves to `Emu(609_600)` in the
`LaidOutDeck`. A brand with `font_size_emu: 457_200` (36pt, default) resolves
`ShapeUnit::Em(1000)` to `Emu(457_200)`.

The resolution uses the formula from BC-3.04.001 PC-2:
```
em_emu = em_milliems * brand_font_size_emu / 1_000
```

Integer division (`i64`), no `f64` at any point.

```rust
// Canonical test fixture (must be a unit test in slideforge-layout tests)
// See the real, compiling integration test at:
//   crates/slideforge-layout/tests/brand_em_sizing.rs
//
// NOTE — illustrative pseudocode below. Two helpers have no real definition:
//   PAGE_SIZE_EMU    → use PageSize { width: Emu(12_192_000), height: Emu(6_858_000) }
//   minimal_shape()  → build ShapeSpec inline (see brand_em_sizing.rs em_x_shape_spec / em_width_shape_spec)
// All other symbols are real types from the delivered implementation.

// brand_48pt is slideforge_types::BrandFonts (NOT slideforge_brand::BrandConfig —
// that is the TOML-layer type whose `fonts` field is FontConfig, which has no
// font_size_emu).  The IR type that owns font_size_emu is BrandFonts.
let brand_48pt = BrandFonts { font_size_emu: 609_600, ..Default::default() };

// ShapeSpec / ShapePosition / ShapeUnit are slideforge_types; layout_shapes is in
// slideforge_layout::shapes (5-arg signature: shapes, page, source_slide_index,
// em_in_emu: i64, base_index: usize).
let shape_spec = ShapeSpec {
    position: ShapePosition {
        x: ShapeUnit::Em(1000),    // 1em
        y: ShapeUnit::Inches(0),
        width: ShapeUnit::Em(2000), // 2em
        height: ShapeUnit::Em(1000),
    },
    ..minimal_shape() // illustrative — build ShapeSpec inline in the real test
};
let output = layout_shapes(&[shape_spec], PAGE_SIZE_EMU, 0, brand_48pt.font_size_emu, 0)
    //                                    ^illustrative   ^real field on BrandFonts
    .expect("should succeed");
let frame = &output.frames[0];
// Frame.bbox: BoundingBox (field is `bbox`, NOT `bounding_box`).
// `bounding_box` belongs to TextFlow, not Frame — do NOT confuse them.
assert_eq!(frame.bbox.x,     Emu(609_600));
assert_eq!(frame.bbox.width, Emu(1_219_200));
```

### AC-002: DEFAULT_EM_IN_EMU constant is removed
(traces to BC-3.04.001 Postcondition 2 — no magic constants in production path)

After this story, `DEFAULT_EM_IN_EMU` is no longer `pub` or referenced by
production code. It MAY be retained as a `const` in tests only, clearly
annotated. The deferred comment at `layout.rs:205–211` is replaced with a
comment citing STORY-074 as closed.

`cargo clippy` must report no `dead_code` warnings for the constant in
production modules. In the delivered implementation the STORY-074-CLOSED
comment replacing the original placeholder lives at `layout.rs:262–266`
(historical STORY-028 placeholder was at `layout.rs:205–211`).

### AC-003: BrandFonts default is backward-compatible
(traces to BC-3.04.001 Invariant 2 — integer EMU, no regressions)

`BrandFonts::default()` (or the `Default` impl) sets `font_size_emu: 457_200`.
All existing tests that relied on the `DEFAULT_EM_IN_EMU` constant produce
identical results when run against the new default-brand path. No test that
passed before STORY-074 may fail after it.

## Implementation Notes

- `BrandFonts` location: check `slideforge-types/src/brand.rs`. If the struct
  is in `slideforge-brand`, the field addition is in that crate but the
  `slideforge-layout` change (threading the value) is still in SS-05 scope.
- `layout_shapes` signature accepts `em_in_emu: i64` as the fourth argument
  and `base_index: usize` as the fifth — confirmed in the STORY-028 + STORY-055
  delivered implementation. This story only changes the call site in
  `layout::run` from `DEFAULT_EM_IN_EMU` to `brand.fonts.font_size_emu` (or
  equivalent accessor); `base_index` is passed through unchanged from the
  surrounding context.
- No change to `layout_shapes` internal logic or signature required.
- The `Emu` newtype or `i64` raw value: match whatever type `DEFAULT_EM_IN_EMU`
  is today. Do not introduce a new type.

## Change Log

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-29 | product-owner | Created — resolves STORY-NNN-brand-em-sizing placeholder at layout.rs:208 (F-P18-MED-001 from pass-18 report). Closes structural deferral from STORY-028. |
| 1.1 | 2026-06-08 | story-writer | Prose-only correction per adversary Pass-1 F-074-P1-LOW-001: updated illustrative fixture to 5-arg `layout_shapes` call (added trailing `base_index` = 0) and updated Implementation Note to reflect `em_in_emu` (4th arg) + `base_index` (5th arg) per actual delivered signature. No AC semantics or contract thresholds changed. |
| 1.2 | 2026-06-08 | story-writer | Prose-only correction per adversary Pass-2 F-074-P2-LOW-001: corrected `font_size_emu` field-type annotation from `Emu` to `i64` in Dependency Anchor Justifications and Summary §1, matching the delivered `DEFAULT_EM_IN_EMU: i64` constant and `em_in_emu: i64` parameter in `layout_shapes`. Implementation Notes §4 (`i64` authorization) and all AC semantics unchanged. |
| 1.3 | 2026-06-08 | story-writer | Prose↔code coherence sweep per adversary Pass-5 F-074-P5-LOW-001 + proactive full-fixture reconciliation (LESSON-10). Fixed two type/field defects in the AC-001 illustrative fixture: (1) replaced `BrandConfig { fonts: BrandFonts { ... } }` with `BrandFonts { ... }` — `BrandConfig` is the slideforge-brand TOML-schema type (fields: `ColorConfig`, `FontConfig`); the IR type that owns `font_size_emu` is `slideforge_types::BrandFonts`; (2) replaced `frame.bounding_box.x` / `frame.bounding_box.width` with `frame.bbox.x` / `frame.bbox.width` — `Frame.bbox: BoundingBox` (field `bbox`); `bounding_box` belongs to `TextFlow`, not `Frame`. Also added inline annotations for two illustrative-only helpers (`PAGE_SIZE_EMU`, `minimal_shape()`) that have no real definition, with a cross-reference to `crates/slideforge-layout/tests/brand_em_sizing.rs`. No AC semantics, BC references, thresholds, or em-sizing contract changed. |
| 1.4 | 2026-06-08 | story-writer | Comprehensive prose coherence sweep resolving adversary Pass-8 F-074-P8-LOW-001 (wave contradiction) plus all remaining frontmatter↔body drift, stale line citations, and hedging language: (1) Replaced "Wave TBD / pending / likely Wave 3 or 4" in Dependency Anchor Justifications with resolved Wave 5 statement matching frontmatter `wave: 5`; (2) Corrected three stale `layout.rs:205–211` citations — the STORY-028 deferral placeholder location — to accurately reflect that the STORY-074-CLOSED comment now lives at `layout.rs:262–266` in the delivered implementation; historical origin clearly labelled in each case; (3) Updated Summary tense from present-continuous ("is used as a safe placeholder") to past tense reflecting the delivered state ("was used… historically"). No AC semantics, BC references, thresholds, or em-sizing contract changed. |
| 1.5 | 2026-06-08 | story-writer | BC version cell update per adversary Pass-9 F-074-P9-LOW-001: corrected BC-3.04.001 version in Behavioral Contracts table from v1.5.0 to v1.6 (source-of-truth: BC-3.04.001.md frontmatter `version: "1.6"`). Final exhaustive label/version audit confirmed: BC H1 title matches source-of-truth; all body AC traces cite BC clauses by name (no version numbers embedded in AC text); no other BC/ADR/spec version numbers cited in this story; no stale line citations, no hedging, no frontmatter↔body incoherence. No AC semantics, BC contracts, or thresholds changed. |
| 1.6 | 2026-06-08 | story-writer | Final derivation-claim accuracy audit per adversary Pass-10 F-074-P10-LOW-001. Fixed Summary §1 line: removed category-error `96dpi` annotation from `457_200` default value — `96dpi` is a screen-pixel density, not a pt↔inch↔EMU conversion factor. Corrected phrasing to `457_200 (36pt body font = 0.5 inch at 914_400 EMU/inch)`. Full exhaustive audit of all derivation claims confirmed arithmetically correct: 457_200 = 36pt/72 × 914_400; 609_600 = 48pt/72 × 914_400; 1_219_200 = 2000 × 609_600 / 1_000; formula em_milliems × brand_font_size_emu / 1_000 is integer-exact; no other dpi claims, no stale line citations, no frontmatter↔body drift, BC v1.6 confirmed. No AC semantics, BC contracts, or thresholds changed. |
