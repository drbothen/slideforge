---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-25T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-05
capability: CAP-010
lifecycle_status: active
introduced: v1.0.0
modified: ["v1.2 — pass-9 fix (F-P9-HIGH-003): InvalidFrameDimension → InvalidBoundingBox (canonical per error.rs:99-109); slide_index → source_slide_index; bounding_box → bbox; updated postcondition 2 and EC-004", "v1.3 — pass-10 completion of F-P9-HIGH-003 partial sweep: all remaining Frame.bounding_box occurrences in postconditions replaced with Frame.bbox; TextFlow.bounding_box left intact (correct production name)"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.06.003: All Positioned Elements Have Valid Non-Negative EMU Coordinates Within Slide Bounds

## Description

Every `Frame` in every `LaidOutSlide` produced by `layout::run` must have a `BoundingBox`
with all four coordinates (`x`, `y`, `width`, `height`) expressed as non-negative `Emu(i64)`
values. Additionally, the region defined by `(x + width)` must not exceed the slide's
`page_size.width`, and `(y + height)` must not exceed `page_size.height`. This contract
enforces that the layout engine never produces geometrically invalid output — negative
positions, zero-dimension frames, or out-of-bounds regions — which would cause silent
corruption in OOXML consumers.

## Preconditions

1. `layout::run` has successfully computed a `LaidOutDeck` (postconditions from BC-3.06.001 hold).
2. The `Brand`'s `PageSize` is valid: `page_size.width > Emu(0)` and `page_size.height > Emu(0)`.
3. Region maps for all 31 slide types have been defined as `const` values and verified at
   crate build time (static assertions on region map correctness).

## Postconditions

1. For every `LaidOutSlide` in `laid_out_deck.slides`:
   - For every `Frame` in `slide.frames`:
     - `frame.bbox.x >= Emu(0)`
     - `frame.bbox.y >= Emu(0)`
     - `frame.bbox.width > Emu(0)`
     - `frame.bbox.height > Emu(0)`
     - `frame.bbox.x + frame.bbox.width <= page_size.width`
     - `frame.bbox.y + frame.bbox.height <= page_size.height`
2. No `Frame` with a zero-dimension bounding box is emitted. Zero-dimension frames indicate
   a layout engine defect; the function returns `Err(LayoutError::InvalidBoundingBox { ... })`
   instead of producing invalid output.
3. EMU values in all `BoundingBox` fields are represented as `Emu(i64)`, not raw `i64`, `f32`,
   or `f64`.

## Invariants

1. All `BoundingBox` values are derived from region maps scaled to the active `PageSize`. No
   frame coordinate is ever calculated using `f64` intermediate arithmetic. (DI-010)
2. The region map for each `SlideTypeKind` is validated at crate initialization that its
   canonical coordinates fit within the default 16:9 page size (static assertion or unit test).
3. Custom `Brand` page sizes scale all region maps proportionally; no frame exceeds the custom
   bounds after scaling.
4. `TextFlow.bounding_box` follows the same constraints as `Frame.bbox`. (DI-010)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Default 16:9 page size (`9_144_000 × 5_143_500` EMU) — title slide | All frames within `(0,0)–(9_144_000, 5_143_500)`; title at `(457200, 1600200)` |
| EC-002 | Custom brand page size (`6_858_000 × 6_858_000` EMU, square slide) | Region maps scale proportionally; all frames remain within custom bounds |
| EC-003 | Slide type with a frame touching the right/bottom edge exactly | `x + width == page_size.width` or `y + height == page_size.height` is valid (inclusive bound) |
| EC-004 | Region map bug produces `x = -1` (hypothetical internal defect) | `Err(LayoutError::InvalidBoundingBox { source_slide_index, frame_index, bbox })` returned; no output produced |
| EC-005 | `TextFlow.bounding_box` for empty-content frame | `width > 0` and `height > 0` (frame is allocated even for empty content); `TextFlow.overflow = Fit`, `line_count = 0` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide title:` with default 16:9 brand | Title frame `BoundingBox { x: Emu(457200), y: Emu(1600200), width: Emu(8229600), height: Emu(1143000) }` — all fields > 0 and within bounds | happy-path |
| `slide content:` with default 16:9 brand | Body frame x ≥ 0, y ≥ 0, x+width ≤ 9_144_000, y+height ≤ 5_143_500 | happy-path |
| All 31 slide types with default brand | Every frame in every slide satisfies coordinate invariants | happy-path (snapshot suite) |
| Custom brand with `slide_width_emu: 12192000, slide_height_emu: 6858000` (4:3 wide) | All frames scale within `(0,0)–(12_192_000, 6_858_000)` | happy-path |
| Proptest: arbitrary valid `(Deck, Brand)` pair | All frame coordinates satisfy non-negative and within-bounds constraints | property-test |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | All `BoundingBox` fields in `LaidOutDeck` are non-negative and within `PageSize` bounds | proptest + snapshot tests for all 31 slide type region maps; unit assertion on canonical title slide |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — "Each type enforces its own required fields and layout rules via the SlideType plugin trait." Valid EMU coordinates within slide bounds ARE the "layout rules" that the SlideType trait enforces per type; this BC defines what "valid layout" means geometrically. |
| L2 Domain Invariants | DI-010 (Integer EMU for All Coordinates — `Emu(i64)` exclusively; no `f64` in BoundingBox); DI-009 (Two-IR Model Integrity — LaidOutDeck must not lose geometric information required by exporters; invalid coordinates would silently corrupt OOXML output) |
| Architecture Module | `slideforge-layout` crate — `regions.rs` region maps; `types.rs` BoundingBox; SS-05 (Layout Engine) |
| Stories | STORY-026 |

## Related BCs

- BC-3.06.001 — composes with (a slide must exist before its frames can be validated)
- BC-3.06.002 — composes with (coordinate validity holds across all runs, not just one)
- BC-3.03.001 — related to (canvas overflow detection uses the same `BoundingBox` to estimate overflow EMU; overflow is a warning, not a coordinate-invalidity error)
- BC-4.01.001 — downstream of (PPTX exporter translates `BoundingBox` EMU directly to OOXML `<a:off x=... y=...>` and `<a:ext cx=... cy=...>`; invalid coordinates corrupt the OOXML schema)
- BC-4.03.005 — downstream of (PDF coordinate mapping flips Y-axis from EMU; requires non-negative Y baseline)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-05 Layout Engine; region map constants
- `architecture/purity-boundary-map.md` — pure-core; coordinate correctness is statically asserted

## Story Anchor

STORY-026

## VP Anchors

VP-011
