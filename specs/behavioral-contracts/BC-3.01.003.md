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

# BC-3.01.003: Unknown Slide Type Keyword Produces Compile Error with Suggestion

## Description

When a slide block declares an unknown keyword (e.g., `slide spreasheet:`), the parser
emits E-PAR-007 with the unknown keyword and a closest-match suggestion from the 31
registered slide type names. This prevents typos from silently producing no-op slide blocks.
The error is accumulated with all other parse errors before the build halts.

## Preconditions

1. The parser encounters a `slide <keyword>:` block where `<keyword>` is not in the
   registered slide type table.
2. The keyword is not a reserved future keyword (if reserved, E-PAR-006 is emitted instead).

## Postconditions

1. E-PAR-007 is emitted:
   `Unknown slide type '<keyword>' at <file>:<line>:<col>. Did you mean '<closest-match>'?`
2. The unknown slide block is skipped; subsequent slides are still parsed.
3. Build exits with code 1.
4. No output files are written.

## Invariants

1. The suggestion algorithm uses edit-distance (Levenshtein or Jaro-Winkler) against
   the 31 registered slide type names. A suggestion is always provided if any registered
   type has edit-distance ≤ 3.
2. If no type is within edit-distance 3, the error message lists all 31 type names as a
   reference. (DI-018: error accumulation)
3. The slide type registry is the single source of truth — no hard-coded if-chains per type.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slide conetnt:` (typo of "content") | E-PAR-007: "Did you mean 'content'?" |
| EC-002 | `slide xyz:` (completely unrelated name) | E-PAR-007 with no suggestion (edit-distance > 3); message lists all 31 types |
| EC-003 | `slide raw:` (reserved keyword) | E-PAR-006 (reserved keyword error), not E-PAR-007, with v2 feature description |
| EC-004 | `slide CONTENT:` (wrong case — keyword is case-sensitive) | E-PAR-007: "Did you mean 'content'?" (case-insensitive suggestion; keyword itself is case-sensitive) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slide conetnt: title "Risks"` | E-PAR-007: Unknown slide type 'conetnt'. Did you mean 'content'?; exit 1 | error |
| `slide content: title "Risks"` | Parsed successfully; no E-PAR-007 | happy-path |
| `slide flibbertigibbet: title "X"` | E-PAR-007: Unknown slide type 'flibbertigibbet'. Known types: [list of 31]; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | One-character typos always produce a meaningful suggestion | unit test (typo matrix against all 31 type names) |
| VP-TBD | Error message always lists at least one suggestion or the full type list | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 |
| Capability Anchor Justification | CAP-010 ("31 Built-in Slide Types") per capabilities.md §CAP-010 — validation that a declared keyword is a known type is the gating check for invoking any of the 31 built-in types |
| L2 Domain Invariants | DI-018 (error accumulation), DI-021 (reserved keywords rejected with descriptive errors) |
| Architecture Module | slideforge-syntax crate — slide-type keyword table; slideforge-eval crate — unknown-type error path (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-3.01.001 — composes with (unknown type check precedes required-field check)
- BC-1.01.005 — related to (reserved keyword errors use a different error code, E-PAR-006)
- BC-1.15.001 — depends on (error carries file:line:col span and correction hint)

## Architecture Anchors

- `architecture/layout-subsystem.md#slide-type-registry` — registration of 31 built-in types

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
