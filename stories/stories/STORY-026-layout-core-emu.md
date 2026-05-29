---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-026
title: "Core Layout: Deck → LaidOutDeck, EMU System"
epic: EPIC-07
wave: 3
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: pending PO authorship — behavioral_contracts array below is populated
# from architecture structural contracts; story is marked draft until PO authors
# BC-3.06.001 for layout engine core (structural LaidOutDeck production).
# STORY-026 covers structural layout semantics. BCs below are the closest traceable
# contracts; the story is gated draft until formal BC-3.06.* are authored or the
# PO confirms these structural BCs suffice.
behavioral_contracts: [BC-3.06.001, BC-3.06.002, BC-3.06.003]
verification_properties: [VP-011]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-layout
target_module: slideforge-layout
subsystems: [SS-05]
depends_on: [STORY-001, STORY-003, STORY-011, STORY-015, STORY-016]
blocks:
  - STORY-027
  - STORY-028
  - STORY-037
  - STORY-038
  - STORY-039
  - STORY-040
  - STORY-041
  - STORY-043
  - STORY-044
  - STORY-045
  - STORY-046
  - STORY-047
  - STORY-048
  - STORY-069
estimated_days: 4
---

# STORY-026: Core Layout — Deck → LaidOutDeck, EMU System

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns this story's scope because `slideforge-layout` is the
exclusive owner of the `Deck → LaidOutDeck` transformation per ARCH-INDEX Subsystem
Registry. No other subsystem may perform layout computations.

## Dependency Anchor Justifications

- Depends on STORY-001: `Deck`, `Slide`, `ContentBlock`, `Value`, and `Emu` types must
  exist before layout can consume them.
- Depends on STORY-003: `SlideType` impls supply required field metadata needed for
  layout region computation.
- Depends on STORY-011: Expression evaluator completes `{{ expr }}` resolution; layout
  receives a fully evaluated `Deck`, not an AST.
- Depends on STORY-015/016: Validation must clear alt text, canvas overflow, and
  zero-slide checks before layout runs.
- Blocks STORY-027/028: Section generation and shape layout are additive to the core
  layout foundation this story establishes.
- Blocks all exporters (037-048): All exporters consume `LaidOutDeck`; none can start
  until this story delivers the transformation.

## Summary

Implement the `slideforge-layout` crate's core `layout::run()` function. The function
accepts a fully evaluated `Deck` and a `Brand` and produces a `LaidOutDeck` containing
per-slide positioned shape frames in integer EMU coordinates. This is the foundation
all five exporters consume. The EMU coordinate system (914400 EMU = 1 inch) is the
exclusive representation for all spatial values in the IR — no `f64`, no millimeters,
no points.

## Context: Two-IR Model

```
Deck (semantic, pre-layout)
   title: String
   slides: Vec<Slide>
   vars: HashMap<Arc<str>, Value>
        |
        | layout::run(deck, brand)  ← THIS STORY
        v
LaidOutDeck (geometric, post-layout)
   page_size: PageSize          -- EMU width × height
   slides: Vec<LaidOutSlide>
```

`LaidOutDeck` slide count MUST equal `Deck` slide count. This is VP-011.

## Acceptance Criteria

### AC-001: layout::run signature and return type
`slideforge_layout::layout::run(deck: &Deck, brand: &Brand) -> Result<LaidOutDeck,
LayoutError>` is a public function callable from any exporter crate. It returns
`Err(LayoutError)` only for internal invariant violations (not validation errors —
those must be caught in the validation stage). All `LayoutError` variants carry a
`source_slide_index: usize` field.

### AC-002: Slide count invariant (traces to VP-011 postcondition — slide count preserved)
`laid_out_deck.slides.len() == deck.slides.len()` always holds after a successful
`layout::run()`. If this invariant cannot be satisfied (e.g., an internal bug skips
a slide), `layout::run` returns `Err(LayoutError::SlideCountMismatch { expected,
actual })`. This invariant is the anchor for VP-011 proptest in STORY-069.

### AC-003: EMU coordinate system exclusive throughout LaidOutDeck
All position and size fields in `LaidOutSlide`, `Frame`, `BoundingBox`, and `TextFlow`
use `Emu(i64)` exclusively. No `f32`, `f64`, `u32`, or bare `i64` coordinate fields
are permitted in the layout IR types. The `Emu` newtype from `slideforge-types` is
imported directly. Conversion from user-declared inch values (e.g., `position x 1.0in`)
uses `Emu::from_inches(f: f64) -> Emu` which multiplies by 914400 and truncates to
`i64`.

### AC-004: PageSize from Brand
`LaidOutDeck.page_size: PageSize` is derived from the active `Brand`'s slide
dimensions. Default: widescreen 16:9 = `PageSize { width: Emu(9_144_000),
height: Emu(5_143_500) }` (10 inches × 5.625 inches). Standard 4:3 =
`PageSize { width: Emu(9_144_000), height: Emu(6_858_000) }`. Brand may override
via `slide_width_emu` and `slide_height_emu` fields.

### AC-005: LaidOutSlide structure
Each `LaidOutSlide` contains:
```rust
pub struct LaidOutSlide {
    pub source_index: usize,          // index in Deck.slides
    pub slide_type: SlideTypeKind,    // enum from slideforge-types
    pub frames: Vec<Frame>,           // positioned content regions
    pub speaker_notes: Option<String>, // resolved notes register content
    pub register_tags: RegisterSet,   // which registers are present
}
```
`Frame` contains `BoundingBox` (x, y, width, height as `Emu`) + `FrameContent` enum.

### AC-006: SlideType-driven region layout
Each `SlideTypeKind` produces a deterministic set of `Frame`s according to the type's
canonical region map. Example for `SlideTypeKind::TitleSlide`:
- Title region: `BoundingBox { x: 457200, y: 1600200, width: 8229600, height: 1143000 }`
  (0.5in, 1.75in, 9.0in, 1.25in in 16:9 default)
- Subtitle region: `BoundingBox { x: 457200, y: 2743200, width: 8229600, height: 914400 }`

Region maps for all 31 slide types must be defined as `const` or `fn` in a
`regions` submodule and covered by snapshot tests via `insta`.

### AC-007: TextFlow computation for text frames
`TextFlow` describes how text fits within a `Frame`:
```rust
pub struct TextFlow {
    pub bounding_box: BoundingBox,
    pub overflow: TextOverflow,       // Fit | Truncate | Overflow(excess_emu)
    pub line_count: u32,
    pub estimated_char_width_emu: Emu,
}
```
Canvas overflow detection (BC-3.03.001) uses `TextOverflow::Overflow(excess_emu)`
to compute the EMU estimate for the warning message. This value must be non-negative.

### AC-008: Pure function — no I/O, no side effects
`layout::run()` is a pure function: no file I/O, no network, no panics, no
`println!`, no `tracing` spans (those belong to the CLI orchestration layer).
This purity makes the function Kani-amenable and enables proptest (VP-011, VP-012).

### AC-009: Forbidden dependencies in slideforge-layout
`slideforge-layout` must NOT depend on `slideforge-pptx`, `slideforge-docx`,
`slideforge-pdf`, `slideforge-html`, `slideforge-preview`, `slideforge-data`,
`slideforge-brand` (brand is passed in, not imported), or `slideforge-cli`.
If any of these appear in `Cargo.toml [dependencies]`, the build MUST fail via
`cargo deny` rule or a CI grep check.

### AC-010: All public types implement Hash + Eq + Clone
`LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`, `TextFlow`, `PageSize`,
`LayoutError` all implement `Hash + Eq + Clone` for comemo compatibility and
proptest `Arbitrary` derivability.

### AC-011: #![forbid(unsafe_code)] + #![warn(missing_docs)]
Both attributes are declared at the crate root. All public items have rustdoc comments.
`clippy::pedantic` is clean with zero undocumented suppressions.

### AC-012: Slide count preservation (traces to BC-3.06.001)
Layout transformation output has exactly the same number of slides as the input Deck
(`Deck.slides.len() == LaidOutDeck.slides.len()`). If any internal path skips or
duplicates a slide, `layout::run` returns `Err(LayoutError::SlideCountMismatch {
expected, actual })` rather than silently producing a mismatched output.

### AC-013: Deterministic output for identical inputs (traces to BC-3.06.002)
Given identical `Deck` + `Brand` inputs, `layout::run` produces a `LaidOutDeck` with
byte-identical content on every invocation, verified via `Hash` equality:
`layout::run(&deck, &brand).unwrap() == layout::run(&deck, &brand).unwrap()`.
No random seeds, no timestamp fields, no non-deterministic data structures (e.g.,
unordered `HashMap` iteration) may influence frame ordering or field values.

### AC-014: Valid EMU coordinates for all frames (traces to BC-3.06.003)
Every `BoundingBox` in every `Frame` across all `LaidOutSlide`s satisfies:
`x >= 0`, `y >= 0`, `width > 0`, `height > 0`,
`x + width <= page_size.width.0`, `y + height <= page_size.height.0`.
Violations are caught by a post-layout integrity check inside `layout::run` and
returned as `Err(LayoutError::InvalidBoundingBox { source_slide_index, frame_index, bbox })`.

## Behavioral Contracts

| BC ID | Title | Verified By |
|-------|-------|-------------|
| BC-3.06.001 | Layout preserves slide count | Unit test: `assert_eq!(laid_out.slides.len(), deck.slides.len())` |
| BC-3.06.002 | Layout is deterministic | Proptest: identical inputs → `Hash`-equal outputs |
| BC-3.06.003 | Valid EMU coordinates | Proptest: all bounding boxes within slide bounds |

## Tasks

- [ ] Create `crates/slideforge-layout/` with `Cargo.toml` (deps: `slideforge-types`, `thiserror = "=2.0.18"`)
- [ ] Add `slideforge-layout` to workspace `Cargo.toml` members
- [ ] Declare `LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`, `TextFlow`, `PageSize` types in `src/types.rs`
- [ ] Declare `LayoutError` with `thiserror` in `src/error.rs`
- [ ] Implement `layout::run()` stub returning `todo!()` in `src/lib.rs`
- [ ] Implement `regions` submodule with region maps for all 31 slide types in `src/regions.rs`
- [ ] Implement `TextFlow` computation in `src/text_flow.rs`
- [ ] Implement `Emu::from_inches` and `Emu::from_points` helpers if not already in `slideforge-types`
- [ ] Write unit tests in `src/lib.rs #[cfg(test)]` covering: slide count invariant, EMU conversion, region maps for 5 representative slide types
- [ ] Write `insta` snapshot tests for region maps (all 31 types, one snapshot per type)
- [ ] Write proptest for VP-011 skeleton: `layout::run` preserves slide count for arbitrary `Deck` (proptest to be expanded in STORY-069)

## Previous Story Intelligence

N/A — STORY-026 is the first story in EPIC-07 (Layout Engine). No predecessor in
this epic.

The dependency STORY-022/023 (Brand) establishes the `Brand` type consumed here.
Lesson from similar foundation stories (STORY-001): define all `Hash + Eq + Clone`
derives from day 1. Do not defer them to later stories. They are required for
proptest and Kani.

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and `architecture/purity-boundary-map.md`:

1. **SS-05 is Pure core**: `slideforge-layout` is classified as a pure core crate.
   Zero I/O operations are permitted. Zero side effects.
2. **Two-IR boundary**: `layout::run` is the ONLY function that crosses the Deck →
   LaidOutDeck boundary. No exporter may reach into a `Deck` directly; all must
   consume `LaidOutDeck`.
3. **EMU exclusivity (ADR-013)**: All coordinate and size values in layout output
   use `Emu(i64)`. No floating point in the IR.
4. **Slide count invariant (VP-011)**: The transformation preserves slide count.
   Violation is an internal error (return `Err`), not a user error.
5. **Region map immutability**: Slide type region maps are `const` definitions or
   pure functions, not mutable state. Same `SlideTypeKind` always produces the same
   region layout for a given `PageSize`.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `Emu`, `Brand`, `Value`, `SlideTypeKind` |
| `thiserror` | `=2.0.18` | `LayoutError` derive |
| `insta` | `=1.39.0` | snapshot tests for region maps |
| `proptest` | `=1.5.0` | VP-011 proptest skeleton |

Do NOT add `serde`, `tokio`, `reqwest`, or any exporter crate as a dependency.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-layout/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-layout/src/lib.rs` | Create | Crate root + `layout::run` |
| `crates/slideforge-layout/src/types.rs` | Create | `LaidOutDeck`, `LaidOutSlide`, `Frame`, `BoundingBox`, `TextFlow`, `PageSize` |
| `crates/slideforge-layout/src/error.rs` | Create | `LayoutError` enum |
| `crates/slideforge-layout/src/regions.rs` | Create | Region maps for 31 slide types |
| `crates/slideforge-layout/src/text_flow.rs` | Create | `TextFlow` computation |
| `crates/slideforge-layout/src/regions/snapshots/` | Create | insta snapshot directory |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| `slideforge-types` source (types, Emu) | ~3,000 |
| `slideforge-eval` output types (Deck IR) | ~2,000 |
| Test files to write | ~3,000 |
| Architecture section files | ~1,500 |
| BC files (3 BCs) | ~1,200 |
| **Total** | **~13,200** |

Well within 20% of agent context window. No split needed.

## Test Strategy

- **Unit tests**: `layout::run` on a 0-slide deck returns `Err(SlideCountMismatch)`;
  on a 3-slide deck returns `Ok` with `slides.len() == 3`; EMU conversion from 1.0in
  = 914400; from 0.5in = 457200.
- **Snapshot tests (insta)**: One snapshot per slide type showing the canonical
  region map BoundingBox values for 16:9 default page size.
- **Proptest (VP-011 skeleton)**: `∀ deck: Deck. layout::run(deck, brand).map(|d|
  d.slides.len()) == Ok(deck.slides.len())`. Proptest `Arbitrary` implementation for
  `Deck` with bounded sizes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-slide `Deck` (caught upstream but defensive) | `layout::run` returns `Err(LayoutError::EmptyDeck)` |
| EC-002 | Slide type without a defined region map | `layout::run` returns `Err(LayoutError::UnknownSlideType { kind })` |
| EC-003 | `Brand` declares non-standard page dimensions | `PageSize` uses brand values; region maps scale proportionally |
| EC-004 | TextFlow: content text is empty string | `TextFlow { overflow: TextOverflow::Fit, line_count: 0, ... }` — no error |

## Forbidden Dependencies

`slideforge-layout/Cargo.toml` MUST NOT contain any of:
- `slideforge-pptx`
- `slideforge-docx`
- `slideforge-pdf`
- `slideforge-html`
- `slideforge-preview`
- `slideforge-data`
- `slideforge-cli`
- `slideforge-brand` (Brand is passed by reference — not imported as a crate dep)

If any of these appear, the CI `cargo deny` check or a workspace-level grep hook
MUST fail the build.

## Implementation Notes

The region maps for all 31 slide types are numeric constants. The Python reference
implementation in `.factory/seed/reference/` is the source of truth for visual
behavior. The implementer MUST verify the region map values produce correct EMU
positions by cross-referencing the reference implementation's pixel-to-inch ratios.

For `Emu::from_inches(f: f64) -> Emu`: use `Emu((f * 914_400.0) as i64)`. This
is integer truncation, which is correct for layout purposes (sub-EMU precision
is below perceptible rounding at any real DPI).

The `TextFlow` overflow detection is a heuristic for the canvas overflow warning
(BC-3.03.001). It is NOT a pixel-perfect reflow. Use a fixed estimated character
width: `Emu(914400 / 12)` per character at 12pt font size (72 points/inch * 12
points = 1 inch / 12 chars ~ 76200 EMU/char). This heuristic is acceptable per
BC-3.03.001 which requires an "EMU estimate" not an exact value.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-24 | story-writer | Initial decomposition — Deck → LaidOutDeck, EMU system, layout::run stub, region maps for 31 slide types |
| 1.1 | 2026-05-29 | product-owner | Pass-9 fix (F-P9-HIGH-003 sibling-sweep): AC-014 field name corrected — `slide_index` → `source_slide_index` in `InvalidBoundingBox` variant pattern to match canonical `error.rs:99-109` |
