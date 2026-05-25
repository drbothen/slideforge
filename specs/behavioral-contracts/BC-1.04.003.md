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

# BC-1.04.003: Reject Any Construct That Could Produce Non-Termination

## Description

The slideforge computation model (rungs 1-9) is provably terminating. `@for` iterates
only over finite collections. `@if` is a branch (not a loop). There are no user-defined
functions in v1.0. Any DSL construct that could in principle produce an infinite loop or
unbounded computation is rejected at parse time or compile time. This is a hard domain
invariant required for the < 500ms cold build guarantee and for formal verification via Kani.

## Preconditions

1. A .sf source file containing iteration (`@for`), conditional (`@if`), or data computation constructs.

## Postconditions

1. `@for item in collection:` is valid ONLY when `collection` evaluates to a finite list or map (determined at compile time from the data source schema or inline value).
2. No `@while` keyword or equivalent unbounded loop construct exists in the grammar.
3. No recursive `@include` chains are allowed (cycle detection in BC-1.01.004 covers this).
4. Variant inheritance graph must be a DAG (BC-1.07.003 covers this).
5. Any attempt to use a keyword that implies unbounded computation (e.g., a future `@while`) is rejected with E-PAR-006 as a reserved keyword.
6. The build pipeline terminates in bounded time for any valid .sf source.

## Invariants

1. The iteration depth is bounded: the total number of slide-generating iterations is bounded by the product of all data collection sizes encountered in the source.
2. No user-defined function calls are allowed (functions are reserved for v2+).
3. The ~15 built-in filter functions are all pure and non-recursive.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | @for over an HTTP data source that returns a very large collection (e.g., 100,000 rows) | No termination error — the computation terminates after all rows. Performance may be poor; a lint warning for collections > configurable threshold (1000 items) is emitted. |
| EC-002 | Nested @for: outer × inner = 1000 × 1000 iterations | Compiles and runs; 1,000,000 slides generated (memory/performance concern, not a correctness concern). |
| EC-003 | @while keyword used in source | E-PAR-006: `'while' is reserved for future iteration constructs (planned v2+)`. Exit 1. |
| EC-004 | User attempts to define a function: `@fn myfunc(x):` | E-PAR-006: `'@fn' is reserved for user-defined functions (planned v2+)`. Exit 1. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@for item in [1, 2, 3]:` (inline literal list) | 3 iterations; terminates | happy-path |
| `@for item in items:` where `items` from JSON file = 25 rows | 25 iterations; terminates | happy-path |
| `@while true:` in source | E-PAR-006: 'while' is reserved; exit 1 | error |
| `@fn compute(x):` in source | E-PAR-006: '@fn' is reserved; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | For any valid .sf AST, the evaluation terminates in O(N × M) where N = slides, M = max collection size | kani (bounded model: N ≤ 1000, M ≤ 10000) |
| VP-TBD | No cycles possible in include graph + variant graph combined | kani (DAG property on combined graph) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 |
| Capability Anchor Justification | CAP-004 ("Iteration over Data Collections") per capabilities.md §CAP-004 — termination is explicitly stated in CAP-004: "provably terminating (no unbounded loops, no user-defined functions)" |
| L2 Domain Invariants | DI-005 (computation must be provably terminating) |
| Architecture Module | slideforge-eval crate — evaluator termination proof (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.04.001 — composes with (this is the termination guarantee for BC-1.04.001's iteration)
- BC-1.01.004 — related to (cycle detection in includes is the analogous guarantee for composition)
- BC-1.07.003 — related to (DAG property for variant inheritance)

## Architecture Anchors

- `architecture/authoring-subsystem.md#computation-model` — rungs 1-9 design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
