---
document_type: verification-property
vp_id: VP-022
title: "XLSX: non-whole Float 3.14 loads as Value::Float"
module: slideforge-data
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-1.03.006, DI-004]
anchored_to_bc: BC-1.03.006
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-022: XLSX Non-Whole Float Loads as Value::Float

## Property Statement

For any calamine `Data::Float(f)` in a data row where `f.fract() != 0.0` (i.e., the
value has a non-zero fractional part) and `f` is finite, the loader MUST produce
`Value::Float(OrderedFloat(f))` — NOT `Value::Int(_)`.

Formally: `f.fract() != 0.0 ∧ f.is_finite() → promote_float(f) == Ok(Value::Float(OrderedFloat(f)))`.

## Motivation

BC-1.03.006 EC-011 specifies this exact behavior. This property is the complement of
VP-021: while VP-021 proves whole-number floats promote to Int, VP-022 proves
non-whole floats do NOT promote. Both directions must be verified — a naive
implementation could incorrectly promote 3.14 to Int(3) via floor/truncation.

## Feasibility Assessment

Kani-amenable: `promote_float(f: f64) -> Result<Value, DataError>` is pure. The
non-whole check is `f.fract() != 0.0`, which Kani can reason about for specific values.
Combined with VP-021, covers the full `promote_float` decision tree.

`kani_amenable: true` — pure promotion function.

## Proof Harness Skeleton

```rust
// crates/slideforge-data/src/proofs/xlsx_float_promotion.rs (extends VP-021 module)
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn non_whole_finite_float_stays_float() {
        let f: f64 = kani::any();
        kani::assume(f.is_finite());
        kani::assume(f.fract() != 0.0);
        let result = promote_float(f);
        kani::assert!(matches!(result, Ok(Value::Float(_))));
    }
}
```

## Test Coverage (before Phase 6)

See VP-021 test module — `test_non_whole_float_stays_float` covers this property.

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Data::Float(3.14)` | `Value::Float(OrderedFloat(3.14))` | happy-path (EC-011) |
| `Data::Float(0.001)` | `Value::Float(OrderedFloat(0.001))` | happy-path |
| `Data::Float(-2.5)` | `Value::Float(OrderedFloat(-2.5))` | happy-path |
