---
document_type: verification-property
vp_id: VP-023
title: "XLSX: non-finite Float (NaN/Infinity) produces ParseError"
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

# VP-023: XLSX Non-Finite Float Produces ParseError

## Property Statement

For any calamine `Data::Float(f)` where `f.is_nan() || f.is_infinite()`, the loader
MUST return `Err(DataError::ParseError)` with a message conforming to EC-012 format
(citing the cell coordinate and the type as NaN or Infinity). The load MUST NOT
produce `Value::Float(OrderedFloat(NaN))` — `OrderedFloat` panics on NaN in `Ord`
comparisons, and non-finite values cannot be represented in slideforge's value tree.

Formally: `f.is_nan() ∨ f.is_infinite() → promote_float(f) == Err(DataError::ParseError { .. })`.

## Motivation

BC-1.03.006 EC-012 and AC-BC-003 (Item C) both explicitly prohibit non-finite floats
from passing through. `OrderedFloat<f64>` uses a total ordering that treats NaN
specially, but downstream serialization (JSON, TOML, YAML output) and arithmetic
operations would produce undefined behavior if NaN/Infinity enter the value tree.
The production-grade default (CLAUDE.md) requires an early, actionable error.

## Feasibility Assessment

Kani-amenable: the non-finite check `f.is_nan() || f.is_infinite()` is a pure boolean
predicate on f64. Kani can symbolically verify that the promote_float branch for
non-finite values always returns `Err`.

`kani_amenable: true` — pure decision in promote_float.

## Proof Harness Skeleton

```rust
// crates/slideforge-data/src/proofs/xlsx_float_promotion.rs (extends VP-021 module)
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn nan_produces_parse_error() {
        let result = promote_float(f64::NAN);
        kani::assert!(matches!(result, Err(DataError::ParseError { .. })));
    }

    #[kani::proof]
    fn pos_infinity_produces_parse_error() {
        let result = promote_float(f64::INFINITY);
        kani::assert!(matches!(result, Err(DataError::ParseError { .. })));
    }

    #[kani::proof]
    fn neg_infinity_produces_parse_error() {
        let result = promote_float(f64::NEG_INFINITY);
        kani::assert!(matches!(result, Err(DataError::ParseError { .. })));
    }
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Data::Float(f64::NAN)` | `DataError::ParseError` citing NaN | error (EC-012) |
| `Data::Float(f64::INFINITY)` | `DataError::ParseError` citing Infinity | error (EC-012) |
| `Data::Float(f64::NEG_INFINITY)` | `DataError::ParseError` | error (EC-012) |
