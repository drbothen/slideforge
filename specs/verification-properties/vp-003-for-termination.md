---
document_type: verification-property
vp_id: VP-003
title: "@for over bounded collection always terminates"
module: slideforge-syntax
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-1.04.003, DI-005]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-003: @for Over Bounded Collection Always Terminates

## Property Statement

For any `@for` iteration over a collection of bounded size N, the expansion
of the `@for` body terminates in O(N × body_size) time and produces exactly
N copies of the body (or 0 if the collection is empty per BC-1.04.002).

Formally: the `@for` expansion function is a bounded-time pure function.
There is no path in the evaluator that causes it to loop infinitely given
a finite-sized `Value::List`.

## Motivation

DI-005 (domain invariant): computation must be provably terminating to satisfy
the NFR-001 (< 500ms cold build) and the product promise of deterministic output.
BC-1.04.003 rejects non-terminating constructs at parse time — the grammar
structurally excludes `@while`, user functions, and recursion. Kani proves the
evaluator upholds this even under adversarial inputs.

## Feasibility Assessment

Feasible. The `@for` expansion is a `for item in collection` Rust iteration over a
`Vec<Value>`. Kani can prove loop termination for bounded vector sizes. The proof
bounds: `|collection| ≤ 1000` items, body depth ≤ 5 nested @for levels. This covers
all realistic slideforge use cases.

## Proof Harness Skeleton

```rust
// crates/slideforge-syntax/src/proofs/for_termination.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(1010)]
    fn for_expansion_terminates() {
        let size: usize = kani::any();
        kani::assume(size <= 1000);
        let collection: Vec<Value> = (0..size).map(|_| Value::Int(0)).collect();
        let body = ForBody::simple_text(); // single field assignment body
        let result = expand_for("item", &collection, &body);
        // Result must have exactly `size` expanded items
        kani::assert!(result.len() == size);
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit tests: `test_for_empty_collection`, `test_for_single_item`,
`test_for_nested_loops` in `slideforge-eval/src/tests/`.
