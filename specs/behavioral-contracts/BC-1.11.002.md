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

# BC-1.11.002: Chart with Empty Data Produces Error-Slide Placeholder Not a Crash

## Description

When a `slide chart:` block's data binding evaluates to an empty collection (zero rows),
the chart renderer must not crash or produce malformed output. Instead, a validation
warning (E-LAY-003) is emitted and an error-slide placeholder is rendered in the slide's
position. This applies in both strict mode (warning + non-zero exit) and warn-only mode
(placeholder rendered, build continues).

## Preconditions

1. A `slide chart:` block exists with a valid `type:` and a `data:` binding.
2. The data binding evaluates to an empty collection (zero rows or empty list).

## Postconditions

1. E-LAY-003 is emitted: "Chart data is empty for slide '<title>'. Rendering error-slide placeholder."
2. In strict mode (default): build exits with code 2; no output file is written.
3. In `--warn-only` mode: an error-slide placeholder is rendered at the chart slide's position in the output; remaining slides render normally.
4. The chart renderer does not panic, crash, or produce a malformed SVG.
5. The error names the slide title and data binding expression to help the user locate the source.

## Invariants

1. Empty data is a runtime condition (data is resolved at evaluate time), not a parse error.
2. The ChartRenderer plugin is never called with empty data — the validator intercepts before plugin invocation.
3. DI-018: this validation error is accumulated alongside other errors; it does not halt other slide rendering.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-014) | `slide chart:` + `data {{ kpis.monthly }}` where `kpis.monthly` is `[]` | E-LAY-003; error-slide placeholder in warn-only; exit 2 in strict |
| EC-002 | Data binding resolves to null (not a list) | E-EVL-006: @for over non-collection — caught before chart stage |
| EC-003 | Data has 1 row (edge of "non-empty") | Valid — chart renders normally with 1 data point |
| EC-004 | Multiple chart slides, one has empty data | Only the empty-data chart emits E-LAY-003; others render normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide chart:` + `type: bar` + `data []` (empty list literal) | E-LAY-003 warning; exit 2 in strict; no output | error (DEC-014) |
| Same with `--warn-only` | E-LAY-003 warning; error-slide placeholder in output; exit 0 | edge-case |
| `slide chart:` + non-empty data | No E-LAY-003; chart renders; exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty data input to chart renderer never panics (fuzz boundary) | cargo-fuzz: send empty data to chart pipeline |
| VP-TBD | E-LAY-003 names the slide title and data binding | unit test with fixture |

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

- `architecture/authoring-subsystem.md#chart-renderer` — empty data guard in ChartRenderer

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
