---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-072
title: "shape: Gradient Fills (FillSpec::Gradient)"
epic: EPIC-07
wave: 5
points: 3
priority: P2
tdd_mode: strict
status: draft
spec_version: "1.3"
behavioral_contracts: [BC-3.04.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
crate: slideforge-layout
target_module: slideforge-layout
subsystems: [SS-05]
depends_on: [STORY-028]
blocks: []
estimated_days: 1
---

# STORY-072: shape: Gradient Fills (FillSpec::Gradient)

## Subsystem Anchor Justification

SS-05 (Layout Engine) owns this story's scope because `FillSpec::Gradient` is
processed during the `Deck → LaidOutDeck` transformation in `slideforge-layout`.
PPTX/DOCX/PDF/HTML exporters receive the positioned `ShapeFrame` with
`FillSpec::Gradient { from, to }` payload — they do not construct the gradient
themselves. SS-06 (PPTX), SS-07 (PDF), SS-09 (HTML) each implement their own
format-specific gradient encoding from the common `ShapeFrame` data.

## Dependency Anchor Justifications

- Depends on STORY-028: `ShapeSpec`, `FillSpec`, `ShapeFrame`, `BoundingBox`, and
  the `layout::run()` shape pass are all established by STORY-028. This story adds
  the `FillSpec::Gradient { from: Rgb, to: Rgb }` variant to the `FillSpec` enum in
  `slideforge-types` (currently only `SolidColor(Rgb)` and `None` per BC-3.04.001
  v1.4.2 — the Gradient variant does NOT exist in the v1.0 codebase) and wires
  parser → IR → layout → exporter. It also removes E-PAR-016 (the "gradient
  unsupported" guard).

## Summary

Add `FillSpec::Gradient { from: Rgb, to: Rgb }` to the `FillSpec` enum in
`slideforge-types` and wire it end-to-end: DSL parser → `ShapeSpec` IR →
layout passthrough → PPTX/DOCX/PDF/HTML exporters.

The `FillSpec::Gradient` variant does NOT exist in the v1.0 codebase (confirmed per
BC-3.04.001 v1.4.2 and code audit of `slideforge-types/src/shape_types.rs`). The
`FillSpec` enum currently contains only `SolidColor(Rgb)` and `None`. Any attempt to
use gradient syntax at the DSL level produces E-PAR-016 ("Shape gradient fill is not
supported in v1.0"). This story ships the gradient surface, removes E-PAR-016, and
implements native gradient emission in all four output formats (with a documented
fallback where the format does not natively support gradients).

This is a deferred surface per BC-3.04.001 v1.3 "Deferred Surfaces" section. The
story is P2 (not blocking v1.0 core), wave TBD (orchestrator assigns post-Wave 4
when exporters are available).

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-3.04.001 | shape: block declares custom shape with type/position/fill/text/alt | v1.3 | AC-001, AC-002, AC-003, AC-004, AC-005 |

## Acceptance Criteria

### AC-001: DSL gradient syntax accepted by parser
(traces to BC-3.04.001 precondition 5 — fill syntax; Deferred Surfaces section)

The DSL parser accepts the gradient fill syntax:

```
shape:
  type rect
  position x 1.0in y 1.0in width 3.0in height 2.0in
  fill gradient "#FF0000" to "#0000FF"
  alt "Gradient background"
```

Note: hex color values MUST be quoted strings (the lexer treats `#` as a line
comment character when unquoted). Color syntax follows the same hex rules as
solid fills: 6-digit only, case-insensitive, short/alpha forms rejected with
E-PAR-015. The parser records the gradient in the AST (`ShapeNode.fill` stores
the interim string `"gradient \"#FF0000\" to \"#0000FF\""`); the structural
`FillSpec::Gradient { from: Rgb, to: Rgb }` is constructed at the
`ShapeNode → ShapeSpec` decode boundary (see AC-002). The E-PAR-016 error is
removed by this story.

### AC-002: FillSpec::Gradient in ShapeSpec IR — two-step AST→IR decode
(traces to BC-3.04.001 postcondition 1 — ShapeSpec.fill populated)

Gradient fill is produced through a two-step process:

1. **Parser step (AST):** When the parser processes a
   `fill gradient "#RRGGBB" to "#RRGGBB"` declaration (quoted hex required — `#`
   is a comment character when unquoted), it records the fill in `ShapeNode.fill`
   as an interim AST string representation (e.g.,
   `"gradient \"#FF0000\" to \"#0000FF\""`). The parser does NOT directly
   construct `FillSpec::Gradient`.

2. **Decode step (IR):** At the `ShapeNode → ShapeSpec` decode boundary
   (implemented in `slideforge-syntax` or whichever crate owns `ShapeNode`
   lowering to `ShapeSpec`), the interim AST string is parsed and the structural
   `FillSpec::Gradient { from: Rgb, to: Rgb }` IR variant is constructed. This
   is the form consumed by `slideforge-layout`, `slideforge-pptx`,
   `slideforge-pdf`, `slideforge-html`, and `slideforge-docx`.

Both colors follow the same hex parsing rules as `FillSpec::SolidColor`: quoted
6-digit hex (`"#RRGGBB"`), uppercase or lowercase, normalized to
`Rgb { r: u8, g: u8, b: u8 }` integers. Short form `"#RGB"` and alpha form
`"#RRGGBBAA"` are rejected with E-PAR-015 at the decode step.

> **Shape-pipeline-wiring dependency note (OBS-072-P1-001):** STORY-072 delivers
> (a) the parser branch that records gradient syntax in `ShapeNode.fill`, (b) the
> `ShapeNode → ShapeSpec` decode that produces `FillSpec::Gradient`, and (c) the
> gradient rendering logic in all 5 exporters (PPTX/PDF/HTML/DOCX-fallback/layout
> passthrough). These are verified via unit tests constructed with a direct
> `ShapeSpec { fill: FillSpec::Gradient { ... }, ... }`. However, the
> end-to-end DSL path from a `.sf` source file through to rendered output is NOT
> wired in production: Stage-2b (`thread_fields_to_blocks`) does not yet emit
> `ContentBlock::Shape` for any shape block (pre-existing gap, broader than
> gradients — see BC-1.16.001 inv-4). The full DSL→output pipeline requires a
> separate shape-pipeline-wiring story (route to wave-gate as
> FU-SHAPE-PIPELINE-WIRING). Do NOT claim a working end-to-end DSL path in tests
> or implementation claims.

### AC-003: Layout passthrough — gradient frame in LaidOutDeck
(traces to BC-3.04.001 postcondition 3 — shape appears in LaidOutDeck)

The layout stage (`layout::run()`) passes `FillSpec::Gradient` through to
`ShapeFrame.fill` in `LaidOutDeck` without modification. The gradient direction and
stop colors are preserved verbatim. No EMU conversion is needed for fill (only for
position — already handled by STORY-028).

### AC-004: Exporter gradient emission (native where supported, fallback otherwise)
(traces to BC-3.04.001 postcondition 5 — shape appears in all output formats)

Each exporter emits gradients as follows:

| Format | Gradient Behavior |
|--------|------------------|
| PPTX | Native: ooxmlsdk `=0.6.1` typed builders for `a:gradFill` / `a:gsLst` / `a:gs` / `a:lin` — emit typed DrawingML gradient fill with `a:lin ang="5400000"` (top→bottom) and two `a:gs` stops at `pos="0"` (from) and `pos="100000"` (to). Use the typed-builder API, not raw XML. |
| DOCX | Fallback: solid first color (`from` Rgb) rendered as `<w:shd w:fill="RRGGBB"/>`. A lint warning is emitted: "DOCX gradient fill downgraded to solid (DOCX does not support shape gradient fills)" |
| PDF | Native via krilla `=0.6.0`: `Surface::set_fill(Some(Fill { paint: LinearGradient { x1, y1, x2, y2, spread_method, stops: Vec<Stop> }.into(), rule: FillRule::NonZero, opacity }))` then `draw_path(rect)`. Uses `paint::LinearGradient` from the krilla API. `pdf-writer` is NOT a direct dependency — krilla wraps it internally and a direct `pdf-writer` dep creates version-skew risk. (per export-architecture v1.2) |
| HTML | Native: CSS `background: linear-gradient(to bottom, #RRGGBB, #RRGGBB)` on the shape div |

Gradient direction in v1.0 is fixed as top-to-bottom (vertical linear). Arbitrary
angle gradients are deferred to v2.

### AC-005: Alt text and decorative semantics preserved for gradient shapes
(traces to BC-3.04.001 invariant 1 — alt required on every shape)

Gradient shapes are subject to the same `alt` / `decorative: true` requirement as
solid shapes. A gradient `shape:` without `alt` or `decorative: true` produces
`LayoutError::MissingAlt` (same as AC-BC-A5 in STORY-028). The `FillSpec` variant
does not change the alt-text contract.

## Tasks

- [ ] Add `Gradient { from: Rgb, to: Rgb }` variant to `FillSpec` in `slideforge-types/src/shape_types.rs` (variant does NOT exist yet — must be added as the FIRST task; confirm it derives `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility)
- [ ] Extend DSL parser (in `slideforge-syntax` or `slideforge-eval` — whichever owns `shape:` parsing) to accept `fill gradient "#RRGGBB" to "#RRGGBB"` (quoted hex — `#` is a line-comment when unquoted) and decode to `FillSpec::Gradient`
- [ ] Remove E-PAR-016 error code from the parser (the "gradient not supported" guard)
- [ ] Update `layout::run()` shape pass to pass `FillSpec::Gradient` through to `ShapeFrame` (likely already correct — passthrough logic in STORY-028 should be variant-agnostic)
- [ ] Implement PPTX gradient: use ooxmlsdk `=0.6.1` typed builders for `a:gradFill` / `a:gsLst` / `a:gs` / `a:lin` — typed API, not raw XML
- [ ] Implement PDF gradient via krilla `=0.6.0`: call `Surface::set_fill(Some(Fill { paint: paint::LinearGradient { x1, y1, x2, y2, spread_method: SpreadMethod::Pad, stops: vec![Stop { offset: 0.0, color: from_color }, Stop { offset: 1.0, color: to_color }] }.into(), rule: FillRule::NonZero, opacity: NormalizedF32::ONE }))` then `draw_path(rect)`. Do NOT add `pdf-writer` as a direct dep — krilla exposes the full gradient API.
- [ ] Implement HTML gradient: emit `background: linear-gradient(to bottom, ...)` CSS
- [ ] Implement DOCX fallback: solid `from` color + lint warning
- [ ] Write unit tests:
  - Parser: `fill gradient "#FF0000" to "#0000FF"` → AST `ShapeNode.fill = "gradient \"#FF0000\" to \"#0000FF\""` then decoded to `FillSpec::Gradient { from: Rgb(255,0,0), to: Rgb(0,0,255) }`
  - Layout passthrough: gradient preserved in `ShapeFrame.fill`
  - Alt-text enforcement still applies to gradient shapes
- [ ] Write snapshot tests: PPTX XML output for a gradient shape (verify `<a:gradFill>` structure)
- [ ] Write integration test: full pipeline for a `.sf` file with a gradient shape → `.pptx` output

## Previous Story Intelligence

STORY-028 established the `FillSpec` enum with `SolidColor(Rgb)` and `None` variants
and a comment-placeholder noting `Gradient` as deferred (per BC-3.04.001 v1.4.2 and
confirmed by code audit of `slideforge-types/src/shape_types.rs`). The
`FillSpec::Gradient` variant was NOT added during STORY-028 — it is deferred to this
story. The implementer's FIRST task is adding `Gradient { from: Rgb, to: Rgb }` to
`slideforge-types/src/shape_types.rs`.

The STORY-028 `layout::run()` shape pass should already be variant-agnostic for
`FillSpec` — the passthrough stores `ShapeSpec.fill` verbatim into `ShapeFrame.fill`.
If match arms are exhaustive on `FillSpec`, adding `Gradient` will trigger compile
errors in exporter code — fix those exhaustive matches in the same story.

## Architecture Compliance Rules

1. **Plugin-first (DI-021)**: The gradient exporter logic belongs in the respective
   exporter plugins (`slideforge-pptx`, `slideforge-pdf`, `slideforge-html`). The
   layout stage only passes through the `FillSpec::Gradient` — it does not produce
   format-specific gradient markup.
2. **No f64 in IR (DI-010, ADR-013)**: `Rgb` components are `u8`. No float in gradient
   stops or positions. PPTX gradient stop positions are integer (0 and 100000 per
   OOXML spec).
3. **Alt text required (DI-001)**: Gradient shapes are NOT exempt from the alt/decorative
   requirement. The `MissingAlt` check in `layout::run()` is variant-agnostic.
4. **DOCX fallback is intentional**: The DOCX fallback to solid color is a documented
   v1.0 limitation, not a bug. The lint warning must be emitted.
5. **Error accumulation (DI-018)**: If multiple gradient shapes are missing alt text,
   ALL errors are accumulated before returning.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `FillSpec::Gradient`, `Rgb`, `ShapeSpec` |
| `slideforge-layout` | workspace | `layout::run()` shape pass extension |
| `ooxmlsdk` | `=0.6.1` | PPTX typed builders for `a:gradFill` / `a:gsLst` / `a:gs` / `a:lin` |
| `krilla` | `=0.6.0` (pinned in slideforge-pdf/Cargo.toml, NOT workspace) | PDF `paint::LinearGradient` + `Surface::set_fill` + `draw_path` (per export-architecture v1.2) |
| `thiserror` | `{workspace = true}` (=2.0.18 — centralized in [workspace.dependencies] per ADR-022) | `LayoutError` extension (if new variants needed) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-types/src/shape_types.rs` | Modify | Add `FillSpec::Gradient { from: Rgb, to: Rgb }` variant (does not yet exist) |
| `crates/slideforge-syntax/src/shape.rs` (or equivalent) | Modify | Add `fill gradient` parser branch; remove E-PAR-016 guard |
| `crates/slideforge-layout/src/shapes.rs` | Modify | Verify gradient passthrough (should be no-op if match is exhaustive-safe) |
| `crates/slideforge-pptx/src/shape.rs` (or equivalent) | Modify | Emit `<a:gradFill>` for `FillSpec::Gradient` |
| `crates/slideforge-pdf/src/shape.rs` (or equivalent) | Modify | Emit krilla `paint::LinearGradient` via `Surface::set_fill` + `draw_path` for `FillSpec::Gradient` (per export-architecture v1.2) |
| `crates/slideforge-html/src/shape.rs` (or equivalent) | Modify | Emit CSS `linear-gradient` for `FillSpec::Gradient` |
| `crates/slideforge-docx/src/shape.rs` (or equivalent) | Modify | Emit solid fallback + lint warning for `FillSpec::Gradient` |
| `crates/slideforge-layout/tests/gradient_integration.rs` | Create | End-to-end test: gradient shape in Deck → LaidOutDeck |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| STORY-028 (shape pass context) | ~1,500 |
| BC-3.04.001 v1.3 (gradient deferred surfaces section) | ~2,000 |
| `slideforge-types/src/specs.rs` | ~800 |
| Exporter files to modify (4 exporters) | ~4,000 |
| Test files to write | ~2,000 |
| **Total** | **~13,100** |

## Test Strategy

- **Unit tests**: Parser: `fill gradient "#FF0000" to "#0000FF"` → AST interim string
  in `ShapeNode.fill`; then decoded at `ShapeNode → ShapeSpec` boundary to correct
  `FillSpec::Gradient { from: Rgb(255,0,0), to: Rgb(0,0,255) }`; hex case-insensitivity
  preserved for gradient colors; E-PAR-015 still fires on short/alpha quoted hex in
  gradient stops; alt-text enforcement applies. Tests use constructed `ShapeSpec` directly
  (not end-to-end DSL pipeline — see shape-pipeline-wiring dependency note in AC-002).
- **Snapshot tests**: PPTX XML for a gradient shape — assert `a:gradFill` structure
  with two `a:gs` stops at pos=0 and pos=100000, emitted via ooxmlsdk `=0.6.1` typed builders.
- **Constructed-ShapeSpec integration test**: Directly construct
  `ShapeSpec { fill: FillSpec::Gradient { from: Rgb(255,0,0), to: Rgb(0,0,255) }, ... }`
  and drive it through `layout::run()` → exporter → `.pptx` output (assert `<a:gradFill>`);
  `.html` output (assert `linear-gradient` CSS); `.pdf` output (no panics); `.docx` output
  (assert solid fallback + lint warning). Note: an end-to-end `.sf` source file integration
  test (DSL → output) requires the shape-pipeline-wiring story (FU-SHAPE-PIPELINE-WIRING)
  and is out of STORY-072's scope.
- **Lint test**: DOCX output for a gradient shape produces a lint warning string
  containing "gradient fill downgraded to solid".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Gradient shape without alt or decorative | `LayoutError::MissingAlt` — same as solid shape |
| EC-002 | Gradient with short-form hex (e.g., `#F00 to #00F`) | E-PAR-015 on the short-form stop |
| EC-003 | Gradient in DOCX output | Solid fallback (`from` color) + lint warning |
| EC-004 | Gradient shape in multi-shape slide where another shape has missing alt | Both errors accumulated per DI-018 |
| EC-005 | Same `from` and `to` color (e.g., `"#FF0000" to "#FF0000"`) | Valid — equivalent to solid fill; no error; output is a flat gradient (visually solid) |
| EC-006 | Gradient shape with `decorative: true` | `AltText::Decorative`; PDF Artifact tag; HTML `alt=""` — gradient fill does not affect decorative semantics |

## Forbidden Dependencies

`slideforge-layout` MUST NOT depend on `slideforge-pptx`, `slideforge-docx`,
`slideforge-pdf`, or `slideforge-html`. The gradient passthrough in layout is
format-agnostic. Exporter gradient logic lives in the respective exporter crates.
Build MUST fail if those crates appear in `slideforge-layout/Cargo.toml`.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-28 | story-writer | Initial creation — deferred surface from BC-3.04.001 v1.3 "Deferred Surfaces" section; resolves STORY-TBD-shape-gradient-fills placeholder |
| 1.1 | 2026-05-29 | product-owner | Pass-9 sweep (F-P9-HIGH-001): removed false "ALREADY exists / placeholder" claims — FillSpec::Gradient is NOT in v1.0 codebase per BC-3.04.001 v1.4.2 and code audit; rewrote Dependency Anchor, Summary, Previous Story Intelligence to describe ADDING the variant as the first task; updated E-PAR-014 references to E-PAR-016 and E-PAR-013 references to E-PAR-015 per F-P9-HIGH-002 namespace collision resolution; corrected file path from specs.rs → shape_types.rs |
| 1.2 | 2026-06-07 | story-writer | Wave-5 remove-uncertainty propagation: removed pdf-writer direct dep (krilla=0.6.0 wraps it; direct dep causes version-skew risk per export-architecture v1.2); updated AC-004 PDF row and PDF task to use krilla paint::LinearGradient + Surface::set_fill + draw_path API; updated AC-004 PPTX row and PPTX task to use ooxmlsdk=0.6.1 typed builders for a:gradFill/a:gsLst/a:gs/a:lin; changed thiserror to {workspace=true} form (=2.0.18 per ADR-022). |
| 1.3 | 2026-06-08 | product-owner | Adversary Pass-1 MED-001/MED-002/OBS-072-P1-001 fixes: (MED-002) corrected ALL unquoted gradient hex examples to quoted form (`"#FF0000"` not `#FF0000`) — lexer treats `#` as line-comment when unquoted, making unquoted examples unparseable; fixed in AC-001 DSL block, AC-001 prose, Tasks unit-test bullet, Test Strategy unit-test line, Test Strategy integration-test line; (MED-001) rewrote AC-002 to accurately describe two-step AST→IR reality: parser stores interim string in `ShapeNode.fill`, structural `FillSpec::Gradient` is constructed at the `ShapeNode→ShapeSpec` decode boundary; (OBS-001) added shape-pipeline-wiring dependency note in AC-002 clarifying STORY-072 scope (parser branch + structural FillSpec + 5 exporter renderers verified via constructed-ShapeSpec tests) vs. end-to-end DSL path blocked by pre-existing FU-SHAPE-PIPELINE-WIRING gap; updated Test Strategy integration-test to constructed-ShapeSpec form. |
