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
capability: CAP-007
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

# BC-1.07.005: Variant That Excludes All Slides Produces Warning in Warn-Only Mode

## Description

When a variant's `include_tags` and/or `exclude_tags` filters result in zero slides
being selected, this is a validation condition requiring special handling. In strict mode
(the default), this is a validation error and no output is written for the variant. In
`--warn-only` mode, a warning is emitted and the variant produces an empty output file
(or is omitted, depending on the exporter). This covers DEC-012 from the domain edge-
case catalog.

## Preconditions

1. A variant's tag filters are applied and produce a zero-slide result set.

## Postconditions (strict mode)

1. Validation error emitted: `Variant '<name>' produces a zero-slide deck after filtering.`
2. No output file written for this variant.
3. Build exits with code 2 (validation error).

## Postconditions (--warn-only mode)

1. Lint warning emitted: `Variant '<name>' produces a zero-slide deck. No output file written.`
2. No output file is written for the zero-slide variant.
3. Build continues for other variants.
4. Build exits with code 0 if no other errors exist.

## Invariants

1. A zero-slide variant never produces an output file (strict or warn-only) — DI-017.
2. The error/warning message always names the variant.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-012) | Variant `exec` has `exclude_tags: ["*"]` (excludes all) | Validation error/warning for exec variant; other variants unaffected |
| EC-002 | Multi-variant build: two variants defined; one produces zero slides | Error/warning for zero-slide variant; other variant builds successfully |
| EC-003 | `--variant exec` targets the zero-slide variant explicitly | Strict mode: error; warn-only: warning + no output |
| EC-004 | No `--variant` flag; default variant produces zero slides | Same behavior: error in strict, warning in warn-only |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| exec variant, all slides tagged `internal`, exec excludes `internal` | Validation error in strict mode; warning in warn-only | error/warning (DEC-012) |
| Two variants; one zero-slide, one valid | Error for zero-slide variant; valid variant output produced in warn-only | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Zero-slide variant never writes output file | unit test (assert no file written) |
| VP-TBD | Warning/error names the specific variant | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 — zero-slide result from variant filtering is an explicitly specified edge case of the variant system |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error) |
| L2 Edge Cases | DEC-012 (variant that excludes all slides) |
| Architecture Module | slideforge-eval crate — variant validation (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.07.001 — depends on (this is the zero-result boundary of BC-1.07.001's filtering)
- BC-3.03.002 — composes with (strict mode no-output rule applies here too)

## Architecture Anchors

- `architecture/system-overview.md` — zero-slide variant detection

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
