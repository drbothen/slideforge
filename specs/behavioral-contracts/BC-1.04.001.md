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

# BC-1.04.001: @for over Finite Collection Generates Slides/Blocks per Item

## Description

The `@for item in collection:` construct generates one slide, content block, or section
for each element in the collection. The iteration variable `item` is bound to each
element in turn and is accessible via `{{ item.field }}` inside the loop body. The
resulting slides or blocks are inserted at the position of the `@for` block in
document order.

## Preconditions

1. A `@for item in collection:` block exists in the deck source.
2. `collection` resolves to a non-empty finite list or map.
3. The loop body contains at least one slide or element definition.

## Postconditions

1. One slide/block is generated for each element in `collection`.
2. The generated slides appear in document order at the position of the `@for` block.
3. `{{ item.field }}` inside the loop body references the current element's fields.
4. Build exits with code 0 on success.

## Invariants

1. The number of generated slides equals the length of the collection.
2. Iteration order is the same as the source collection order.
3. The loop variable is not accessible outside the loop body.
4. Iteration is always finite — see BC-1.04.003 for the termination guarantee.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Collection is an inline literal list: `@for x in [1, 2, 3]:` | 3 iterations; `x` takes values 1, 2, 3 |
| EC-002 | Collection is a map: `@for key, value in config:` | One iteration per key-value pair |
| EC-003 | `@for` generates slides that themselves contain `@if` blocks | `@if` evaluated per iteration with iteration variable in scope |
| EC-004 | `@for item in data.nested.list:` (collection from nested field access) | Collection resolved via dot notation; iterations proceed normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data items from "items.json"` (5 items) / `@for item in items:` / `slide title:` / `title "{{ item.name }}"` | 5 title slides with correct names | happy-path |
| `@for x in [1, 2, 3]:` / `slide stat:` / `stat "{{ x }}"` | 3 stat slides with values 1, 2, 3 | happy-path |
| `@for item in []:` | 0 slides generated (see BC-1.04.002) | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Slide count = collection length for any finite collection | proptest (arbitrary list lengths ≤ 10000) |
| VP-TBD | Iteration variable value in slide N matches element N of collection | unit test with known fixture |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 |
| Capability Anchor Justification | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 — this BC is the core happy-path contract for the @for construct |
| L2 Domain Invariants | DI-005 (computation must be provably terminating) |
| Architecture Module | slideforge-eval crate — @for block evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.04.002 — composes with (empty collection handling)
- BC-1.04.003 — depends on (termination guarantee)
- BC-1.02.005 — composes with (lexical scoping in nested loops)

## Architecture Anchors

- `architecture/authoring-subsystem.md#computation-model` — iteration design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
