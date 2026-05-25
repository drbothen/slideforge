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
capability: CAP-004
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

# BC-1.04.002: @for over Empty Collection Generates Zero Items Without Error

## Description

When `@for item in collection:` iterates over an empty list or empty object, it
generates zero slides or blocks. This is a valid and expected state — it is not an
error. An empty data source (e.g., "no active incidents today") is a legitimate
production case. This covers DEC-001 from the domain edge-case catalog.

## Preconditions

1. A `@for item in collection:` block exists in the deck source.
2. `collection` resolves to an empty list (`[]`) or empty map (`{}`).

## Postconditions

1. Zero slides or blocks are generated for this loop.
2. No error or warning is emitted for the empty collection itself.
3. Build continues; subsequent slides and blocks are processed normally.
4. Build exits with code 0 (unless there are other errors in the deck).

## Invariants

1. Empty iteration is never an error — it is a no-op.
2. If the empty `@for` is the only slide-generating construct in the deck, the zero-slide deck validation error (BC-3.03.004 / E-LAY-002) is triggered by the subsequent validation stage, not by `@for` itself.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-001) | `@data incidents from "incidents.json"` (empty array) / `@for i in incidents:` | Zero slides generated; no error |
| EC-002 | `@for x in []:` (empty inline literal) | Zero iterations; no error |
| EC-003 | Deck with one `@for` over empty collection and no other slides | Zero slides — E-LAY-002 at validation stage |
| EC-004 | Nested `@for` where outer loop is non-empty but inner is empty | Outer slides generated; inner produces zero inner elements per outer iteration |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data d from "empty.json"` (d=[]) / `@for x in d:` / slide block | 0 slides from loop; no error | happy-path (DEC-001) |
| `@for x in []:` / slide block | 0 slides; build continues | happy-path |
| Only construct is `@for x in []:` | E-LAY-002 at validation stage (zero-slide deck) | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty `@for` produces exactly 0 slides and 0 errors for the iteration | unit test with empty-list fixture |
| VP-TBD | Non-empty deck is unaffected by a zero-producing @for block | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 |
| Capability Anchor Justification | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 — empty collection behavior is explicitly required by CAP-004 as a valid production state |
| L2 Edge Cases | DEC-001 (empty @for loop) |
| Architecture Module | slideforge-eval crate — @for block evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.04.001 — composes with (this is the zero-item boundary case of BC-1.04.001)
- BC-3.03.004 — related to (zero-slide deck error triggered at validation stage, not @for stage)

## Architecture Anchors

- `architecture/system-overview.md` — empty iteration handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
