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
capability: CAP-020
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

# BC-5.01.003: Missing label on Color-Coded Element Is Compile Error

## Description

Any slide element that uses color to convey meaning — `severity_cards` (severity levels),
`status` indicator, `progress_bar` fill, `weighted_composite` scores — must declare a
`label "..."` field encoding the meaning in text form. Omitting `label` on a color-coded
element produces E-A11-002 with element location. This enforces WCAG 1.4.1 (Use of Color)
at compile time. `label` is distinct from `alt` — it describes what the color means,
not what the visual element is.

## Preconditions

1. A color-coded element type (`severity_cards`, `status`, `progress_bar`,
   `weighted_composite`) is declared in a .sf source file.
2. The element does NOT have a non-empty `label "..."` field.

## Postconditions

1. E-A11-002 is emitted: `Missing label on color-coded element '<type>' '<identifier>'
   at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".`
2. Build exits with code 2 (validation error) in strict mode.
3. No output is produced in strict mode (DI-017).
4. In `--warn-only` mode: warning emitted; build continues with error-slide placeholder.

## Invariants

1. Color-coded elements ALWAYS require `label "..."`. There is no decorative: true
   opt-out for color-coded elements (the label requirement is independent of alt text).
   (DI-002)
2. The `label "..."` value must be non-empty and non-whitespace (same rule as alt text
   in BC-5.01.001).
3. The validator checks ALL slide types that use color to convey meaning. If a new slide
   type is added that uses color semantically, it must be added to the label-required list.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `severity_cards` with `label ""` (empty string) | E-A11-002: empty label is not valid (same rule as empty alt) |
| EC-002 | `status` element with `label "  "` (whitespace only) | E-A11-002: whitespace-only label rejected |
| EC-003 | `progress_bar` with both `label "50% complete"` and color fill | No error; label "50% complete" embedded in all output formats |
| EC-004 | Multiple severity_cards on one slide, one missing label | E-A11-002 per missing label; error count = number of violations |
| EC-005 | severity_cards with `decorative: true` | The `decorative` flag does NOT exempt color-coded elements from label requirement. E-A11-002 still emitted. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `severity_cards:` block with no `label` | E-A11-002 at the element's source location | error |
| `severity_cards:` with `label "HIGH: Immediate action required"` | No error; label embedded in output | happy-path |
| `progress_bar:` with `label ""` | E-A11-002: empty label | error |
| 3 `status:` elements, 2 missing label | 2× E-A11-002; build exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-A11-002 count = number of color-coded elements missing label | unit test with N-element fixture |
| VP-TBD | decorative: true does not suppress E-A11-002 for color-coded elements | unit test: color-coded + decorative → still E-A11-002 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 |
| Capability Anchor Justification | CAP-020 ("Accessibility Validation") per capabilities.md §CAP-020 — "label '...' on all color-coded elements (severity_cards, status, progress_bar), compile error if absent" is verbatim from CAP-020 |
| L2 Domain Invariants | DI-002 (color-coded elements must have text labels) |
| Architecture Module | slideforge-eval or slideforge-validate crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.01.001 — related to (same enforcement pattern as alt text; different field and invariant)
- BC-3.03.002 — depends on (label validation error triggers the no-output gate in strict mode)

## Architecture Anchors

- `architecture/plugin-architecture.md#accessibility-validation` — color-coded element label enforcement

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
