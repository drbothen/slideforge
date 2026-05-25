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
capability: CAP-010
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

# BC-3.01.002: Missing Required Field on Slide Type Produces Compile Error with Field Name

## Description

When a slide block is missing one or more fields that the `SlideType` trait declares as
required, the evaluator emits a structured compile error naming the missing field, the
slide type, and the source location. The error message includes the list of all required
fields for that type as a hint. Multiple missing fields on the same slide each produce
their own error entry (error accumulation per DI-018).

## Preconditions

1. A slide block has been parsed successfully (no parse errors on the block itself).
2. The evaluator's required-field pass has identified one or more absent required fields.

## Postconditions

1. One error is emitted per missing required field with format:
   `Required field '<field>' missing on <type> slide at <file>:<line>:<col>. Required fields for '<type>': [<list>].`
2. The error is accumulated (not fatal-on-first); subsequent slides are still evaluated.
3. Build exits with code 1 (parse/eval errors).
4. No output files are written (strict mode, per DI-017 and BC-3.03.002).

## Invariants

1. Error accumulation: all missing-field errors on all slides are reported in a single
   build run (DI-018). One missing-field error never prevents other slides from being checked.
2. The error message ALWAYS includes the list of required fields for the slide type
   (corrective hint per CAP-030).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide missing 3 required fields | 3 separate errors accumulated; build reports all 3; exit 1 |
| EC-002 | Multiple slides each missing one field | N errors (one per slide-field pair); all accumulated; exit 1 |
| EC-003 | Field present but provided via @if block that evaluates to false | Field is absent; missing-field error triggered (conditional absence = logical absence) |
| EC-004 | Field provided via @include that resolves to empty file | Missing-field error with resolved @include path shown |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide stat_callout:` missing `stat_1` and `stat_2` | 2 errors: "Required field 'stat_1' missing…" and "Required field 'stat_2' missing…"; exit 1 | error |
| `slide title:` with `title "Launch"` (all required fields present) | No errors; slide in Deck IR | happy-path |
| Deck with 3 slides, first and third each missing one field | 2 errors (accumulated); second slide processed normally | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Error message always includes the required field list for the slide type | unit test (parse error message string) |
| VP-TBD | N missing fields on N slides → exactly N error entries (no de-duplication or early exit) | unit test with multi-slide fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — field enforcement via SlideType trait is the central mechanism of CAP-010; error reporting with field name is the required user experience for the compile-time contract |
| L2 Domain Invariants | DI-017 (strict mode: no output on validation error), DI-018 (error accumulation in one pass) |
| Architecture Module | slideforge-eval crate — required-field validation pass (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.01.001 — composes with (this BC defines the error output; BC-3.01.001 defines the validation trigger)
- BC-1.15.001 — depends on (all errors carry file:line:col span and correction hint per CAP-030)
- BC-1.15.002 — depends on (error accumulation; multiple missing-field errors all reported in one pass)

## Architecture Anchors

- `architecture/crate-architecture.md` — required-field error format

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
