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
capability: CAP-022
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
departed_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.03.004: Zero-Slide Deck Produces Validation Error

## Description

A .sf file that parses successfully but contains no `slide` blocks (only vars, set rules,
section blocks, or an empty file) is rejected at the validation stage with E-LAY-002.
A deck must contain at least one slide to be valid. This covers DEC-011.

## Preconditions

1. The .sf file has been parsed without parse errors (the file is syntactically valid).
2. After parsing, the evaluated `Deck` IR contains zero slide nodes.

## Postconditions

1. E-LAY-002 is emitted:
   `Zero-slide deck: no slide blocks found in '<file>'. A deck must contain at least one slide.`
2. No output files are written.
3. Build exits with code 2 (validation error).

## Invariants

1. This check is a validation-stage error, not a parse-stage error. A file with
   syntactically valid metadata but no slides is a different error from a file with
   parse errors.
2. A deck that has slides but all are excluded by `@if false` or variant `exclude_tags`
   also triggers E-LAY-002 (zero slides after evaluation).
3. `--warn-only` does NOT demote E-LAY-002 — a zero-slide deck produces no output
   regardless of warn-only mode (there is nothing to render as an error-slide placeholder).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | .sf file is completely empty (DEC-011 canonical) | E-LAY-002: zero slides; exit 2 |
| EC-002 | .sf file has vars: and set rules but no slide blocks | E-LAY-002: zero slides; exit 2 |
| EC-003 | Deck has slides but all are @if-excluded | E-LAY-002: zero slides after evaluation; exit 2 |
| EC-004 | Deck has one slide that has a parse error | Parse error is reported first (E-PAR-*); E-LAY-002 may also be reported if no slides survive parse; exit 1 |
| EC-005 | Zero-slide deck in --warn-only mode | E-LAY-002 still blocking; no output; exit 2 (--warn-only does not apply to zero-slide condition) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| deck.sf with only `vars: { name: "test" }` and no slides | E-LAY-002; no output files; exit 2 | error |
| Empty deck.sf | E-LAY-002; no output files; exit 2 | error |
| deck.sf with 1 slide | 0 E-LAY-002; output written; exit 0 | happy-path |
| deck.sf with 1 slide but `@if false` wrapping it | E-LAY-002 (zero slides after eval); exit 2 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Zero-slide deck always exits 2 with E-LAY-002 regardless of --warn-only flag | integration test (both with and without --warn-only) |
| VP-TBD | No output directory created for zero-slide deck | integration test (assert output dir absent after build) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 |
| Capability Anchor Justification | CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — zero-slide detection is an instance of "strict mode: validation errors produce no output"; DEC-011 explicitly names zero-slide decks |
| L2 Domain Invariants | DI-017 (strict mode produces no output on validation error) |
| Architecture Module | slideforge-eval crate — post-evaluation slide count check (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.03.002 — composes with (this BC is a specific validation error gated by the same no-output invariant as BC-3.03.002)
- BC-1.07.005 — related to (variant-excluded all slides also results in zero-slide condition)

## Architecture Anchors

- `architecture/system-overview.md` — zero-slide validation check

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
