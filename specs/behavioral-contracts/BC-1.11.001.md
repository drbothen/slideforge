---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-013
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.11.001: slide chart: Renders bar/line/pie/scatter/area/histogram/stacked-bar as SVG

## Description

The `slide chart:` slide type accepts a chart type keyword and a data binding, and
produces an SVG chart via the ChartRenderer plugin (backed by the `plotters` crate).
Seven chart types are supported in v1.0: bar, line, pie, scatter, area, histogram,
and stacked-bar. The rendered SVG is embedded in all output formats: embedded in PPTX
as a media part, inlined or as `<img>` in HTML, and rendered to paths for PDF.

## Preconditions

1. A `slide chart:` block exists with a `type:` field set to one of the 7 supported types.
2. A `data:` binding provides the data collection for the chart (via `@data` variable or inline list).
3. The `alt "..."` field is present (required by BC-5.01.001).
4. The data collection is non-empty (empty data handled by BC-1.11.002).

## Postconditions

1. A valid SVG string is produced by the ChartRenderer plugin for the requested chart type.
2. The SVG uses the active brand's color palette for chart series colors.
3. The SVG has `viewBox`, `width`, and `height` attributes with absolute (px) values.
4. `aria-label` and `<title>` are injected with the alt text value.
5. The SVG contains no `<script>` or `<foreignObject>` elements.
6. For PPTX: SVG is embedded as `/ppt/media/chartN.svg` with correct relationship reference.
7. Build exits with code 0.

## Invariants

1. No Node.js or external process is spawned — charts are rendered entirely by the `plotters` crate.
2. Chart SVG is produced via the `ChartRenderer` plugin trait; bundled implementation uses plotters.
3. Brand colors are applied to series; chart axis labels use brand font family.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `type: bar` with 20 data series (many bars) | SVG produced; no cropping or crash; may trigger E-LAY-001 if canvas overflow detected |
| EC-002 | `type: pie` with a single data series (1 slice) | Valid SVG with single full-circle slice |
| EC-003 | Unknown chart type (e.g., `type: radar`) | E-PAR-007 style error: unknown chart type in v1.0; lists supported types |
| EC-004 | Scatter chart with data containing null coordinates | Compile error (E-DAT-005): missing field in data source |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide chart:` + `type: bar` + 5-row data + `alt "Q1 Revenue"` | Valid SVG with 5 bars; aria-label="Q1 Revenue"; exit 0 | happy-path |
| `slide chart:` + `type: pie` + 3-item data | Valid pie chart SVG; brand colors applied; exit 0 | happy-path |
| `slide chart:` + `type: radar` (unsupported) | Compile error: unsupported chart type; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Rendered SVG contains no <script> or <foreignObject> (PPTX safety) | unit test: parse SVG, check for forbidden elements |
| VP-TBD | All 7 chart types produce valid SVG without crash | integration test: one fixture per chart type |
| VP-TBD | Brand colors appear in chart series paths | unit test: extract fill colors from SVG, match to brand palette |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-013 ("Chart Rendering from Data") per capabilities.md §CAP-013 |
| Capability Anchor Justification | CAP-013 ("Chart Rendering from Data") per capabilities.md §CAP-013 — "Support bar, line, pie, scatter, area, histogram, and stacked bar in v1.0" is verbatim from CAP-013 |
| L2 Domain Invariants | DI-001 (alt text required on all visual elements) |
| Architecture Module | slideforge-charts crate — ChartRenderer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.11.002 — related to (error path for empty chart data)
- BC-5.01.001 — depends on (alt text required on chart; checked before rendering)

## Architecture Anchors

- `architecture/system-overview.md` — ChartRenderer plugin + plotters integration

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
