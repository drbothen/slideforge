---
document_type: adr
adr_id: ADR-007
title: Data-reactive computation model (rungs 1-9)
status: accepted
date: 2026-05-24
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-007: Data-Reactive Computation Model (Rungs 1-9)

## Context

slideforge must support data-driven presentations where content is computed from external
sources. The computation model must be provably terminating (DI-005) and have no implicit
type coercion (DI-004). Q1 decision established a nine-rung computation ladder.

## Decision

The computation model supports exactly rungs 1-9 in v1.0:
1. Static literals (strings, numbers, booleans)
2. Variable assignments (`@set name = value`)
3. External data binding (`@data name = file.json`)
4. `{{ expr }}` interpolation with arithmetic and pipe filters (~15 built-in functions)
5. Conditional rendering (`@if/@elif/@else`)
6. Collection iteration (`@for item in collection`)
7. Cross-file composition (`@include`, `@import`)
8. Variant-based deck segmentation (`@variant`)
9. Register selection (`notes:`, `report:`, `detail:`)

No user-defined functions. No recursion. No `@while`. Deferred to v2+.

## Consequences

**Termination guarantee (DI-005):** The grammar structurally excludes non-terminating
constructs. `@for` iterates over finite data collections only. `@if` is a branch.
No recursion paths exist. Kani can prove termination for `@for` over bounded collections
(VP-003).

**No implicit coercion (DI-004):** The lexer preserves `"1.10"` as a string token (not
float). `true`/`false` are boolean keywords. All other tokens are strings until the
evaluator explicitly converts. Type conversion requires explicit pipe filters (e.g.,
`| to_int`). Wrong-type usage is E-TYP-NNN.

**Evaluator is pure (purity-boundary-map.md):** The evaluator resolves expressions from
the parsed AST and data-source values. All data fetching happens BEFORE evaluation, in
`slideforge-data`. The evaluator receives data as `Value` structs — no I/O in eval.

**v2 extension path:** User-defined functions (`@fn`) and mixins (`@mixin`) are reserved
keywords (DI-021). Attempting to use them in v1.0 produces E-PAR-XXX with a message
naming the v2 feature. This prevents silent misinterpretation.
