---
document_type: behavioral-contract
level: L3
version: "1.3.1"
status: active
producer: product-owner
timestamp: 2026-05-28T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-023
lifecycle_status: active
introduced: v1.0.0
modified: ["v1.2 — adversary pass 1 adjudication: codified ShapeSpec position schema, hex color contract, shape_type closed vocabulary, off-canvas boundary semantics, gradient deferral, MissingAlt span, multi-error accumulation", "v1.3 — roundRect added to closed vocabulary per Q7 decision example", "v1.3.1 — STORY-TBD-shape-gradient-fills placeholder resolved to STORY-072"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.04.001: shape: Block Declares Custom Shape with type/position/fill/text/alt

## Description

A `shape:` block inside a slide declaration adds a custom visual shape to that slide.
The shape DSL supports `type` (a closed v1.0 vocabulary of 6 keywords), `position`
(x/y/width/height in inches or em), `fill` (solid hex color — gradient deferred),
`text` (inline content), and required `alt` (accessibility description). The shape
is rendered as an explicit OOXML `<p:sp>` element positioned using integer EMU
coordinates. ALL errors from a slide's shape set are accumulated before returning
(multi-error, per DI-018).

## Preconditions

1. The `shape:` block is nested inside a `slide <type>:` block.
2. The `shape:` block declares `type:` with a keyword from the closed v1.0 vocabulary:
   `rect`, `ellipse`, `arrow`, `line`, `star`, `roundRect`. Any other keyword
   produces E-PAR-012 (unknown shape type; hard error).
3. The `shape:` block declares `alt "..."` OR `decorative: true`.
4. `position:` specifies x, y, width, height as numbers with units (in or em).
5. Hex fill color, if supplied, is exactly 6 uppercase or lowercase hexadecimal
   digits preceded by `#` (i.e., `#RRGGBB` or `#rrggbb`). Case-insensitive;
   both forms accepted. Short form `#RGB` and alpha form `#RRGGBBAA` are rejected.

## Postconditions

1. The shape is added to the slide's `Deck` IR node via a `ShapeSpec` that carries:
   ```rust
   pub struct ShapeSpec {
       pub shape_type: ShapeType,   // resolved enum variant — NOT Arc<str>
       pub position: ShapePosition, // x/y/width/height in ShapeUnit (milliinches or milliem)
       pub fill: FillSpec,          // SolidColor(Rgb) or None; Gradient deferred to STORY-NNN
       pub text: Option<Vec<InlineNode>>,
       pub alt: Option<AltText>,
       pub decorative: bool,
       pub span: SourceSpan,
   }
   ```
   with supporting types:
   ```rust
   pub struct ShapePosition {
       pub x: ShapeUnit,
       pub y: ShapeUnit,
       pub width: ShapeUnit,
       pub height: ShapeUnit,
   }

   pub enum ShapeUnit {
       /// inches × 1000 stored as i64 (avoids f64)
       /// e.g., 0.5in → Inches(500)
       Inches(i64),
       /// em × 1000 stored as i64
       /// e.g., 2em → Em(2000)
       Em(i64),
   }

   pub enum ShapeType {
       Rect,
       Ellipse,
       Arrow,
       Line,
       Star,
       RoundRect,
       // NOTE: no Custom variant in v1.0 — unknown keyword is a parse error
   }

   pub enum FillSpec {
       SolidColor(Rgb),
       None,
       // Gradient variant: DEFERRED to STORY-072 (see Deferred Surfaces section)
   }
   ```
   All types derive `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility
   (DI-010, CLAUDE.md hash+eq+clone rule).

2. The layout stage converts `ShapeSpec.position` fields to integer EMU at layout
   time using the following exact constants:

   | User Unit | Conversion Formula | Example |
   |-----------|-------------------|---------|
   | `1in` | `inches_milliinches * 914_400 / 1_000` | `0.5in → Emu(457_200)` |
   | `1em` | `em_milliems * brand_font_size_emu / 1_000` | `1em → Emu(457_200)` (default 36pt) |
   | `1cm` | `cm_millicm * 360_000 / 1_000` | `1cm → Emu(360_000)` |
   | `1mm` | `mm_millimm * 36_000 / 1_000` | `1mm → Emu(36_000)` |
   | `1pt` | `pt_millipt * 12_700 / 1_000` | `1pt → Emu(12_700)` |
   | `1px` | `px_millipx * 9_525 / 1_000` | `1px → Emu(9_525)` (96 dpi screen) |

   Note: `cm`, `mm`, `pt`, `px` are v1.0 supported units. The division is integer
   division (no `f64`). All intermediate values are `i64`.

3. The shape is produced in the `LaidOutDeck` as a `Frame` with
   `FrameContent::Shape(ShapeFrame { ... })` appended after all placeholder
   frames, in source order.
4. The shape is rendered into the .pptx as `<p:sp>` with
   `<p:cNvPr name="<alt text>">` (semantic name from alt text per accessibility rule).
5. The shape appears in all output formats that support positioned shapes
   (PPTX, PDF, HTML).
6. ALL `MissingAlt` errors for a slide are accumulated in a `Vec<LayoutError>`
   before returning — the function does NOT bail on the first missing-alt shape.
   The caller receives all errors at once (per DI-018 and postcondition on G below).

## Invariants

1. `alt` or `decorative: true` is REQUIRED on every shape. A shape without either is a
   compile error (E-A11-001 at parse/validation; `LayoutError::MissingAlt` at layout
   as a defensive check). (DI-001)
2. Position values are converted to integer EMU at layout time; no `f64` coordinates
   anywhere in the IR. (DI-010, CLAUDE.md)
3. `raw` keyword is NOT available in user .sf files — the shape DSL is the only way
   to add custom shapes. (DI-021, BC-3.04.002)
4. The shape type vocabulary is closed in v1.0: `rect`, `ellipse`, `arrow`, `line`,
   `star`, `roundRect`. Unknown keywords produce E-PAR-012 (no silent
   `ShapeType::Custom(...)` fallback). (CLAUDE.md "no silent fallback" canonical rule)
5. Hex fill colors are case-insensitive (`#FF6F00` and `#ff6f00` both accepted);
   both forms are normalized to uppercase `Rgb { r, g, b }` in the IR.
   Short-form `#RGB` and alpha form `#RRGGBBAA` are rejected with E-PAR-013.
6. `x + width == page_width` (or `y + height == page_height`) is ON-canvas (inclusive
   boundary). Only strict `> page_width` / `> page_height` triggers off-canvas.
7. `LayoutError::MissingAlt` carries a `span: SourceSpan` field pointing to the
   offending `shape:` block in the source file (per CLAUDE.md error-handling rule:
   all errors carry source spans).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | shape: without alt and without decorative: true | `LayoutError::MissingAlt { slide_index, span }` (layout defensive check); E-A11-001 at validation |
| EC-002 | shape: with position where x + width == page_width | ON-canvas (inclusive boundary). No off-canvas warning. Canonical test vector: x=8in, width=2in on 10in canvas → no warning. |
| EC-003 | shape: with position where x + width > page_width by 1 EMU | `LayoutWarning::OffCanvas { slide_index, shape_type, x_emu, y_emu }`. Shape frame still produced at declared position. |
| EC-004 | shape: with negative x or y | `LayoutWarning::OffCanvas`. Shape frame produced. |
| EC-005 | shape: with unknown type keyword (e.g., `type frobnicator`) | E-PAR-012: "Unknown shape type 'frobnicator' at `<file>:<line>:<col>`. Known types: [rect, ellipse, arrow, line, star, roundRect]" |
| EC-006 | shape: with fill "#FF6F00" (uppercase) | Accepted; normalized to `Rgb { r: 255, g: 111, b: 0 }` |
| EC-007 | shape: with fill "#ff6f00" (lowercase) | Accepted; normalized to same `Rgb { r: 255, g: 111, b: 0 }` |
| EC-008 | shape: with fill "#F60" (short-form #RGB) | E-PAR-013: "Invalid hex color '#F60' at `<file>:<line>:<col>`. Expected 6-digit hex (#RRGGBB). Short-form #RGB is not supported." |
| EC-009 | shape: with fill "#FF6F00FF" (alpha #RRGGBBAA) | E-PAR-013: "Invalid hex color '#FF6F00FF' at `<file>:<line>:<col>`. 8-digit alpha hex not supported. Use fill-opacity: attribute for transparency." |
| EC-010 | Slide with two shapes where both lack alt | Both `LayoutError::MissingAlt` errors accumulated; function returns `Err(Vec<LayoutError>)` containing both. |
| EC-011 | shape: with gradient fill (e.g., `fill gradient(brand.primary, brand.accent1)`) | E-PAR-014: "Shape gradient fill is not supported in v1.0. Use a solid hex color or 'none'. Gradient fill is planned for a future release." |
| EC-012 | shape: in DOCX export | Shape rendered as floating `<w:drawing>` inline image (SVG rasterized at 150 DPI) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `shape: type rect position x 0.5in y 1.0in width 2.0in height 1.0in fill "#003766" alt "Blue rectangle highlight"` | `BoundingBox { x: Emu(457_200), y: Emu(914_400), width: Emu(1_828_800), height: Emu(914_400) }` in PPTX `<p:sp>`; exit 0 | happy-path |
| `shape: type rect` (no alt, no decorative) | `LayoutError::MissingAlt { slide_index: 0, span: <span> }` | error |
| `shape: type ellipse decorative: true fill "#FF6F00"` | `AltText::Decorative`; PPTX `<p:sp>` with empty `descr=""` and PDF Artifact tag; exit 0 | edge-case |
| `shape: type frobnicator alt "x"` | E-PAR-012 with known-types list; exit 1 | error |
| `shape: type rect position x 8.0in y 0in width 2.0in height 1.0in alt "edge"` (x+w == page_width == 10in) | ON-canvas; no `OffCanvas` warning | boundary |
| `shape: type rect fill "#ff6f00" alt "orange"` (lowercase hex) | Accepted; same `Rgb { r: 255, g: 111, b: 0 }` as uppercase | case-insensitive |
| `shape: type rect fill "#F60" alt "x"` (short-form) | E-PAR-013; exit 1 | error |
| Slide with 2 shapes both missing alt | `Vec<LayoutError>` with 2 `MissingAlt` entries | multi-error |

## Deferred Surfaces

The following surface is explicitly deferred to a future story. It is NOT silently
missing — it is a planned feature with a spec boundary:

**Gradient fills** (`fill gradient(from, to)`): The `FillSpec::Gradient { from: Rgb, to: Rgb }`
variant is defined in the layout IR type but is NOT reachable from the DSL parser in v1.0.
Any attempt to use gradient syntax at the DSL level produces E-PAR-014.
Target story: **STORY-072** (`STORY-072-shape-gradient-fills.md`).
When that story ships, it will: (1) add gradient parsing to the DSL parser,
(2) remove E-PAR-014, (3) implement `FillSpec::Gradient` handling in PPTX/PDF/HTML exporters.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Shape position in PPTX XML matches EMU conversion of declared user-unit values | unit test (known input → known EMU assertion) |
| VP-TBD | Shape without alt or decorative always produces LayoutError::MissingAlt | unit test |
| VP-TBD | Unknown shape type keyword produces E-PAR-012 (no silent Custom fallback) | unit test |
| VP-TBD | Hex color case-insensitive: lowercase = uppercase for all 6-digit forms | unit test |
| VP-TBD | x + width == page_width is NOT off-canvas; x + width == page_width + 1 EMU IS off-canvas | unit test (boundary) |
| VP-TBD | Slide with N shapes all missing alt returns Vec with N MissingAlt errors | unit test (multi-error accumulation) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 |
| Capability Anchor Justification | CAP-023 ("Structured Shape DSL") per capabilities.md §CAP-023 — "shape: blocks with type, position, size, fill, text, and required alt text" is the verbatim description of CAP-023 |
| L2 Domain Invariants | DI-001 (alt text required on visual elements), DI-010 (integer EMU for all coordinates), DI-018 (error accumulation), DI-021 (raw keyword rejected) |
| Architecture Module | slideforge-layout crate — shape block parsing and EMU conversion; slideforge-pptx crate — shape to sp element |
| Stories | STORY-028, STORY-072 |
| Schema Delegation | `ShapeSpec` position fields (`ShapePosition`, `ShapeUnit`) are added to `slideforge-types/src/specs.rs` by data-engineer per orchestrator dispatch. BC defines the contract; implementation is in data-engineer scope. |

## Related BCs

- BC-3.04.002 — depends on (raw keyword rejection is the complementary contract to shape DSL availability)
- BC-5.01.001 — depends on (alt text enforcement applies to shape: blocks)

## Architecture Anchors

- `architecture/crate-architecture.md` — shape block parsing and EMU conversion

## Story Anchor

STORY-028

## VP Anchors

(filled after VP creation)
