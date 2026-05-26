---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-031
title: "Chart Renderer: bar/line/pie/scatter/area/histogram/stacked-bar"
epic: EPIC-11
wave: 3
points: 8
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.11.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-charts
target_module: slideforge-charts
subsystems: [SS-12]
depends_on: [STORY-001, STORY-002, STORY-011]
blocks:
  - STORY-032
  - STORY-037
  - STORY-043
  - STORY-046
estimated_days: 4
---

# STORY-031: Chart Renderer — bar/line/pie/scatter/area/histogram/stacked-bar

## Subsystem Anchor Justification

SS-12 (Charts) owns this story's scope because `slideforge-charts` is the exclusive
owner of chart SVG generation per ARCH-INDEX Subsystem Registry. All 7 chart types
are implemented as a single `ChartRenderer` plugin within this crate. No exporter
performs chart computation.

## Dependency Anchor Justifications

- Depends on STORY-001: `Value`, `Brand`, `Rgb` types from `slideforge-types` are
  the input data format for chart data series.
- Depends on STORY-002: `ChartRenderer` plugin trait is declared in
  `slideforge-plugin-api`.
- Depends on STORY-011: The evaluator resolves `@data` bindings and `{{ expr }}`
  before chart rendering; `slideforge-charts` receives evaluated `Value::List` data.
- Blocks STORY-032: Empty data handling extends this story's `ChartRenderer`.
- Blocks exporters: PPTX (037), PDF (043), HTML (046) all embed the SVG this
  story produces.

## Summary

Implement `slideforge-charts` as the `ChartRenderer` plugin. Using the `plotters`
0.3.7 crate with its SVG backend, produce chart SVGs for all 7 types:

1. `bar` — vertical bar chart with grouped series
2. `line` — line chart with optional data points
3. `pie` — pie/donut chart
4. `scatter` — x/y scatter plot
5. `area` — area (filled line) chart
6. `histogram` — frequency distribution bars
7. `stacked-bar` — stacked vertical bars

SVG output is brand-aware (color palette), PPTX-safe (no script, no foreignObject),
and accessible (aria-label + title element).

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.11.001 | slide chart: renders all 7 chart types as SVG via plotters | AC-001 through AC-008 |

## Acceptance Criteria

### AC-001: ChartRenderer plugin trait implementation
(traces to BC-1.11.001 invariant 2)

`slideforge_charts::ChartRendererImpl` implements
`slideforge_plugin_api::ChartRenderer`:
```rust
pub trait ChartRenderer: Send + Sync {
    fn render(
        &self,
        spec: &ChartSpec,
        brand: &Brand,
    ) -> Result<ChartSvg, ChartError>;
}
```
`ChartSpec` carries: `chart_type: ChartType`, `data: Vec<DataSeries>`,
`title: Option<Arc<str>>`, `x_label: Option<Arc<str>>`, `y_label: Option<Arc<str>>`,
`alt: Arc<str>`. `ChartSvg` is a newtype `ChartSvg(String)` containing the SVG.

### AC-002: All 7 chart types produce valid SVG
(traces to BC-1.11.001 postcondition 1)

`ChartType` enum has variants: `Bar`, `Line`, `Pie`, `Scatter`, `Area`,
`Histogram`, `StackedBar`. Each variant produces a non-empty, well-formed SVG string
via `plotters` 0.3.7 SVGBackend. Unknown chart type keyword produces
`ChartError::UnsupportedType { name }` (E-PAR-007-class).

### AC-003: SVG uses brand color palette for series
(traces to BC-1.11.001 postcondition 2)

Chart series colors are taken from `brand.accent_colors` in order. The first series
uses `accent_colors[0]`, second uses `accent_colors[1]`, etc., cycling if more series
than colors. If `brand.accent_colors` is empty, fallback palette:
`[#003766, #FF6F00, #009E60, #7030A0, #FF0000, #0070C0, #00B050]`.

### AC-004: SVG has viewBox and absolute dimensions
(traces to BC-1.11.001 postcondition 3)

The root `<svg>` element has:
- `viewBox="0 0 {width} {height}"` (default: `viewBox="0 0 800 450"`)
- `width="{width}px"` and `height="{height}px"` (no percentage values)
- Default output size: 800 × 450 pixels (16:9 proportion matching default slide dimensions)

### AC-005: Accessibility attributes injected
(traces to BC-1.11.001 postcondition 4)

Before returning `ChartSvg`, inject:
- `<title>{alt text}</title>` as the first child of the root `<svg>` element
- `aria-label="{alt text}"` attribute on the root `<svg>` element
- `role="img"` attribute on the root `<svg>` element

### AC-006: No script or foreignObject in output
(traces to BC-1.11.001 postcondition 5)

The SVG produced by `plotters` SVGBackend must not contain `<script>` or
`<foreignObject>` elements. The `plotters` SVGBackend produces pure path/shape
SVG — no scripts. A post-generation assertion verifies this before returning.

### AC-007: PPTX embedding relationship
(traces to BC-1.11.001 postcondition 6)

`ChartSvg` is stored in the `LaidOutSlide`'s `FrameContent::ChartSvg(ChartSvg)`
variant. The PPTX exporter (STORY-037) reads this frame and writes the SVG to
`/ppt/media/chartN.svg` with a correct relationship entry in the slide's
`.rels` file. This story establishes the `FrameContent::ChartSvg` variant but does
not implement the PPTX serialization (that is STORY-037's responsibility).

### AC-008: Brand font applied to axis labels
(traces to BC-1.11.001 invariant 3)

Axis labels and legend text use `brand.font_family` (e.g., `"Calibri"`). The
`plotters` `TextStyle` is constructed with the brand font family name. If the font
is not available in the rendering context, `plotters` falls back to its built-in
font — this is acceptable (a lint warning is emitted, not an error).

## Tasks

- [ ] Create `crates/slideforge-charts/` with `Cargo.toml`
- [ ] Add to workspace members
- [ ] Define `ChartType`, `ChartSpec`, `DataSeries`, `DataPoint`, `ChartSvg`, `ChartError` types in `src/types.rs`
- [ ] Add `ChartRenderer` trait to `slideforge-plugin-api` (if not already from STORY-002)
- [ ] Implement `ChartRendererImpl` struct in `src/lib.rs`
- [ ] Implement bar chart renderer in `src/bar.rs` using `plotters` `ChartBuilder`
- [ ] Implement line chart renderer in `src/line.rs`
- [ ] Implement pie chart renderer in `src/pie.rs`
- [ ] Implement scatter chart renderer in `src/scatter.rs`
- [ ] Implement area chart renderer in `src/area.rs`
- [ ] Implement histogram renderer in `src/histogram.rs`
- [ ] Implement stacked-bar renderer in `src/stacked_bar.rs`
- [ ] Implement `inject_aria_attributes` post-processor in `src/accessibility.rs`
- [ ] Implement `assert_no_forbidden_elements` validator in `src/safety.rs`
- [ ] Add `FrameContent::ChartSvg(ChartSvg)` variant to `slideforge-layout` (coordinate with STORY-026)
- [ ] Write unit tests:
  - all 7 chart types produce non-empty SVG without panic
  - `viewBox` and `width`/`height` attributes present with px values
  - `aria-label` and `<title>` injected with alt text
  - no `<script>` or `<foreignObject>` in output
  - brand colors applied in order to series
  - unknown chart type → `ChartError::UnsupportedType`
- [ ] Write `insta` snapshot tests for SVG output of each chart type with reference data

## Previous Story Intelligence

N/A — first story in EPIC-11. `plotters` is a well-established pure-Rust charting
library. Key constraint: use `plotters::backend::SVGBackend` (not a bitmap backend).
The SVGBackend renders directly to a `String` — no intermediate PNG step.

## Architecture Compliance Rules

1. **SS-12 is Pure core**: No I/O, no network, no Node.js. `plotters` SVGBackend
   writes to an in-memory `String`. All chart rendering is synchronous and pure.
2. **ChartRenderer trait compliance (DI-008)**: `ChartRendererImpl` MUST use only
   the `slideforge-plugin-api::ChartRenderer` public trait. No internal type bypass.
3. **No Node.js (BC-1.11.001 invariant 1)**: `plotters` is a pure Rust library.
   Never spawn a subprocess for chart rendering.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `Brand`, `Rgb`, `Value` |
| `slideforge-plugin-api` | workspace | `ChartRenderer` trait |
| `plotters` | `=0.3.7` | SVG chart rendering |
| `plotters-backend` | `=0.3.7` | SVGBackend (bundled with plotters) |
| `quick-xml` | `=0.36.0` | Accessibility attribute injection into SVG |
| `thiserror` | `=2.0.18` | `ChartError` |
| `insta` | `=1.39.0` | snapshot tests |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-charts/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-charts/src/lib.rs` | Create | `ChartRendererImpl` |
| `crates/slideforge-charts/src/types.rs` | Create | `ChartType`, `ChartSpec`, `DataSeries`, `ChartSvg`, `ChartError` |
| `crates/slideforge-charts/src/bar.rs` | Create | Bar chart |
| `crates/slideforge-charts/src/line.rs` | Create | Line chart |
| `crates/slideforge-charts/src/pie.rs` | Create | Pie chart |
| `crates/slideforge-charts/src/scatter.rs` | Create | Scatter chart |
| `crates/slideforge-charts/src/area.rs` | Create | Area chart |
| `crates/slideforge-charts/src/histogram.rs` | Create | Histogram |
| `crates/slideforge-charts/src/stacked_bar.rs` | Create | Stacked bar |
| `crates/slideforge-charts/src/accessibility.rs` | Create | aria-label + title injection |
| `crates/slideforge-charts/src/safety.rs` | Create | SVG safety assertion |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-1.11.001 | ~1,800 |
| `plotters` SVGBackend API | ~2,000 |
| `slideforge-types` Brand/Rgb | ~1,000 |
| Test files to write | ~4,000 |
| **Total** | **~11,300** |

## Test Strategy

- **Unit tests per chart type**: Each of the 7 chart types is tested with:
  - 3-row input data → non-empty SVG string
  - SVG contains `viewBox` and absolute `width`/`height`
  - SVG contains `<title>` and `aria-label`
  - SVG does not contain `<script>` or `<foreignObject>`
- **Brand color test**: Brand with 2 accent colors → first series uses `accent_colors[0]`,
  second uses `accent_colors[1]`
- **Snapshot tests**: Reference SVG for each chart type with a fixed 5-row dataset

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 20 data series (many bars) | SVG produced; no crash; may trigger canvas overflow warning in layout |
| EC-002 | Pie chart with 1 slice | Valid SVG: single full-circle arc |
| EC-003 | Unknown chart type (`type: radar`) | `ChartError::UnsupportedType { name: "radar" }` (E-PAR-007-class) |
| EC-004 | Scatter chart with null coordinates | `ChartError::MissingDataField` (E-DAT-005-class) |
| EC-005 | Brand with empty accent_colors | Fallback palette used; no error |

## Forbidden Dependencies

`slideforge-charts` MUST NOT depend on any exporter crate, `slideforge-data`,
`slideforge-layout`, `slideforge-cli`, `slideforge-syntax`, or `slideforge-eval`.
The chart renderer is a pure data → SVG transformer.
