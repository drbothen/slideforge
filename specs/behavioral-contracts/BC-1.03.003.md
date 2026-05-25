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
capability: CAP-003
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

# BC-1.03.003: Fail with Structured Error on Missing Field Access

## Description

When a `{{ name.field }}` interpolation references a field that does not exist in the
loaded data source, the evaluator produces E-DAT-005 with the full field path, data
source name, and source span. There is no silent empty-string substitution. This covers
DEC-002 from the domain edge-case catalog.

## Preconditions

1. A `@data` source has been loaded and bound to `name`.
2. A `{{ name.field }}` interpolation references a key that is absent in the data.

## Postconditions

1. E-DAT-005 is emitted: `Missing field '<field-path>' in data source '<name>' at <file>:<line>:<col>. Field does not exist in source data.`
2. Build exits with code 2 in strict mode.
3. In `--warn-only` mode: error-slide placeholder rendered for the affected slide.
4. All missing field errors are accumulated — not just the first (DI-018).

## Invariants

1. No silent substitution of empty string, null, or "undefined" for missing fields.
2. The field path in the error shows the full dot-notation path (e.g., `items[0].name`).
3. Error is emitted for EVERY missing field reference in the source.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-002) | JSON row missing a field that other rows have (inconsistent schema) | E-DAT-005 for the specific row where the field is absent |
| EC-002 | `{{ data.nested.deep.field }}` where `nested` exists but `deep` does not | E-DAT-005 with path `nested.deep` as the point of failure |
| EC-003 | Field exists but has JSON null value | No error — null is a valid value; interpolation produces "null" string or uses `| default` filter |
| EC-004 | `@for item in items:` where not all items have the accessed field | E-DAT-005 accumulated for each item missing the field |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data d from "data.json"` / `stat "{{ d.missing_field }}"` | E-DAT-005: missing field 'missing_field'; exit 2 | error |
| `@for item in items:` / `title "{{ item.name }}"` where row 2 has no `name` | E-DAT-005 for row 2; all rows reported; exit 2 | error |
| `{{ data.count }}` where count exists | Renders correctly | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every absent field access produces E-DAT-005 (never empty-string fallback) | unit test with fixture missing key |
| VP-TBD | Multiple missing fields in one build are all reported | unit test with multiple-missing fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — missing field errors are the primary error mode of data binding |
| L2 Domain Invariants | DI-006 (undefined variables are compile errors), DI-018 (error accumulation) |
| L2 Edge Cases | DEC-002 (data source returns null or missing field) |
| Architecture Module | slideforge-eval crate — field access resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.002 — related to (same pattern for DSL variables; this BC is the data-source variant)
- BC-1.03.001 — depends on (file must be loaded before field access is checked)

## Architecture Anchors

- `architecture/system-overview.md#data-binding` — field access error handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
