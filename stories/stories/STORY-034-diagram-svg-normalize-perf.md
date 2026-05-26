---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-034
title: "SVG Normalization via usvg + Performance Gate"
epic: EPIC-12
wave: 3
points: 5
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.12.003]
verification_properties: []
nfr_refs: [NFR-003, NFR-004, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-diagrams
target_module: slideforge-diagrams
subsystems: [SS-11]
depends_on: [STORY-033]
blocks:
  - STORY-037
  - STORY-043
  - STORY-046
estimated_days: 2
---

# STORY-034: SVG Normalization via usvg + Performance Gate

## Subsystem Anchor Justification

SS-11 (Diagrams) owns SVG normalization because it is a mandatory post-processing
step within the diagram rendering pipeline, immediately after `mermaid-rs-renderer`
produces the raw SVG. All format exporters receive the normalized form. The
normalization is not the exporter's responsibility — it is the diagram crate's
responsibility to deliver PPTX-safe SVG.

## Dependency Anchor Justifications

- Depends on STORY-033: `RawDiagramSvg` (output of mermaid-rs-renderer) is the input
  to the usvg normalization step. This story wraps STORY-033's `render()` output
  with the normalization pass.
- Blocks STORY-037 (PPTX): The PPTX exporter embeds `NormalizedDiagramSvg` not
  `RawDiagramSvg`. STORY-037 depends on normalized SVG being available.
- Blocks STORY-043 (PDF) and STORY-046 (HTML): Same dependency on normalized SVG.

## Summary

Extend the diagram rendering pipeline with a mandatory usvg 0.47.0 normalization
step that runs on every `RawDiagramSvg` before it is stored in `LaidOutDeck`:

1. Pass raw SVG string through `usvg::Tree::from_str()` for parsing.
2. Serialize back with `usvg::TreeWriting::to_string()`.
3. The normalized output is guaranteed: no `<foreignObject>`, no `<script>`,
   no CSS `@keyframes`, no percentage dimensions, all `<use>` references resolved.

The public interface changes from `RawDiagramSvg` to `NormalizedDiagramSvg`. All
exporters use `NormalizedDiagramSvg`.

Update Criterion benchmarks to measure the combined render + normalization time
to confirm the NFR-003/NFR-004 budgets still hold after normalization.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.12.003 | SVG normalized via usvg before PPTX embedding (no foreignObject, absolute dims) | AC-001 through AC-007 |

## Acceptance Criteria

### AC-001: usvg normalization applied to every diagram SVG
(traces to BC-1.12.003 invariant 1 — normalization applied to EVERY diagram SVG)

The diagram rendering pipeline calls `usvg_normalize(raw_svg: RawDiagramSvg) ->
Result<NormalizedDiagramSvg, DiagramError>` after every successful `render()` call.
There is no bypass path. If `render()` succeeds but `usvg_normalize()` fails,
`DiagramError::SvgNormalizationFailed` is returned with `E-EXP-004`.

### AC-002: Normalized SVG has no foreignObject
(traces to BC-1.12.003 postcondition 1)

After usvg normalization, the SVG string does not contain any `<foreignObject>`
element. A post-normalization assertion scans the string for `foreignObject`
(case-insensitive) and panics in debug builds if found (programming error, not
user error — usvg should always remove foreignObject).

### AC-003: Normalized SVG has no script elements
(traces to BC-1.12.003 postcondition 2)

After normalization, the SVG string does not contain any `<script>` element.
Same post-normalization assertion as AC-002.

### AC-004: Normalized SVG has no CSS @keyframes or class-based styles
(traces to BC-1.12.003 postcondition 3)

usvg inlines all CSS `class="..."` styles into presentation attributes and strips
`<style>` blocks containing `@keyframes`. The normalized SVG contains no `<style>`
element and no `@keyframes` text.

### AC-005: Normalized SVG has absolute pixel dimensions
(traces to BC-1.12.003 postcondition 4)

The normalized SVG root element has `width` and `height` as absolute pixel values
(e.g., `width="800"` or `width="800px"`), NOT as percentage values. usvg computes
absolute dimensions from the `viewBox` attribute when the source SVG uses percentage
dimensions. A post-normalization test verifies: `width` and `height` attributes are
present and do not end with `%`.

### AC-006: All use references resolved
(traces to BC-1.12.003 postcondition 5)

After normalization, the SVG contains no `<use>` elements. usvg inlines all `<use
href="#symbol">` references. A post-normalization assertion checks for absence of
`<use` in the string.

### AC-007: usvg failure produces E-EXP-004
(traces to BC-1.12.003 postcondition 7)

If `usvg::Tree::from_str()` returns an error (malformed SVG from the renderer),
`DiagramError::SvgNormalizationFailed { slide_title }` is returned. The error code
`E-EXP-004` is emitted. Exit code 3 in strict mode. Error-slide placeholder in
warn-only mode.

### AC-008: Performance budget maintained after normalization
(traces to NFR-003 and NFR-004)

Update Criterion benchmarks to measure the full pipeline: `render()` +
`usvg_normalize()`. The combined time must still satisfy:
- Cold (first call, includes font DB init): < 200ms total
- Warm (subsequent calls, font DB cached): < 10ms total

The usvg normalization step is expected to add < 1ms overhead (empirical from S14).
If it exceeds 10% of the warm budget (1ms), flag for architect review.

### AC-009: NormalizedDiagramSvg stored in LaidOutDeck
(traces to BC-1.12.003 invariant 2 — normalization before format embedding)

`LaidOutSlide` stores `NormalizedDiagramSvg`, not `RawDiagramSvg`. The type change
ensures all exporters work with normalized SVG. `NormalizedDiagramSvg(String)` is a
distinct newtype from `RawDiagramSvg(String)` so that the type system prevents
skipping normalization.

## Tasks

- [ ] Implement `usvg_normalize(raw: RawDiagramSvg) -> Result<NormalizedDiagramSvg, DiagramError>` in `src/normalize.rs`
- [ ] Define `NormalizedDiagramSvg(String)` newtype in `src/types.rs`
- [ ] Add `DiagramError::SvgNormalizationFailed { slide_title: Arc<str> }` variant
- [ ] Add post-normalization assertions: no `foreignObject`, no `script`, no `<use`, no `%` dimensions
- [ ] Update `DiagramRendererImpl::render()` to call `usvg_normalize` after `mermaid_rs_renderer::render()` succeeds
- [ ] Update `LaidOutSlide` `FrameContent::DiagramSvg(...)` to use `NormalizedDiagramSvg`
- [ ] Update Criterion benchmarks to measure full render + normalize pipeline
- [ ] Write unit tests:
  - SVG with `<foreignObject>` → after normalization, no foreignObject
  - SVG with `width="100%"` → after normalization, width is absolute px
  - SVG with `<use href="#symbol">` → after normalization, no `<use>` elements
  - Malformed SVG → `DiagramError::SvgNormalizationFailed`
  - post-normalization assertion triggers (debug-only panic test)
- [ ] Write `insta` snapshot test: flowchart SVG before vs after normalization

## Previous Story Intelligence

STORY-033 produces `RawDiagramSvg`. This story wraps that output with normalization.
The type change from `RawDiagramSvg` to `NormalizedDiagramSvg` propagates through:
1. `DiagramRendererImpl::render()` return type
2. `LaidOutSlide::FrameContent::DiagramSvg` variant type
3. All exporter call sites (STORY-037, 043, 046)

Coordinate with STORY-037/043/046 implementers: they should declare their dependency
on `NormalizedDiagramSvg` type from `slideforge-diagrams`. The type signature change
is a compile-time guarantee that normalization ran.

## Architecture Compliance Rules

1. **Normalization is mandatory (BC-1.12.003 invariant 1)**: No bypass path exists.
   The type system enforces this: exporters receive `NormalizedDiagramSvg`, not
   `RawDiagramSvg`. Passing raw SVG to an exporter is a compile error.
2. **Normalization runs after renderer, before exporters (BC-1.12.003 invariant 2)**:
   The call sequence is `render() → usvg_normalize() → store NormalizedDiagramSvg →
   exporter embeds`. No exporter calls normalization itself.
3. **Pure core post-normalization (SS-11)**: `usvg` normalization is a pure
   parse-and-reserialize operation. No I/O, no network.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-diagrams` (self, from STORY-033) | workspace | Extends rendering pipeline |
| `usvg` | `=0.47.0` | SVG normalization (foreignObject removal, absolute dims, use resolution) |
| `criterion` | `=0.5.1` | Updated benchmarks |
| `insta` | `=1.39.0` | Before/after normalization snapshot |

Note: `usvg` 0.47.0 is pinned. Do NOT use `resvg` for normalization — `resvg` is
a rasterizer. `usvg` is the normalization library (it parses and re-serializes SVG
into a normalized form).

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-diagrams/src/normalize.rs` | Create | usvg normalization pipeline |
| `crates/slideforge-diagrams/src/types.rs` | Modify | Add `NormalizedDiagramSvg` newtype |
| `crates/slideforge-diagrams/src/error.rs` | Modify | Add `SvgNormalizationFailed` variant |
| `crates/slideforge-diagrams/src/lib.rs` | Modify | Call `usvg_normalize` in render pipeline |
| `crates/slideforge-diagrams/benches/cold_render.rs` | Modify | Add normalization to benchmark |
| `crates/slideforge-diagrams/benches/warm_render.rs` | Modify | Add normalization to benchmark |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,000 |
| BC-1.12.003 | ~1,500 |
| STORY-033 types and pipeline | ~1,500 |
| usvg 0.47.0 API reference | ~1,000 |
| Test files to write | ~2,000 |
| **Total** | **~8,000** |

## Test Strategy

- **Unit tests**: All post-normalization assertions pass for reference Mermaid SVG;
  `width="100%"` → absolute px after normalization; no `<use>` after normalization;
  malformed SVG → E-EXP-004 (not panic).
- **Snapshot test**: Raw SVG vs normalized SVG for a flowchart — diff shows style
  inlining and absolute dimension conversion.
- **Benchmark gates**: Combined render + normalize < 200ms cold, < 10ms warm.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SVG with `<use href="#symbol">` | Resolved and inlined; no `<use>` in output |
| EC-002 | SVG with `width="100%"` | Normalized to absolute px from viewBox |
| EC-003 | SVG with `<foreignObject>` | Removed by usvg; no `<foreignObject>` in output |
| EC-004 | usvg fails on malformed SVG | E-EXP-004 emitted; error-slide placeholder in warn-only |
| EC-005 | SVG with CSS `@keyframes` | Animation removed by usvg normalization |

## Forbidden Dependencies

Same as STORY-033. No exporter crates. `usvg` is the normalization library — do not
add `resvg` (rasterizer) to this crate's dependencies unless PDF path rendering
requires it (that is STORY-030's concern in `slideforge-math`).
