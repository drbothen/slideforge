---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: ["v1.2: F-098-P1-003 + F-098-P1-007 — precondition 1 widened to cover missing data: field; description updated; EC-005 added; references to DiagnosticSeverity::Error confirmed"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.11.002: Chart with Empty or Missing Data Produces Error-Slide Placeholder Not a Crash

## Description

When a `slide chart:` block either has no `data:` field at all (missing data source) or
has a `data:` binding that evaluates to an empty collection (zero rows), the chart renderer
must not crash or produce malformed output. Instead, E-LAY-003 is emitted and an error-slide
placeholder is rendered in the slide's position. This applies in both strict mode
(blocking error + non-zero exit, no output) and warn-only mode (placeholder rendered, build
continues). Both missing and empty data are treated identically — both are blocking in strict
mode, both render a placeholder in warn-only. (v1.2: precondition 1 widened per F-098-P1-007.)

## Preconditions

1. A `slide chart:` block exists with a valid `type:`, AND EITHER (a) the block has no
   `data:` field (missing data source) OR (b) the block has a `data:` binding that evaluates
   to an empty collection (zero rows or empty list).
2. (Derived from precondition 1.) The ChartRenderer plugin has not yet been invoked for
   this slide.

## Postconditions

1. E-LAY-003 is emitted: "[E-LAY-003] Chart data is empty for slide '<title>'. Rendering error-slide placeholder." (message is identical for both missing-data and empty-evaluating-data cases)
2. In strict mode (default): build exits with code 2; no output file is written.
3. In `--warn-only` mode: an error-slide placeholder is rendered at the chart slide's position in the output; remaining slides render normally.
4. The chart renderer does not panic, crash, or produce a malformed SVG.
5. The error names the slide title and data binding expression to help the user locate the source.

## Invariants

1. Empty data is a runtime condition (data is resolved at evaluate time), not a parse error.
2. The ChartRenderer plugin is never called with empty or absent data — the validator intercepts before plugin invocation. This applies to both the missing-data case (no `data:` field) and the empty-data case.
3. DI-018: this validation error is accumulated alongside other errors; it does not halt other slide rendering.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-014) | `slide chart:` + `data {{ kpis.monthly }}` where `kpis.monthly` is `[]` | E-LAY-003; error-slide placeholder in warn-only; exit 2 in strict |
| EC-002 | Data binding resolves to null (not a list) | E-EVL-006: @for over non-collection — caught before chart stage |
| EC-003 | Data has 1 row (edge of "non-empty") | Valid — chart renders normally with 1 data point |
| EC-004 | Multiple chart slides, one has empty data | Only the empty-data chart emits E-LAY-003; others render normally |
| EC-005 (F-098-P1-007) | `slide chart:` block with no `data:` field at all (missing data source) | E-LAY-003 emitted (identical to empty-data case); error-slide placeholder in warn-only; exit 2 in strict. Layout-preview use case must use --warn-only. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide chart:` + `type: bar` + `data []` (empty list literal) | E-LAY-003 warning; exit 2 in strict; no output | error (DEC-014) |
| Same with `--warn-only` | E-LAY-003 warning; error-slide placeholder in output; exit 0 | edge-case |
| `slide chart:` + `type: bar` (no `data:` field) | E-LAY-003 warning; exit 2 in strict; no output | error (EC-005, F-098-P1-007) |
| Same missing-data with `--warn-only` | E-LAY-003 warning; error-slide placeholder in output; exit 0 | edge-case (EC-005) |
| `slide chart:` + non-empty data | No E-LAY-003; chart renders; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty data input to chart renderer never panics (fuzz boundary) | cargo-fuzz: send empty data to chart pipeline |
| VP-TBD | E-LAY-003 names the slide title and data binding | unit test with fixture |
| VP-TBD | Missing `data:` field (no binding) triggers E-LAY-003 same as empty binding | unit test with fixture (EC-005) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-013 ("Chart Rendering from Data") per capabilities.md §CAP-013 |
| Capability Anchor Justification | CAP-013 ("Chart Rendering from Data") per capabilities.md §CAP-013 — "Chart exporter must not crash on empty data" is explicitly noted in CAP-013 context (DEC-014) |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error), DI-018 (error accumulation) |
| Architecture Module | slideforge-charts crate + slideforge-eval validation layer (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.11.001 — related to (this is the error path companion to BC-1.11.001)
- BC-3.03.003 — composes with (warn-only error-slide placeholder mechanism)

## Architecture Anchors

- `architecture/system-overview.md` — empty data guard in ChartRenderer

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-05-24 | product-owner | Initial draft — empty-data case only (EC-001 through EC-004). |
| 1.2 | 2026-06-11 | product-owner | F-098-P1-003 (HIGH) reclassification: E-LAY-003 is broken/exit-2-in-strict, NOT degraded/strict-overflow-gated. F-098-P1-007 (OBS adjudication, BINDING PO decision): precondition 1 widened to include missing `data:` field (no binding); EC-005 added; new test vectors for missing-data case; VP entry added. BC title updated from "Chart with Empty Data" to "Chart with Empty or Missing Data". Implementer directive: chart.rs doc comment allowing data-less charts without error must be corrected (--warn-only is the correct mechanism for layout previews). |
