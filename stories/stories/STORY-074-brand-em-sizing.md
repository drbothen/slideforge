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
- Wave TBD: Wave assignment pending orchestrator dispatch. Likely Wave 3 or 4,
  after STORY-028 merges and before exporter stories (STORY-037+) consume
  `LaidOutDeck` with shape frames that may use em units.

## Summary

STORY-028 deferred brand-aware em conversion because `BrandFonts` in
`slideforge-types` has no `font_size_emu` field. A compile-time constant
`DEFAULT_EM_IN_EMU = 457_200` (0.5 inch at 36pt body font) is used as a safe
placeholder (see `crates/slideforge-layout/src/layout.rs:205–211`).

BC-3.04.001 Postcondition 2 mandates brand-aware em resolution:
> "1em → `em_milliems * brand_font_size_emu / 1_000`"

The constant satisfies the formula structurally but is not brand-aware. This
story closes the structural gap:

1. **Add `font_size_emu: i64` to `BrandFonts`** in `slideforge-types/src/brand.rs`
   (or wherever `BrandFonts` is defined). Default: `457_200` (36pt body at 96dpi,
   matching the current constant). Field must derive `Debug + Clone + PartialEq +
   Eq + Hash` (comemo compatibility per DI-010).

2. **Thread `brand.fonts.font_size_emu` through `layout::run`** to `layout_shapes()`
   replacing `DEFAULT_EM_IN_EMU`. The `layout::run` signature receives `&Brand`,
   which includes `BrandFonts`. No signature change to `run` itself — just use the
   real field instead of the constant.

3. **Remove `DEFAULT_EM_IN_EMU` from `shapes.rs`** once it is no longer referenced.
   Update the deferred comment at `layout.rs:205–211` to cite this story as closed.

4. **Add unit test**: a brand with a non-default `font_size_emu` (e.g., 609_600 for
   48pt) produces correct EMU from `ShapeUnit::Em(1000)` (i.e., `609_600`), not
   `457_200`. Test must fail before the fix and pass after.

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.04.001 | shape: Block Declares Custom Shape with type/position/fill/text/alt | v1.5.0 | AC-001 |

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
let brand_48pt = BrandConfig {
    fonts: BrandFonts { font_size_emu: 609_600, ..Default::default() },
    ..Default::default()
};
let shape_spec = ShapeSpec {
    position: ShapePosition {
        x: ShapeUnit::Em(1000),   // 1em
        y: ShapeUnit::Inches(0),
        width: ShapeUnit::Em(2000), // 2em
        height: ShapeUnit::Em(1000),
    },
    ..minimal_shape()
};
let output = layout_shapes(&[shape_spec], PAGE_SIZE_EMU, 0, brand_48pt.fonts.font_size_emu, 0)
    .expect("should succeed");
let frame = &output.frames[0];
assert_eq!(frame.bounding_box.x, Emu(609_600));
assert_eq!(frame.bounding_box.width, Emu(1_219_200));
```

### AC-002: DEFAULT_EM_IN_EMU constant is removed
(traces to BC-3.04.001 Postcondition 2 — no magic constants in production path)

After this story, `DEFAULT_EM_IN_EMU` is no longer `pub` or referenced by
production code. It MAY be retained as a `const` in tests only, clearly
annotated. The deferred comment at `layout.rs:205–211` is replaced with a
comment citing STORY-074 as closed.

`cargo clippy` must report no `dead_code` warnings for the constant in
production modules.

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
